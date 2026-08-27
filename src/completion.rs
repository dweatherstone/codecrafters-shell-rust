use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Write},
    rc::Rc,
};

use rustyline::{
    Helper, Highlighter, Hinter, Validator,
    completion::{Completer, FilenameCompleter, Pair},
};

use crate::builtins;

pub type CompletionRegistry = Rc<RefCell<HashMap<String, String>>>;

#[derive(Helper, Hinter, Highlighter, Validator)]
pub struct ShellHelper {
    commands: Vec<String>,
    file_completer: FilenameCompleter,
    completions: CompletionRegistry,
}
impl ShellHelper {
    pub fn new(completions: CompletionRegistry) -> Self {
        let commands = builtins::command_names().map(|n| n.to_string()).collect();

        ShellHelper {
            commands,
            file_completer: FilenameCompleter::new(),
            completions,
        }
    }
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // Do we have a registered completer?
        let completions = self.completions.borrow();
        if let Some(command) = command_name(line)
            && let Some(completer) = completions.get(command)
        {
            let words: Vec<&str> = line[..pos].split_whitespace().collect();
            let command = words.first().copied().unwrap_or("");
            let current_word = words.last().copied().unwrap_or("");
            let previous_word = words
                .get(words.len().saturating_sub(2))
                .copied()
                .unwrap_or("");
            let args = [command, current_word, previous_word];
            let output = std::process::Command::new(completer)
                .env("COMP_LINE", line)
                .env("COMP_POINT", pos.to_string())
                .args(args)
                .output()
                .map_err(|_| {
                    rustyline::error::ReadlineError::Io(io::Error::other("failed to run completer"))
                })?;
            // If completer script does not print anything, then we have no candidates
            if output.stdout.is_empty() {
                // Print bell sound
                println!("\x07");
                return Ok((pos, Vec::new()));
            }

            let mut candidates = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|candidate| Pair {
                    display: candidate.to_string(),
                    replacement: format!("{} ", candidate),
                })
                .collect::<Vec<_>>();
            candidates.sort_by(|a, b| a.display.cmp(&b.display));

            let start_pos = line[..pos]
                .rfind(char::is_whitespace)
                .map(|index| index + 1)
                .unwrap_or(0);
            return Ok((start_pos, candidates));
        }
        let current_word = &line[..pos];

        if !current_word.contains(' ') {
            // Command completion (first word)
            let mut matches = Vec::new();

            // Only suggest if we are at the start of the line or typing the first word
            for cmd in &self.commands {
                if cmd.starts_with(current_word) {
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
                            if filename.starts_with(current_word) {
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
                .into_iter()
                .map(|candidate| {
                    if candidate.replacement.ends_with('/') {
                        Pair {
                            replacement: candidate.replacement,
                            display: format!("{}/", candidate.display),
                        }
                    } else {
                        Pair {
                            replacement: format!("{} ", candidate.replacement),
                            display: candidate.display,
                        }
                    }
                })
                .collect();
            Ok((start_pos, updated_candidates))
        }
    }
}

pub fn handle_complete(
    args: &[String],
    completions: &CompletionRegistry,
    out: &mut dyn Write,
) -> io::Result<()> {
    match args.first().map(String::as_str) {
        Some("-C") => {
            let script = args.get(1).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "complete: script path not provided",
                )
            })?;
            let command = args.get(2).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "complete: command not provided",
                )
            })?;
            completions
                .borrow_mut()
                .insert(command.into(), script.into());
        }
        Some("-p") => {
            let command_str = args.get(1).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "complete: command name not provided",
                )
            })?;
            match completions.borrow().get(command_str) {
                Some(script) => {
                    writeln!(out, "complete -C '{}' {}", script, command_str)?;
                }
                None => {
                    writeln!(
                        out,
                        "complete: {}: no completion specification",
                        command_str
                    )?;
                }
            }
        }
        Some(x) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("complete: unknown argument: {}", x),
            ));
        }
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "complete command requires arguments",
            ));
        }
    }

    Ok(())
}

fn command_name(line: &str) -> Option<&str> {
    line.split_whitespace().next()
}
