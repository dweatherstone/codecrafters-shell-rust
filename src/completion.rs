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

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // Do we have a registered completer?
        if let Some(result) = self.complete_registered(line, pos)? {
            return Ok(result);
        }

        // Are we completing a command?
        if !line[..pos].contains(' ') {
            return self.complete_command(line, pos);
        }

        // Ok, we're completing a filename
        self.complete_filename(line, pos, ctx)
    }
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

    fn complete_registered(
        &self,
        line: &str,
        pos: usize,
    ) -> rustyline::Result<Option<(usize, Vec<Pair>)>> {
        let completer = {
            let completions = self.completions.borrow();

            first_word(line)
                .and_then(|command| completions.get(command))
                .cloned()
        };

        let Some(completer) = completer else {
            return Ok(None);
        };

        let words: Vec<&str> = line[..pos].split_whitespace().collect();

        let command = words.first().copied().unwrap_or("");
        let current_word = words.last().copied().unwrap_or("");
        let previous_word = words
            .get(words.len().saturating_sub(2))
            .copied()
            .unwrap_or("");

        let output = std::process::Command::new(&completer)
            .env("COMP_LINE", line)
            .env("COMP_POINT", pos.to_string())
            .args([command, current_word, previous_word])
            .output()
            .map_err(rustyline::error::ReadlineError::Io)?;

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

        Ok(Some((start_pos, candidates)))
    }

    fn complete_command(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
        let current_word = &line[..pos];

        let mut matches = Vec::new();

        for cmd in &self.commands {
            if cmd.starts_with(current_word) {
                matches.push(Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                });
            }
        }

        if let Ok(path_var) = std::env::var("PATH") {
            for path in path_var.split(':') {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let filename = entry.file_name().to_string_lossy().to_string();

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

        matches.sort_by(|a, b| a.display.cmp(&b.display));
        matches.dedup_by(|a, b| a.display == b.display);

        if matches.len() == 1 {
            matches[0].replacement = format!("{} ", matches[0].display);
        }

        Ok((0, matches))
    }

    fn complete_filename(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start_pos, candidates) = self.file_completer.complete(line, pos, ctx)?;

        let candidates = candidates
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

        Ok((start_pos, candidates))
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

fn first_word(line: &str) -> Option<&str> {
    line.split_whitespace().next()
}
