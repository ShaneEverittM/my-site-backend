#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;

use config::Config;
use rocket::http::RawStr;
use rocket::State;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use rocket_cors::CorsOptions;
use subprocess::{Popen, PopenConfig, PopenError, Redirection::Pipe};

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
struct Response {
    message: String,
}

#[derive(Debug)]
struct HitCount(AtomicUsize);

#[get("/<name>")]
fn hello(name: &RawStr, hit_count: State<HitCount>) -> Json<Response> {
    hit_count.0.fetch_add(1, Ordering::Relaxed);
    Json(Response {
        message: format!(
            "Hello, {}! (for the {:?}th time)",
            name.as_str(),
            hit_count.0
        ),
    })
}

#[derive(Deserialize)]
struct Command {
    command: String,
}

struct SubProcessControl {
    // Arc for thread safe multiple ownership.
    // Mutex for thread safe interior mutability.
    // Option because there might not be a subprocess handle.
    sp: Arc<Mutex<Option<Popen>>>,
    proc_info: Arc<Mutex<HashMap<String, ProcInfo>>>,
}

fn send_command(command: &str, sub_proc: &Popen, term: &ProcInfo) -> io::Result<String> {
    // Get write handle to subprocess's stdin.
    let mut proc_input = sub_proc.stdin.as_ref().expect("stdin is redirected");

    // Send command.
    let nl_command = command.to_owned() + "\n";
    proc_input.write_all(nl_command.as_bytes())?;

    // Read output resulting from command.
    read_from_proc(sub_proc, term)
}

fn read_from_proc(sub_proc: &Popen, term: &ProcInfo) -> io::Result<String> {
    // Get read handle to subprocess's stdout and create a buffered reader.
    let proc_output = sub_proc.stdout.as_ref().expect("stdout is redirected");
    let mut reader = BufReader::new(proc_output);

    // Read until the terminating character. This character is supplied from the config file
    // and is guaranteed to only appear after the subprocess is done printing. In the case of
    // shell style interfaces, this will be the prompt, in other cases it will just be eof.
    let mut buf: Vec<u8> = Vec::new();
    reader.read_until(term.term_char as u8, &mut buf)?;

    // Ignore bad characters.
    let mut output = String::from_utf8_lossy(&buf).to_string();

    // Strip process's prompt in favor of the frontend terminal's prompt.
    output = output[0..(output.len() - (term.term_len + 1))].to_string();

    Ok(output)
}

fn init(maybe_sp: &mut Option<Popen>, term: &ProcInfo) -> io::Result<String> {
    // Configuration for the subprocess, must redirect stdin and stdout in order to forward
    // user commands and send output to frontend.
    let config = PopenConfig {
        stdout: Pipe,
        stdin: Pipe,
        ..Default::default()
    };

    // Create subprocess from backend root relative path.
    let new_sp = Popen::create(&[&term.path], config).map_err(|popen_err| match popen_err {
        PopenError::IoError(e) => e,
        PopenError::LogicError(msg) => io::Error::new(io::ErrorKind::InvalidInput, msg),
        _ => io::Error::new(io::ErrorKind::InvalidInput, "Unrecognized error variant"),
    })?;

    // Read first output from process.
    let output = read_from_proc(&new_sp, term)?;

    // Place new subprocess handle in state.
    *maybe_sp = Some(new_sp);

    Ok(output)
}

#[post("/projects/<program_name>", format = "json", data = "<body>")]
fn terminal(
    program_name: String,
    body: Json<Command>,
    sp_control: State<SubProcessControl>,
) -> Result<String, String> {
    eprintln!(
        "Received command: {} for program {}",
        body.command, program_name
    );

    let Command { command } = &*body;

    let mut sub_proc_opt = sp_control.sp.lock().unwrap();
    let sub_proc_settings = sp_control.proc_info.lock().unwrap();
    let term = sub_proc_settings.get(&program_name).unwrap();

    if sub_proc_opt.is_none() {
        if command == "init" {
            let output = init(&mut sub_proc_opt, term).unwrap();
            Ok(output)
        } else {
            Err(String::from("Process is not initialized"))
        }
    } else if command == "init" {
        let old = sub_proc_opt.take();
        if let Err(msg) = old.unwrap().terminate() {
            eprintln!("{}", msg);
        }

        let output = init(&mut sub_proc_opt, term).unwrap();
        Ok(output)
    } else {
        eprintln!("Running command normally");

        let output = send_command(command, &sub_proc_opt.as_ref().unwrap(), term).unwrap();
        dbg!(&output);
        Ok(output)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcInfo {
    path: String,
    term_char: char,
    term_len: usize,
}

fn rocket() -> rocket::Rocket {
    let mut settings = Config::default();
    settings.merge(config::File::with_name("Info")).unwrap();
    let settings_map = settings.try_into::<HashMap<String, ProcInfo>>().unwrap();
    dbg!(&settings_map);

    rocket::ignite()
        .mount("/", routes![hello, terminal])
        .attach(CorsOptions::default().to_cors().unwrap())
        .manage(SubProcessControl {
            sp: Arc::new(Mutex::new(None)),
            proc_info: Arc::new(Mutex::new(settings_map)),
        })
        .manage(HitCount(AtomicUsize::new(0)))
}

fn main() {
    rocket().launch();
}
