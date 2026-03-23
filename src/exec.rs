use crate::{
    builtins::find_in_path,
    command::{ParsedCommand, Redirect, RedirectMode},
};
use std::{
    fs::{File, OpenOptions},
    io,
    os::unix::process::CommandExt,
    process::{Command as ProcessCommand, ExitStatus, Stdio},
};

pub fn handle_external(
    command: &str,
    args: &[String],
    parsed: &ParsedCommand,
) -> io::Result<ExitStatus> {
    let full_path = find_in_path(command)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "command not found"))?;
    let mut cmd = ProcessCommand::new(&full_path);

    cmd.arg0(command).args(args);

    for r in parsed.get_redirects().iter() {
        match &r {
            Redirect::Stdout(path, mode) => {
                let file = match mode {
                    RedirectMode::Write => File::create(path)?,
                    RedirectMode::Append => {
                        OpenOptions::new().create(true).append(true).open(path)?
                    }
                };
                cmd.stdout(Stdio::from(file));
            }
            Redirect::Stderr(path, mode) => {
                let file = match mode {
                    RedirectMode::Write => File::create(path)?,
                    RedirectMode::Append => {
                        OpenOptions::new().create(true).append(true).open(path)?
                    }
                };
                cmd.stderr(Stdio::from(file));
            }
        }
    }
    cmd.status()
}
