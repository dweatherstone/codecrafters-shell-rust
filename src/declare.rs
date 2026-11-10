use std::io::{self, Write};

pub fn handle_declare(args: &[String], out: &mut dyn Write) -> io::Result<()> {
    match args.first().map(String::as_str) {
        Some("-p") => {
            let variable_name = args.get(1).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "declare: variable not provided",
                )
            })?;
            writeln!(out, "declare: {}: not found", variable_name)
        }
        Some(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "declare: unknown flag",
        )),
        None => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "declare: no flag provided",
        )),
    }
}
