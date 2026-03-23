use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Editor};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};

use crate::builtins::{handle_cd, handle_echo, handle_type};
use crate::command::{ParsedCommand, Redirect, RedirectMode, ShellCommand};
use crate::exec::handle_external;

pub mod builtins;
pub mod command;
pub mod exec;
pub mod tokenizer;

fn main() {
    let h = ShellHelper {
        commands: vec![
            "exit".to_string(),
            "echo".to_string(),
            "type".to_string(),
            "cd".to_string(),
            "pwd".to_string(),
        ],
        file_completer: FilenameCompleter::new(),
    };
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(h));

    loop {
        let readline = rl.readline("$ ");
        let input = match readline {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        };

        if let Some(command) = ParsedCommand::parse(input.trim()) {
            let mut stdout = make_stdout_writer(&command);
            let mut stderr = make_stderr_writer(&command);
            match command.kind {
                ShellCommand::Exit => break,
                ShellCommand::Echo(args) => {
                    let _ = handle_echo(&args, &mut stdout);
                }
                ShellCommand::Type(target) => _ = handle_type(&target, &mut stdout),
                ShellCommand::External(ref name, ref args) => {
                    match handle_external(name, args, &command) {
                        Ok(_) => {}
                        Err(_) => _ = writeln!(stderr, "{name}: command not found"),
                    }
                }
                ShellCommand::Pwd => {
                    let _ = writeln!(
                        stdout,
                        "{}",
                        env::current_dir().unwrap_or_default().display()
                    );
                }
                ShellCommand::Cd(absolute_path) => match handle_cd(&absolute_path) {
                    Ok(_) => {}
                    Err(_) => {
                        let _ = writeln!(stderr, "{absolute_path}: No such file or directory");
                    }
                },
            }
        }
    }
}

fn make_stdout_writer(cmd: &ParsedCommand) -> Box<dyn Write> {
    for r in cmd.get_redirects().iter() {
        if let Redirect::Stdout(path, mode) = r {
            let mut open = OpenOptions::new();
            open.create(true);

            match mode {
                RedirectMode::Write => {
                    open.write(true).truncate(true);
                }
                RedirectMode::Append => {
                    open.write(true).append(true);
                }
            }
            let file = open.open(path).unwrap();
            return Box::new(file);
        }
    }
    Box::new(io::stdout())
}

fn make_stderr_writer(cmd: &ParsedCommand) -> Box<dyn Write> {
    for r in cmd.get_redirects().iter() {
        if let Redirect::Stderr(path, mode) = r {
            let mut open = OpenOptions::new();
            open.create(true);

            match mode {
                RedirectMode::Write => {
                    open.write(true).truncate(true);
                }
                RedirectMode::Append => {
                    open.write(true).append(true);
                }
            }
            let file = open.open(path).unwrap();
            return Box::new(file);
        }
    }
    Box::new(io::stderr())
}

#[derive(Helper, Hinter, Highlighter, Validator)]
struct ShellHelper {
    commands: Vec<String>,
    file_completer: FilenameCompleter,
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let folder_part = &line[..pos];

        if !folder_part.contains(' ') {
            // Command completion (first word)
            let mut matches = Vec::new();

            // Only suggest if we are at the start of the line or typing the first word
            for cmd in &self.commands {
                if cmd.starts_with(folder_part) {
                    matches.push(Pair {
                        display: cmd.to_string(),
                        replacement: cmd.to_string(),
                    });
                }
            }

            // Check External Executables in PATH
            if let Ok(path_var) = std::env::var("PATH") {
                // Split PATH by ':' (on Unix) or ';' (on Windows)
                let paths: Vec<&str> = path_var.split(':').collect();

                for path in paths {
                    // Read the directory
                    if let Ok(entries) = std::fs::read_dir(path) {
                        for entry in entries.flatten() {
                            let filename = entry.file_name().to_string_lossy().to_string();
                            // If the filename starts with what the user typed...
                            if filename.starts_with(folder_part) {
                                matches.push(Pair {
                                    display: filename.clone(),
                                    replacement: filename,
                                });
                            }
                        }
                    }
                }
            }

            // Deduplicate matches
            matches.sort_by(|a, b| a.display.cmp(&b.display));
            matches.dedup_by(|a, b| a.display == b.display);

            // Handle trailing space logic
            if matches.len() == 1 {
                matches[0].replacement = format!("{} ", matches[0].display);
            }

            Ok((0, matches))
        } else {
            // Filename completion
            let (start_pos, candidates) = self.file_completer.complete(line, pos, ctx)?;
            let updated_candidates: Vec<Pair> = candidates
                .iter()
                .map(|candidate| Pair {
                    display: candidate.display.clone(),
                    replacement: format!("{} ", candidate.replacement),
                })
                .collect();
            Ok((start_pos, updated_candidates))
        }
    }
}
