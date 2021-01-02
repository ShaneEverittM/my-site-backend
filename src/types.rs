use rocket::{response::Responder, Request};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicUsize;

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

#[derive(Serialize)]
pub struct Response {
    pub message: String,
}

#[derive(Debug)]
pub struct HitCount(pub AtomicUsize);
