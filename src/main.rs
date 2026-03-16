#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::{
    collections::VecDeque, env, os::unix::fs::PermissionsExt, path::PathBuf, process::Command,
};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Read user's input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        let mut args = input.split_whitespace().collect::<VecDeque<_>>();
        if args.is_empty() {
            continue;
        }
        let command = args.pop_front().unwrap().trim();

        match command {
            "exit" => break,
            "echo" => println!("{}", args.make_contiguous().join(" ")),
            "type" => {
                if let Some(target) = args.get(0) {
                    let builtins = ["exit", "echo", "type"];

                    if builtins.contains(target) {
                        println!("{} is a shell builtin", target);
                    } else if let Some(path) = find_in_path(*target) {
                        println!("{} is {}", target, path.display());
                    } else {
                        println!("{}: not found", target);
                    }
                }
            }
            _ => {
                if let Some(full_path) = find_in_path(command) {
                    let _status = Command::new(&full_path)
                        .arg0(command)
                        .args(&args)
                        .status()
                        .expect("failed to execute process");
                } else {
                    println!("{command}: command not found")
                }
            }
        }
    }
}

fn find_in_path(command: &str) -> Option<PathBuf> {
    let path_env = env::var_os("PATH")?;

    for path in env::split_paths(&path_env) {
        let full_path = path.join(command);
        if full_path.is_file() {
            if let Ok(metadata) = std::fs::metadata(&full_path) {
                // Check if any of the "execute" bits (user, group, or other) are set
                // 0o111 is octal for --x--x--x
                if metadata.permissions().mode() & 0o111 != 0 {
                    return Some(full_path);
                }
            }
        }
    }
    None
}
