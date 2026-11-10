use std::{
    collections::HashMap,
    io::{self, Write},
};

pub fn handle_declare(
    args: &[String],
    variables: &mut HashMap<String, String>,
    out: &mut dyn Write,
) -> io::Result<()> {
    match args.first().map(String::as_str) {
        Some("-p") => {
            let variable_name = args.get(1).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "declare: variable not provided",
                )
            })?;
            match variables.get(variable_name) {
                Some(value) => writeln!(out, "declare -- {}=\"{}\"", variable_name, value),
                None => writeln!(out, "declare: {}: not found", variable_name),
            }
        }
        Some(arg) => {
            let (variable_name, value) = arg.split_once("=").ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "declare: new variable must have value",
                )
            })?;
            if !is_variable_name_valid(variable_name) {
                return writeln!(out, "declare: `{}': not a valid identifier", arg);
            }
            variables.insert(variable_name.to_string(), value.to_string());
            Ok(())
        }
        None => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "declare: no flag provided",
        )),
    }
}

pub(crate) fn is_variable_name_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

pub(crate) fn is_variable_name_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_variable_name_valid(name: &str) -> bool {
    let mut chars = name.chars();

    match chars.next() {
        Some(first) if is_variable_name_start(first) => {}
        _ => return false,
    }

    chars.all(is_variable_name_continue)
}
