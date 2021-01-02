#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod routes;
mod subprocess_control;
mod types;
mod utils;

use crate::subprocess_control::SubProcessControl;

use rocket_cors::CorsOptions;

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![routes::hello, routes::bash, routes::project])
        .attach(CorsOptions::default().to_cors().unwrap())
        .manage(SubProcessControl::new(utils::read_config()))
}

fn main() {
    rocket().launch();
}
