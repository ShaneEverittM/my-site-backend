use std::io::{self, BufRead, BufReader, Write};
use subprocess::{Popen, PopenConfig, PopenError, Redirection};

use crate::types::ProcInfo;

pub fn send_command(command: &str, sub_proc: &Popen, term: &ProcInfo) -> io::Result<String> {
    // Get write handle to subprocess's stdin.
    let mut proc_input = sub_proc.stdin.as_ref().expect("stdin is redirected");

    // Send command.
    let nl_command = command.to_owned() + "\n";
    proc_input.write_all(nl_command.as_bytes())?;

    // Read output resulting from command.
    read_from_proc(sub_proc, term)
}

pub fn read_from_proc(sub_proc: &Popen, term: &ProcInfo) -> io::Result<String> {
    // Get read handle to subprocess's stdout and create a buffered reader.
    let proc_output = sub_proc.stdout.as_ref().expect("stdout is redirected");
    let mut reader = BufReader::new(proc_output);

    // Read until the terminating character. This character is supplied from the config file
    // and is guaranteed to only appear after the subprocess is done printing. In the case of
    // shell style interfaces, this will be the prompt, in other cases it will just be eof.
    let mut buf: Vec<u8> = Vec::new();
    reader.read_until(term.term_char as u8, &mut buf)?;

    // Ignore bad characters.
    let mut output = String::from_utf8_lossy(&buf).to_string();

    // Strip process's prompt in favor of the frontend terminal's prompt.
    output = output[0..(output.len() - (term.term_len + 1))].to_string();

    Ok(output)
}

pub fn init(maybe_sp: &mut Option<Popen>, term: &ProcInfo) -> io::Result<String> {
    // Configuration for the subprocess, must redirect stdin and stdout in order to forward
    // user commands and send output to frontend.
    let config = PopenConfig {
        stderr: Redirection::Merge,
        stdout: Redirection::Pipe,
        stdin: Redirection::Pipe,
        ..Default::default()
    };

    // Create subprocess from backend root relative path.
    let new_sp = Popen::create(&[&term.path], config).map_err(|popen_err| match popen_err {
        PopenError::IoError(e) => e,
        PopenError::LogicError(msg) => io::Error::new(io::ErrorKind::InvalidInput, msg),
        _ => io::Error::new(io::ErrorKind::InvalidInput, "Unrecognized error variant"),
    })?;

    // Read first output from process.
    let output = read_from_proc(&new_sp, term)?;

    // Place new subprocess handle in state.
    *maybe_sp = Some(new_sp);

    Ok(output)
}
