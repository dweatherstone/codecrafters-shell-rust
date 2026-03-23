use std::env;
use std::fs::File;
use std::io::{self, Write};

use crate::builtins::{handle_cd, handle_echo, handle_type};
use crate::command::{ParsedCommand, Redirect, ShellCommand};
use crate::exec::handle_external;

pub mod builtins;
pub mod command;
pub mod exec;
pub mod tokenizer;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Read user's input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if let Some(command) = ParsedCommand::parse(input.trim()) {
            let mut stdout = make_stdout_writer(&command);
            match command.kind {
                ShellCommand::Exit => break,
                ShellCommand::Echo(args) => {
                    let _ = handle_echo(&args, &mut stdout);
                }
                ShellCommand::Type(target) => _ = handle_type(&target, &mut stdout),
                ShellCommand::External(ref name, ref args) => handle_external(name, args, &command),
                ShellCommand::Pwd => {
                    let _ = writeln!(
                        stdout,
                        "{}",
                        env::current_dir().unwrap_or_default().display()
                    );
                }
                ShellCommand::Cd(absolute_path) => handle_cd(&absolute_path),
            }
        }
    }
}

fn make_stdout_writer(cmd: &ParsedCommand) -> Box<dyn Write> {
    for r in cmd.get_redirects().iter() {
        if let Redirect::Stdout(path) = r {
            return Box::new(File::create(path).unwrap());
        }
    }
    Box::new(io::stdout())
}

fn _make_stderr_writer(cmd: &ParsedCommand) -> Box<dyn Write> {
    for r in cmd.get_redirects().iter() {
        if let Redirect::_Stderr(path) = r {
            return Box::new(File::create(path).unwrap());
        }
    }
    Box::new(io::stderr())
}
