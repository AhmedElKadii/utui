use std::process::{Command, Stdio};
use std::io::{self, Write};

pub fn run_command(cmd: String, args: Vec<&str>) -> String {
    let output = Command::new(cmd.trim())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run command...");

    let stdout = String::from_utf8(output.stdout).unwrap();

    return stdout;
}
