#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();

    // Read user's input
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    print!("{input}: command not found");
    io::stdout().flush().unwrap();
}
