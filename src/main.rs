#![feature(proc_macro_hygiene, decl_macro)]

extern crate rocket;
use rocket::http::RawStr;

use rocket::http::Method;
use rocket::{get, routes};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};

use rocket_contrib::json::Json;
use serde::Serialize;

#[derive(Serialize)]
struct Greeting {
    message: String,
}

#[get("/hello/<name>")]
fn hello(name: &RawStr) -> Json<Greeting> {
    Json(Greeting {
        message: format!("Hello, {}!", name.as_str()).into(),
    })
}
// Because CORS be like that
fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:3000"]);
    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn main() {
    rocket::ignite()
        .mount("/", routes![hello])
        .attach(make_cors())
        .launch();
}
