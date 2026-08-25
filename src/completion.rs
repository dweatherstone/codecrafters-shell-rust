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
