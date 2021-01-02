#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;

mod routes;
mod subprocess;
mod types;

use crate::routes::*;
use config::Config;
use rocket::routes;
use rocket_cors::CorsOptions;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use types::{HitCount, ProcInfo, SubProcessControl};

fn rocket() -> rocket::Rocket {
    let mut settings = Config::default();

    settings
        .merge(config::File::with_name("./bins/Info"))
        .expect("File exists");

    let settings_map = settings
        .try_into::<HashMap<String, ProcInfo>>()
        .expect("File format matches the try_into type parameter");

    let initial_sp_control = SubProcessControl {
        sub_proc: Arc::new(Mutex::new(None)),
        proc_info: Arc::new(Mutex::new(settings_map)),
    };

    rocket::ignite()
        .mount("/", routes![hello, bash, project])
        .attach(CorsOptions::default().to_cors().unwrap())
        .manage(initial_sp_control)
        .manage(HitCount(AtomicUsize::new(0)))
}

fn main() {
    rocket().launch();
}
