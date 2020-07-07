#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;
use rocket::http::Method;
use rocket::routes;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};

use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use subprocess::Popen;

pub mod models;
pub mod routes;

pub struct SubProcessControl {
    pub sp: Mutex<Option<Popen>>, // TODO: extend to be a Mutex<HashMap<http_session, Option<Popen>>>
}

pub struct HitCount(pub AtomicUsize);

fn main() {
    rocket::ignite()
        .mount("/", routes![routes::hello, routes::filesystem])
        .attach(make_cors())
        .manage(SubProcessControl {
            sp: Mutex::new(None),
        })
        .manage(HitCount(std::sync::atomic::AtomicUsize::from(0)))
        .launch();
}

fn make_cors() -> Cors {
    let _allowed_origins = AllowedOrigins::some_exact(&["http://localhost:3000"]); //, "http://192.168.1.49:3000"]);
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
