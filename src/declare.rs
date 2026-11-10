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
            variables.insert(variable_name.to_string(), value.to_string());
            Ok(())
        }
        None => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "declare: no flag provided",
        )),
    }
}
