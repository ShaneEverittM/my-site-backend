use rocket::{
    response::{self, Responder},
    Request,
};
use serde::Deserialize;

pub type RouteResponse = Result<String, ErrString>;

#[derive(Debug)]
pub struct ErrString(pub String);

impl<'r> Responder<'r> for ErrString {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
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

impl From<String> for ErrString {
    fn from(s: String) -> Self {
        ErrString(s)
    }
}

#[derive(Deserialize)]
pub struct Command {
    pub command: String,
}
