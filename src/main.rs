#![feature(proc_macro_hygiene, decl_macro)]

extern crate rocket;
use rocket::http::RawStr;
use rocket::State;
use rocket::{get, routes};
use rocket_contrib::json::Json;
use rocket_cors::CorsOptions;

use serde::Serialize;

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

#[get("/secret/<name>")]
fn secret_hello(name: &RawStr, hit_count: State<HitCount>) -> Json<Greeting> {
    hit_count.0.fetch_add(1, Ordering::Relaxed);
    Json(Greeting {
        message: format!(
            "Psst! Hello, {}! (for the {:?}th time)",
            name.as_str(),
            hit_count.0
        )
        .into(),
    })
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![hello, secret_hello])
        .attach(CorsOptions::default().to_cors().unwrap())
        .manage(HitCount(AtomicUsize::new(0)))
}

fn main() {
    rocket().launch();
}
