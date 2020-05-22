#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;
use rocket::http::RawStr;
use rocket::State;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use rocket_cors::CorsOptions;
use std::process::Command;

use serde::{Deserialize, Serialize};

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Serialize)]
struct Greeting {
    message: String,
}

struct HitCount(AtomicUsize);

#[get("/<name>")]
fn hello(name: &RawStr, hit_count: State<HitCount>) -> Json<Greeting> {
    hit_count.0.fetch_add(1, Ordering::Relaxed);
    Json(Greeting {
        message: format!(
            "Hello, {}! (for the {:?}th time)",
            name.as_str(),
            hit_count.0
        )
        .into(),
    })
}

#[derive(Deserialize)]
struct MyCommand<'a> {
    command: &'a str,
}

#[post("/filesystem", format = "json", data = "<body>")]
fn filesystem(body: Json<MyCommand>) -> Json<Greeting> {
    eprintln!("{:?}", body.command);
    let output = Command::new("sh")
        .arg("-c")
        .arg(body.command)
        .output()
        .expect("failed");
    Json(Greeting {
        message: String::from_utf8(output.stdout).unwrap(),
    })
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![hello, filesystem])
        .attach(CorsOptions::default().to_cors().unwrap())
        .manage(HitCount(AtomicUsize::new(0)))
}

fn main() {
    rocket().launch();
}
