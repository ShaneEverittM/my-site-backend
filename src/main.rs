#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;
use rocket::http::RawStr;
use rocket::State;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use rocket_cors::CorsOptions;
use subprocess::*;

use serde::{Deserialize, Serialize};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
struct Response {
    message: String,
    err: String,
}

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
        err: "".into(),
    })
}

#[derive(Deserialize)]
struct Command {
    command: String,
}
struct SubProcessControl {
    sp: Arc<Mutex<Option<Popen>>>, //TODO: This should really maintain references to n running processes one for each request origin
}

#[post("/filesystem", format = "json", data = "<body>")]
fn filesystem(body: Json<Command>, process: State<SubProcessControl>) -> Json<Response> {
    eprintln!("Recieved command: {}", body.command);
    let sub_proc_option: &mut Option<Popen> = &mut process.inner().sp.lock().unwrap();
    if body.command == "init" {
        if sub_proc_option.is_some() {
            return Json(Response {
                message: "aready initialized".into(),
                err: "".into(),
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
            err: "".into(),
        })
    } else {
        eprintln!("{:?}", body.command);
        let sub_proc = if let Some(sub_proc) = sub_proc_option.as_mut() {
            sub_proc
        } else {
            return Json(Response {
                message: "You must initialize first".into(),
                err: "".into(),
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
                err: "".into(),
            });
        }
        let (out, err) = sub_proc
            .communicate(Some(&format!("{}\n", &body.command)))
            .unwrap();
        Json(Response {
            message: out.unwrap(),
            err: err.unwrap_or_default(),
        })
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![hello, filesystem])
        .attach(CorsOptions::default().to_cors().unwrap())
        .manage(SubProcessControl {
            sp: Arc::new(Mutex::new(None)),
        })
}

fn main() {
    rocket().launch();
}
