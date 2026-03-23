use crate::{builtins::find_in_path, command::ParsedCommand, command::Redirect};
use std::{
    fs::File,
    os::unix::process::CommandExt,
    process::{Command as ProcessCommand, Stdio},
};

pub fn handle_external(command: &str, args: &[String], parsed: &ParsedCommand) {
    if let Some(full_path) = find_in_path(command) {
        let mut cmd = ProcessCommand::new(&full_path);

        cmd.arg0(command).args(args);

        for r in parsed.get_redirects().iter() {
            match &r {
                Redirect::Stdout(path) => {
                    let file = File::create(path).unwrap();
                    cmd.stdout(Stdio::from(file));
                }
                Redirect::_Stderr(path) => {
                    let file = File::create(path).unwrap();
                    cmd.stderr(Stdio::from(file));
                }
            }
        }

        let _ = cmd.status();
    } else {
        eprintln!("{command}: command not found")
    }
}
