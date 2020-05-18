#![feature(proc_macro_hygiene, decl_macro)]

extern crate rocket;
use rocket::http::RawStr;
use rocket::State;
use rocket::{get, routes};
use rocket_contrib::json::Json;
use rocket_cors::{AllowedOrigins, Cors, CorsOptions};

use serde::Serialize;

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Serialize)]
struct Greeting {
    message: String,
}

struct HitCount(AtomicUsize);

#[get("/hello/<name>")]
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

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:3000"]);
    CorsOptions {
        allowed_origins,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![hello])
        .attach(make_cors())
        .manage(HitCount(AtomicUsize::new(0)))
}

fn main() {
    rocket().launch();
}
