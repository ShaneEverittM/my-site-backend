use rocket::{get, post, State};
use rocket_contrib::json::Json;
use subprocess::Redirection;

use crate::{
    subprocess_control::SubProcessControl,
    types::{Command, RouteError::LogicError, RouteResponse},
};

#[post("/projects/<program_name>", format = "json", data = "<body>")]
pub fn project(
    program_name: String,
    body: Json<Command>,
    sp_control: State<SubProcessControl>,
) -> RouteResponse {
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
        None => return Err(LogicError("unrecognized program name")),
    };

    if sub_proc_opt.is_none() {
        // No subprocess.
        if command == "init" {
            // Frontend is (re)-loading the page.
            SubProcessControl::init(&mut sub_proc_opt, term)
        } else {
            // Frontend is trying to send a command.
            Err(LogicError("Process is not initialized"))
        }
    } else if command == "init" {
        // There is a subprocess and the fronted is (re)-loading the page.
        sub_proc_opt
            .take()
            .expect("In this block, old is Some")
            .terminate()?;

        SubProcessControl::init(&mut sub_proc_opt, term)
    } else {
        // There is a subprocess and the frontend is sending a command to it.
        SubProcessControl::send_command(
            command,
            &sub_proc_opt
                .as_ref()
                .expect("In this block, sub_proc_opt is Some"),
            term,
        )
    }
}

#[post("/projects/bash", format = "json", data = "<body>")]
pub fn bash(body: Json<Command>) -> RouteResponse {
    if body.command == "init" {
        Ok("ready".into())
    } else {
        Ok(subprocess::Exec::shell(&body.command)
            .stdout(Redirection::Pipe)
            .capture()?
            .stdout_str())
    }
}

#[get("/<name>")]
pub fn hello(name: String) -> String {
    format!("Hello, {}!", name)
}
