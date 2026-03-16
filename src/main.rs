use std::collections::VecDeque;
#[allow(unused_imports)]
use std::io::{self, Write};

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
                    println!("{}: not found", args[0]);
                } else {
                    continue;
                }
            }
            _ => println!("{command}: command not found"),
        }
    }
}
