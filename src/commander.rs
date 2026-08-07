use std::process::{Command, Stdio};
use std::io;

pub fn run_command(cmd: String, args: Vec<&str>) -> io::Result<(bool, String)> {
    let output = Command::new(cmd.trim())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    if !stdout.is_empty() {
        return Ok((true, stdout));
    }

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok((false, stderr))
}
