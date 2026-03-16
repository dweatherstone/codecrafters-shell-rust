#[allow(unused_imports)]
use std::io::{self, Write};
use std::{collections::VecDeque, env, os::unix::fs::PermissionsExt};

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
                // if !args.is_empty() && ["exit", "echo", "type"].contains(&args[0]) {
                //     println!("{} is a shell builtin", args[0]);
                // } else if !args.is_empty() {
                //     //println!("{}: not found", args[0]);
                //     let key = "PATH";
                //     match env::var_os(key) {
                //         Some(paths) => {
                //             for path in env::split_paths(&paths) {
                //                 let full_path = path.join(args[0]);
                //                 if full_path.is_file() {
                //                     println!("{} is {}", args[0], full_path.display());
                //                     continue 'outer_loop;
                //                 }
                //             }
                //             println!("Part 1 {}: not found", args[0]);
                //         }
                //         None => println!("Part 2 {}: not found", args[0]),
                //     }
                // } else {
                //     continue;
                // }
                if let Some(target) = args.get(0) {
                    let builtins = ["exit", "echo", "type"];

                    if builtins.contains(target) {
                        println!("{} is a shell builtin", target);
                    } else if let Some(path_env) = env::var_os("PATH") {
                        let mut found = false;
                        for path in env::split_paths(&path_env) {
                            let full_path = path.join(target);
                            if full_path.is_file() {
                                if let Ok(metadata) = std::fs::metadata(&full_path) {
                                    let is_executable = metadata.permissions().mode() & 0o111 != 0;
                                    if is_executable {
                                        println!("{} is {}", target, full_path.display());
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if !found {
                            println!("{}: not found", target);
                        }
                    } else {
                        println!("{}: not found", target);
                    }
                }
            }
            _ => println!("{command}: command not found"),
        }
    }
}
