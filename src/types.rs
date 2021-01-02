use rocket::{response::Responder, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use subprocess::Popen;

#[derive(Debug)]
pub struct ErrString(pub String);

impl<'r> Responder<'r> for ErrString {
    fn respond_to(self, request: &Request) -> rocket::response::Result<'r> {
        self.0.respond_to(request)
    }
}

impl From<std::io::Error> for ErrString {
    fn from(e: std::io::Error) -> Self {
        ErrString(e.to_string())
    }
}
impl From<&str> for ErrString {
    fn from(s: &str) -> Self {
        ErrString(s.to_owned())
    }
}

#[derive(Deserialize)]
pub struct Command {
    pub command: String,
}

pub struct SubProcessControl {
    // Arc for thread safe multiple ownership.
    // Mutex for thread safe interior mutability.
    // Option because there might not be a subprocess handle.
    pub sub_proc: Arc<Mutex<Option<Popen>>>,
    pub proc_info: Arc<Mutex<HashMap<String, ProcInfo>>>,
}
#[derive(Serialize)]
pub struct Response {
    pub message: String,
}

#[derive(Debug)]
pub struct HitCount(pub AtomicUsize);

#[derive(Debug, Deserialize, Clone)]
pub struct ProcInfo {
    pub path: String,
    pub override_prompt: String,
    pub term_char: char,
    pub term_len: usize,
}
