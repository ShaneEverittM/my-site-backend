#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;
use rocket::http::{Method, RawStr};
use rocket::{get, post, routes, State};
use rocket_contrib::json::Json;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

// for subprocess management
use std::io::prelude::*;
use std::io::BufReader;
use subprocess::{Popen, PopenConfig, Redirection};

//for Json
use serde::{Deserialize, Serialize};

//state management
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

struct HitCount(AtomicUsize);
#[get("/<name>")]
fn hello(name: &RawStr, hit_count: State<HitCount>) -> String {
    hit_count.0.fetch_add(1, Ordering::Relaxed);
    format!(
        "Hello, {}! (for the {:?}th time)",
        name.as_str(),
        hit_count.0
    )
}

struct SubProcessControl {
    sp: Mutex<Option<Popen>>, // TODO: extend to be a Mutex<HashMap<http_session, Option<Popen>>>
}

#[derive(Deserialize)]
struct Command {
    command: String,
}

#[derive(Serialize)]
struct SubProcOutput {
    msg: String,
}
fn send_command(command: String, sub_proc: &Popen) -> std::result::Result<String, std::io::Error> {
    sub_proc
        .stdin
        .as_ref()
        .expect("Cannot send a command to a subprocess that does not have stdin redirected")
        .write_all(command.as_bytes())?;
    read_from_proc(sub_proc)
}

fn read_from_proc(sub_proc: &Popen) -> std::result::Result<String, std::io::Error> {
    eprintln!("Going to read");
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

#[post("/filesystem", format = "json", data = "<body>")]
fn filesystem(body: Json<Command>, sp_state: State<SubProcessControl>) -> Json<SubProcOutput> {
    let Command { mut command } = body.into_inner();
    command.push('\n');
    let sp_control: &SubProcessControl = sp_state.inner();
    //get the lock once, 4head
    let mut lock = sp_control.sp.lock().unwrap();
    // If some, then just send the command and return the output
    if let Some(sp) = lock.as_mut() {
        return Json(SubProcOutput {
            msg: send_command(command, sp).unwrap(),
        });
    } else {
        // otherwise spawn and return initial output
        *lock = Some(
            Popen::create(
                &["./fsystem"],
                PopenConfig {
                    stdout: Redirection::Pipe,
                    stdin: Redirection::Pipe,
                    ..Default::default()
                },
            )
            .expect("Couldn't spawn child process"),
        );

        let _ = read_from_proc(lock.as_ref().unwrap()).unwrap(); //throw away the first output, frontend handles it
        return Json(SubProcOutput {
            msg: send_command(command, lock.as_ref().unwrap()).unwrap(),
        });
    }
}
fn make_cors() -> rocket_cors::Cors {
    // let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:3000"]); //, "http://192.168.1.49:3000"]);
    CorsOptions {
        allowed_methods: vec![Method::Get, Method::Post, Method::Options]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Invalid cors options")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![hello, filesystem])
        .attach(make_cors())
        .manage(SubProcessControl {
            sp: Mutex::new(None),
        })
        .manage(HitCount(std::sync::atomic::AtomicUsize::from(0)))
}

fn main() {
    rocket().launch();
}
