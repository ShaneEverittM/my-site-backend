use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Command {
    pub command: String,
}

#[derive(Serialize)]
pub struct SubProcOutput {
    pub msg: String,
}
