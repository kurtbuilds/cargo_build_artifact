use anyhow::Result as AnyResult;
use std::env::{args_os, var};
use std::io;
use std::io::{BufRead, BufReader};
use std::process::{Command};

fn main() -> AnyResult<()> {
    let args = args_os().skip(2);

    let cargo_command = var("CARGO_BUILD_ARTIFACT_COMMAND").unwrap_or_else(|_| "build".to_string());
    let (reader, writer) = os_pipe::pipe()?;

    let mut child = Command::new("cargo")
        .arg(&cargo_command)
        .arg("--message-format=json-diagnostic-rendered-ansi")
        .arg("--color=always")
        .args(args)
        .stderr(writer.try_clone()?)
        .stdout(writer)
        .spawn()?;

    let buf = BufReader::new(reader);
    let mut bin = None;
    for line in buf.lines() {
        let line = line?;
        if line.starts_with('{') {
            let data: serde_json::Value = serde_json::from_str(&line)?;
            if let Some(exec) = data["executable"].as_str() {
                bin = Some(exec.to_string());
            }
            let Some(message) = data["message"]["rendered"].as_str() else {
                continue;
            };
            eprint!("{}", message);
        } else {
            eprintln!("{}", line);
        }
    }
    child.wait()?;
    let bin = bin.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No executable found"))?;
    println!("{}", bin);
    Ok(())
}