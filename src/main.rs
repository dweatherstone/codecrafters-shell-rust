#[allow(unused_imports)]
use std::io::{self, Write};
use std::{collections::VecDeque, env};

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
            "echo" => println!("{}", args.iter().map(|&s| s).collect::<Vec<_>>().join(" ")),
            "type" => {
                if !args.is_empty() && ["exit", "echo", "type"].contains(&args[0]) {
                    println!("{} is a shell builtin", args[0]);
                } else if !args.is_empty() {
                    //println!("{}: not found", args[0]);
                    let key = "PATH";
                    match env::var_os(key) {
                        Some(paths) => {
                            for path in env::split_paths(&paths) {
                                let full_path = path.join(args[0]);
                                if full_path.is_file() {
                                    println!("{} is {}", args[0], full_path.to_str().unwrap());
                                    break;
                                }
                            }
                            println!("{}: not found", args[0]);
                        }
                        None => println!("{}: not found", args[0]),
                    }
                } else {
                    continue;
                }
            }
            _ => println!("{command}: command not found"),
        }
    }
}
