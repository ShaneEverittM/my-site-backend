use rocket::{
    response::{self, Responder},
    Request,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RouteError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    PopenError(#[from] subprocess::PopenError),

    #[error("invalid input: {0}")]
    LogicError(&'static str),
}

pub type RouteResponse = Result<String, RouteError>;

#[derive(Deserialize)]
pub struct Command {
    pub command: String,
}
