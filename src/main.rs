#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;

use config::Config;
use rocket::http::RawStr;
use rocket::State;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use rocket_cors::CorsOptions;
use subprocess::{Popen, PopenConfig, Redirection};

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
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

// Arc for thread safe multiple ownership.
// Mutex for thread safe interior mutability.
// Option because there might not be a subprocess handle
struct SubProcessControl {
    sp: Arc<Mutex<Option<Popen>>>, //TODO: This should really maintain references to n running processes one for each request origin
    terminators: Arc<Mutex<HashMap<String, ProcInfo>>>,
}

fn send_command(
    command: &str,
    sub_proc: &Popen,
    term: &ProcInfo,
) -> Result<String, std::io::Error> {
    let nl_command = command.to_owned() + "\n";
    sub_proc
        .stdin
        .as_ref()
        .expect("stdin is redirected")
        .write_all(nl_command.as_bytes())?;
    read_from_proc(sub_proc, term)
}

fn read_from_proc(sub_proc: &Popen, term: &ProcInfo) -> Result<String, std::io::Error> {
    let mut buf: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(sub_proc.stdout.as_ref().expect("stdout is redirected"));
    reader.read_until(term.term_char as u8, &mut buf)?;
    let mut output = String::from_utf8(buf).unwrap();
    output = output[0..(output.len() - (term.term_len + 1))].to_string();
    Ok(output)
}

fn init(maybe_sp: &mut Option<Popen>, term: &ProcInfo) -> Result<String, std::io::Error> {
    *maybe_sp = Some(
        Popen::create(
            &[&term.path],
            PopenConfig {
                stdout: Redirection::Pipe,
                stdin: Redirection::Pipe,
                ..Default::default()
            },
        )
        .unwrap(),
    );
    read_from_proc(maybe_sp.as_ref().unwrap(), term)
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
    let sub_proc_settings = sp_control.terminators.lock().unwrap();
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
            terminators: Arc::new(Mutex::new(settings_map)),
        })
        .manage(HitCount(AtomicUsize::new(0)))
}

fn main() {
    rocket().launch();
}
