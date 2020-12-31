#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;

use rocket::http::{RawStr, Status};
use rocket::State;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use rocket_cors::CorsOptions;
use std::ffi::OsString;
use subprocess::{Popen, PopenConfig, Redirection};

use serde::{Deserialize, Serialize};

use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

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
}

#[post("/filesystem", format = "json", data = "<body>")]
fn filesystem(body: Json<Command>, process: State<SubProcessControl>) -> Json<Response> {
    eprintln!("Received command: {}", body.command);
    let sub_proc_option: &mut Option<Popen> = &mut process.inner().sp.lock().unwrap();
    if body.command == "init" {
        if sub_proc_option.is_some() {
            return Json(Response {
                message: "already initialized".into(),
            });
        }
        *sub_proc_option = Some(
            Popen::create(
                &["./fsystem"],
                PopenConfig {
                    stdout: Redirection::Pipe,
                    stdin: Redirection::Pipe,
                    ..Default::default()
                },
            )
            .unwrap(),
        );
        eprintln!("Created!");
        Json(Response {
            message: "Initialized".into(),
        })
    } else {
        eprintln!("{:?}", body.command);
        let sub_proc = if let Some(sub_proc) = sub_proc_option.as_mut() {
            sub_proc
        } else {
            return Json(Response {
                message: "You must initialize first".into(),
            });
        };
        //let com = sub_proc.communicate_start()
        if let Some(e_status) = sub_proc.poll() {
            eprintln!(
                "The subprocess has terminated with exit status: {:?}",
                e_status
            );
            return Json(Response {
                message: "The process is exited oops".into(),
            });
        }
        let (out, _err) = sub_proc
            .communicate(Some(&format!("{}\n", &body.command)))
            .unwrap();
        Json(Response {
            message: out.unwrap(),
        })
    }
}

fn send_command(command: String, sub_proc: &Popen) -> Result<String, std::io::Error> {
    sub_proc
        .stdin
        .as_ref()
        .expect("Cannot send a command to a subprocess that does not have stdin redirected")
        .write_all(command.as_bytes())?;
    read_from_proc(sub_proc)
}

fn read_from_proc(sub_proc: &Popen) -> Result<String, std::io::Error> {
    let mut buf: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(
        sub_proc
            .stdout
            .as_ref()
            .expect("Cannot read a response from a process without stdout redirected"),
    );
    reader.read_until(b'>', &mut buf)?;
    Ok(String::from_utf8(buf).unwrap())
}

fn init(maybe_sp: &mut Option<Popen>, name: String) -> Result<String, std::io::Error> {
    *maybe_sp = Some(
        Popen::create(
            &[String::from("./") + &name],
            PopenConfig {
                stdout: Redirection::Pipe,
                stdin: Redirection::Pipe,
                ..Default::default()
            },
        )
        .unwrap(),
    );
    read_from_proc(maybe_sp.as_ref().unwrap())
}

#[post("/projects/<name>", format = "json", data = "<body>")]
fn terminal(
    name: String,
    body: Json<Command>,
    sp_control: State<SubProcessControl>,
) -> Result<String, String> {
    eprintln!("Received command: {} for program {}", body.command, name);

    let Command { command } = &*body;

    let maybe_sp = &mut *sp_control.sp.lock().unwrap();

    match maybe_sp {
        None if command == "init" => {
            // Subprocess is not initialized yet and frontend is sending init command
            // Initialize normally
            let output = init(maybe_sp, name).unwrap();
            Ok(output)
        }
        // Subprocess is not initialized yet and frontend is sending some other command
        // Throw error
        None => Err(String::from("Process is not initialized")),

        Some(_) if command == "init" => {
            // There is a currently running subprocess and frontend is sending init command
            // Reinitialize
            let output = init(maybe_sp, name).unwrap();
            Ok(output)
        }
        Some(sp) => {
            // There is a currently running subprocess and frontend is sending some other command
            // Run the command
            eprintln!("Running command normally");
            let output = send_command(command.clone() + "\n", sp).unwrap();
            dbg!(&output);
            Ok(output)
        }
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![hello, terminal])
        .attach(CorsOptions::default().to_cors().unwrap())
        .manage(SubProcessControl {
            sp: Arc::new(Mutex::new(None)),
        })
        .manage(HitCount(AtomicUsize::new(0)))
}

fn main() {
    rocket().launch();
}
