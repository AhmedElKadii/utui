use std::process::{Command, Stdio};
use std::io::{self, Write};

pub fn run_command(cmd: String, args: Vec<&str>) -> Option<String> {
    let output = Command::new(cmd.trim())
        .args(args)
        .stdout(Stdio::piped())
        .output()
        .expect("failed to run command...");

    let stdout = String::from_utf8(output.stdout).unwrap();

    if stdout.len() > 0 {
        return Some(stdout);
    }
    else {
        return None;
    }

}
