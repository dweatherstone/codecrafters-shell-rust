use std::path::PathBuf;

use crate::tokenizer::tokenize;

pub struct ParsedCommand {
    pub kind: ShellCommand,
    redirects: Vec<Redirect>,
}

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

#[derive(Debug, Clone)]
pub enum Redirect {
    Stdout(PathBuf),
    Stderr(PathBuf),
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
                    redirects.push(Redirect::Stdout(path));
                    continue;
                }
            }
            "2>" => {
                if i + 1 < tokens.len() {
                    let path = PathBuf::from(tokens.remove(i + 1));
                    tokens.remove(i);
                    redirects.push(Redirect::Stderr(path));
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    redirects
}
