use rocket::http::RawStr;
use rocket::{get, post, State};
use rocket_contrib::json::Json;

// subprocess management
use std::io::prelude::*;
use std::io::BufReader;
use subprocess::{Popen, PopenConfig, Redirection};

//sent and recieved data
use crate::models::*;

//state types
use crate::HitCount;
use crate::SubProcessControl;

#[get("/<name>")]
pub fn hello(name: &RawStr, hit_count: State<HitCount>) -> String {
    hit_count
        .0
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!(
        "Hello, {}! (for the {:?}th time)",
        name.as_str(),
        hit_count.0
    )
}

#[post("/projects/<exe>", format = "json", data = "<body>")]
pub fn filesystem(
    exe: String,
    body: Json<Command>,
    sp_state: State<SubProcessControl>,
) -> Json<SubProcOutput> {
    let Command { mut command } = body.into_inner();
    command.push('\n');
    let sp_control: &SubProcessControl = sp_state.inner();
    //get the lock once, 4head
    let mut lock = sp_control.sp.lock().unwrap();
    // If some, then just send the command and return the output
    if let Some(sp) = lock.as_mut() {
        return Json(SubProcOutput {
            msg: send_command(command, sp).unwrap(),
        });
    } else {
        // otherwise spawn and return initial output
        let path = String::from("./") + &exe;
        *lock = Some(
            Popen::create(
                &[path],
                PopenConfig {
                    stdout: Redirection::Pipe,
                    stdin: Redirection::Pipe,
                    ..Default::default()
                },
            )
            .expect("Couldn't spawn child process"),
        );
        if exe != "prmanager" {
            let _ = read_from_proc(lock.as_ref().unwrap()).unwrap(); //throw away the first output, frontend handles it
        }
        return Json(SubProcOutput {
            msg: send_command(command, lock.as_ref().unwrap()).unwrap(),
        });
    }
}

fn send_command(command: String, sub_proc: &Popen) -> std::result::Result<String, std::io::Error> {
    sub_proc
        .stdin
        .as_ref()
        .expect("Cannot send a command to a subprocess that does not have stdin redirected")
        .write_all(command.as_bytes())?;
    read_from_proc(sub_proc)
}

fn read_from_proc(sub_proc: &Popen) -> std::result::Result<String, std::io::Error> {
    // let mut buf: Vec<u8> = Vec::new();
    let mut buf: Vec<u8> = vec![0; 1024]; // better be big enough otherwise read will block
    let mut reader = BufReader::new(
        sub_proc
            .stdout
            .as_ref()
            .expect("Cannot read a response from a process without stdout redirected"),
    );
    std::thread::sleep(std::time::Duration::from_millis(10));
    reader.read(&mut buf)?;
    eprintln!("Done");
    // reader.read_until(b'>', &mut buf)?;
    Ok(String::from_utf8(buf).unwrap())
}
