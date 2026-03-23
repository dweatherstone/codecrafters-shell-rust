#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::{env, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Read user's input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if let Some(command) = ShellCommand::parse(input.trim()) {
            match command {
                ShellCommand::Exit => break,
                ShellCommand::Echo(args) => println!("{}", args.join(" ")),
                ShellCommand::Type(target) => handle_type(&target),
                ShellCommand::External(name, args) => handle_external(&name, &args),
                ShellCommand::Pwd => {
                    println!("{}", env::current_dir().unwrap_or_default().display())
                }
                ShellCommand::Cd(absolute_path) => handle_cd(&absolute_path),
            }
        }
    }
}

fn handle_type(target: &str) {
    let builtins = ["exit", "echo", "type", "pwd"];

    if builtins.contains(&target) {
        println!("{} is a shell builtin", target);
    } else if let Some(path) = find_in_path(target) {
        println!("{} is {}", target, path.display());
    } else {
        println!("{}: not found", target);
    }
}

fn handle_external(command: &str, args: &[String]) {
    if let Some(full_path) = find_in_path(command) {
        let _status = Command::new(&full_path)
            .arg0(command)
            .args(args)
            .status()
            .expect("failed to execute process");
    } else {
        println!("{command}: command not found")
    }
}

fn find_in_path(command: &str) -> Option<PathBuf> {
    let path_env = env::var_os("PATH")?;

    for path in env::split_paths(&path_env) {
        let full_path = path.join(command);
        if full_path.is_file()
            && let Ok(metadata) = std::fs::metadata(&full_path)
            && metadata.permissions().mode() & 0o111 != 0
        {
            // Check if any of the "execute" bits (user, group, or other) are set
            // 0o111 is octal for --x--x--x
            return Some(full_path);
        }
    }
    None
}

fn handle_cd(given_path: &str) {
    let raw_path = given_path.replace("~", std::env::var("HOME").unwrap().as_str());
    let path = Path::new(&raw_path);
    if !path.exists() {
        println!("{given_path}: No such file or directory");
    }
    let _ = env::set_current_dir(path);
}

enum ShellCommand {
    Exit,
    Echo(Vec<String>),
    Type(String),
    External(String, Vec<String>),
    Pwd,
    Cd(String),
}

impl ShellCommand {
    fn parse(input: &str) -> Option<Self> {
        let tokens = tokenize(input);
        let mut parts = tokens.into_iter();
        let cmd = parts.next()?;
        let args: Vec<String> = parts.collect();

        match cmd.as_str() {
            "exit" => Some(ShellCommand::Exit),
            "echo" => Some(ShellCommand::Echo(args)),
            "type" => Some(ShellCommand::Type(
                args.first().cloned().unwrap_or_default(),
            )),
            "pwd" => Some(ShellCommand::Pwd),
            "cd" => Some(ShellCommand::Cd(args.first().cloned().unwrap_or_default())),
            _ => Some(ShellCommand::External(cmd.to_string(), args)),
        }
    }
}

enum QuoteState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote_state = QuoteState::Normal;

    for ch in input.chars() {
        match quote_state {
            QuoteState::Normal => match ch {
                '\'' => quote_state = QuoteState::InSingleQuote,
                '"' => quote_state = QuoteState::InDoubleQuote,
                ' ' | '\t' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            },
            QuoteState::InSingleQuote => match ch {
                '\'' => quote_state = QuoteState::Normal,
                _ => current.push(ch),
            },
            QuoteState::InDoubleQuote => match ch {
                '"' => quote_state = QuoteState::Normal,
                _ => current.push(ch),
            },
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}
