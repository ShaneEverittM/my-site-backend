use rocket::{get, http::RawStr, post, State};
use rocket_contrib::json::Json;
use std::sync::atomic::Ordering;
use subprocess::Redirection;

use crate::subprocess::{init, send_command};
use crate::types::{Command, ErrString, HitCount, Response, SubProcessControl};

#[post("/projects/<program_name>", format = "json", data = "<body>")]
pub fn project(
    program_name: String,
    body: Json<Command>,
    sp_control: State<SubProcessControl>,
) -> Result<String, ErrString> {
    // Pull command out of request body.
    let Command { ref command } = body.into_inner();

    // Get lock for subprocess.
    // TODO: retrieve based on http session.
    let mut sub_proc_opt = sp_control
        .sub_proc
        .lock()
        .expect("Thread should not panic with lock");

    // Get lock for subprocess info.
    let sub_proc_settings = sp_control
        .proc_info
        .lock()
        .expect("Thread should not panic with lock");

    // Get the terminating character for the subprocess.
    let term = match sub_proc_settings.get(&program_name) {
        Some(term) => term,
        None => return Err("unrecognized program name".into()),
    };

    if sub_proc_opt.is_none() {
        // No subprocess.
        if command == "init" {
            // Frontend is (re)-loading the page.
            init(&mut sub_proc_opt, term).map_err(|e| e.into())
        } else {
            // Frontend is trying to send a command.
            Err("Process is not initialized".into())
        }
    } else if command == "init" {
        // There is a subprocess and the fronted is (re)-loading the page.
        sub_proc_opt
            .take()
            .expect("In this block, old is Some")
            .terminate()?;

        init(&mut sub_proc_opt, term).map_err(|e| e.into())
    } else {
        // There is a subprocess and the frontend is sending a command to it.
        send_command(
            command,
            &sub_proc_opt
                .as_ref()
                .expect("In this block, sub_proc_opt is Some"),
            term,
        )
        .map_err(|e| e.into())
    }
}

#[post("/projects/bash", format = "json", data = "<body>")]
pub fn bash(body: Json<Command>) -> Result<String, ErrString> {
    if body.command == "init" {
        Ok("ready".into())
    } else {
        let out = subprocess::Exec::shell(&body.command)
            .stdout(Redirection::Pipe)
            .capture()
            .map_err(|e| ErrString(e.to_string()))?
            .stdout_str();
        Ok(out)
    }
}

#[get("/<name>")]
pub fn hello(name: &RawStr, hit_count: State<HitCount>) -> Json<Response> {
    hit_count.0.fetch_add(1, Ordering::Relaxed);
    Json(Response {
        message: format!(
            "Hello, {}! (for the {:?}th time)",
            name.as_str(),
            hit_count.0
        ),
    })
}
