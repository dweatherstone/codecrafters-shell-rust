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

        if input == "exit" {
            break;
        } else if input.starts_with("echo") {
            println!("{}", &input[5..]);
        }

        println!("{input}: command not found");
    }
}
