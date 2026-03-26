use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::tokenizer::tokenize;

pub struct Pipeline {
    pub commands: Vec<ParsedCommand>,
}

impl Pipeline {
    pub fn parse(input: &str) -> Option<Self> {
        // Check if there is even a pipe in the raw input to save time
        if !input.contains('|') {
            // If no pipe, it's just a pipeline with one command
            return ParsedCommand::parse(input).map(|cmd| Pipeline {
                commands: vec![cmd],
            });
        }

        // Split by the pipe character
        // but being careful about quotes.
        let segments: Vec<&str> = input.split('|').collect();
        let mut commands = Vec::new();

        for segment in segments {
            if let Some(cmd) = ParsedCommand::parse(segment.trim()) {
                commands.push(cmd);
            } else {
                // If any part of the pipe is invalid, the whole thing fails
                return None;
            }
        }

        Some(Pipeline { commands })
    }

    pub fn run(&self) {
        // For now, we just work on 2 commands
        if self.commands.len() != 2 {
            return;
        }

        let cmd1 = &self.commands[0];
        let cmd2 = &self.commands[1];

        // Prepare command 1
        if let (ShellCommand::External(program1, args1), ShellCommand::External(program2, args2)) =
            (&cmd1.kind, &cmd2.kind)
        {
            let mut child1 = Command::new(program1)
                .args(args1)
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to start first command");

            // Capture the output of Command 1
            // We MUST .take() it so the child no longer "owns" the handle
            let pipe_reader = child1.stdout.take().expect("failed to open stdout pipe");

            // Prepare and spawn Command 2
            let mut child2 = Command::new(program2)
                .args(args2)
                .stdin(Stdio::from(pipe_reader))
                .spawn()
                .expect("failed to start second command");

            // Wait fo both to finish
            let _ = child1.wait();
            let _ = child2.wait();
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParsedCommand {
    pub kind: ShellCommand,
    redirects: Vec<Redirect>,
}

#[derive(Debug, PartialEq)]
pub enum ShellCommand {
    Exit,
    Echo(Vec<String>),
    Type(String),
    External(String, Vec<String>),
    Pwd,
    Cd(String),
}

impl ParsedCommand {
    pub fn parse(input: &str) -> Option<Self> {
        let mut tokens = tokenize(input);
        let redirects = extract_redirects(&mut tokens);
        let mut parts = tokens.into_iter();
        let cmd = parts.next()?;
        let args: Vec<String> = parts.collect();

        let kind = match cmd.as_str() {
            "exit" => ShellCommand::Exit,
            "echo" => ShellCommand::Echo(args),
            "type" => ShellCommand::Type(args.first().cloned().unwrap_or_default()),
            "pwd" => ShellCommand::Pwd,
            "cd" => ShellCommand::Cd(args.first().cloned().unwrap_or_default()),
            _ => ShellCommand::External(cmd.to_string(), args),
        };

        Some(Self { kind, redirects })
    }

    pub fn get_redirects(&self) -> Vec<Redirect> {
        self.redirects.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Redirect {
    Stdout(PathBuf, RedirectMode),
    Stderr(PathBuf, RedirectMode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectMode {
    Write,  // >
    Append, // >>
}

fn extract_redirects(tokens: &mut Vec<String>) -> Vec<Redirect> {
    let mut redirects = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i].as_str() {
            ">" | "1>" => {
                if i + 1 < tokens.len() {
                    let path = PathBuf::from(tokens.remove(i + 1));
                    tokens.remove(i);
                    redirects.push(Redirect::Stdout(path, RedirectMode::Write));
                    continue;
                }
            }
            ">>" | "1>>" => {
                if i + 1 < tokens.len() {
                    let path = PathBuf::from(tokens.remove(i + 1));
                    tokens.remove(i);
                    redirects.push(Redirect::Stdout(path, RedirectMode::Append));
                    continue;
                }
            }
            "2>" => {
                if i + 1 < tokens.len() {
                    let path = PathBuf::from(tokens.remove(i + 1));
                    tokens.remove(i);
                    redirects.push(Redirect::Stderr(path, RedirectMode::Write));
                    continue;
                }
            }
            "2>>" => {
                if i + 1 < tokens.len() {
                    let path = PathBuf::from(tokens.remove(i + 1));
                    tokens.remove(i);
                    redirects.push(Redirect::Stderr(path, RedirectMode::Append));
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    redirects
}

#[cfg(test)]
mod command_tests {
    use super::*;

    #[test]
    fn parse_tail_f() {
        let input = "tail -f /tmp/foo";
        let cmd = ParsedCommand::parse(input);
        let expected = Some(ParsedCommand {
            kind: ShellCommand::External(
                String::from("tail"),
                vec![String::from("-f"), String::from("/tmp/foo")],
            ),
            redirects: Vec::new(),
        });
        assert_eq!(cmd, expected);
    }
}
