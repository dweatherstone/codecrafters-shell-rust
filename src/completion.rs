use std::{
    collections::HashMap,
    io::{self, Write},
};

use rustyline::{
    Helper, Highlighter, Hinter, Validator,
    completion::{Completer, FilenameCompleter, Pair},
};

use crate::builtins;

#[derive(Helper, Hinter, Highlighter, Validator)]
pub struct ShellHelper {
    commands: Vec<String>,
    file_completer: FilenameCompleter,
}

impl Default for ShellHelper {
    fn default() -> Self {
        let commands = builtins::command_names().map(|n| n.to_string()).collect();

        ShellHelper {
            commands,
            file_completer: FilenameCompleter::new(),
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

#[derive(Default)]
pub struct CompletionRegistry {
    entries: HashMap<String, String>,
}

impl CompletionRegistry {
    fn add(&mut self, command: String, script: String) {
        self.entries.insert(command, script);
    }

    fn get(&self, command: &str) -> Option<&String> {
        self.entries.get(command)
    }
}

pub fn handle_complete(
    args: &[String],
    completions: &mut CompletionRegistry,
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
            completions.add(command.into(), script.into());
        }
        Some("-p") => {
            let command_str = args.get(1).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "complete: command name not provided",
                )
            })?;
            match completions.get(command_str) {
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
