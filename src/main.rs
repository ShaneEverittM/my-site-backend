#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;

mod routes;
mod subprocess_control;
mod types;

use crate::{
    routes::*,
    subprocess_control::{ProcInfo, SubProcessControl},
    types::HitCount,
};

use config::Config;
use rocket::routes;
use rocket_cors::CorsOptions;

use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
};

fn read_config() -> HashMap<String, ProcInfo> {
    let mut settings = Config::default();

    settings
        .merge(config::File::with_name("./bins/Info"))
        .expect("File exists");

    settings
        .try_into::<HashMap<String, ProcInfo>>()
        .expect("File format matches the try_into type parameter")
}

fn initialize_sp_control(config: HashMap<String, ProcInfo>) -> SubProcessControl {
    SubProcessControl {
        sub_proc: Arc::new(Mutex::new(None)),
        proc_info: Arc::new(Mutex::new(config)),
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![hello, bash, project])
        .attach(CorsOptions::default().to_cors().unwrap())
        .manage(initialize_sp_control(read_config()))
        .manage(HitCount(AtomicUsize::new(0)))
}

fn main() {
    rocket().launch();
}
