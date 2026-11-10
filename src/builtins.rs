use std::{
    env,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub fn handle_echo(args: &[String], out: &mut dyn Write) -> io::Result<()> {
    writeln!(out, "{}", args.join(" "))
}

pub fn handle_type(target: &str, out: &mut dyn Write) -> io::Result<()> {
    if is_builtin(target) {
        writeln!(out, "{} is a shell builtin", target)
    } else if let Some(path) = find_in_path(target) {
        writeln!(out, "{} is {}", target, path.display())
    } else {
        writeln!(out, "{}: not found", target)
    }
}

pub fn find_in_path(command: &str) -> Option<PathBuf> {
    let path_env = env::var_os("PATH")?;

    for path in env::split_paths(&path_env) {
        let full_path = path.join(command);
        // Check if any of the "execute" bits (user, group, or other) are set
        // 0o111 is octal for --x--x--x
        if full_path.is_file()
            && let Ok(metadata) = std::fs::metadata(&full_path)
            && metadata.permissions().mode() & 0o111 != 0
        {
            return Some(full_path);
        }
    }
    None
}

pub fn handle_cd(given_path: &str) -> io::Result<()> {
    let raw_path = given_path.replace("~", std::env::var("HOME").unwrap().as_str());
    let path = Path::new(&raw_path);
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No such file or directory",
        ));
    }
    env::set_current_dir(path)
}

pub fn command_names() -> impl Iterator<Item = &'static str> {
    [
        "exit", "echo", "type", "cd", "pwd", "history", "jobs", "complete", "declare",
    ]
    .into_iter()
}

pub fn is_builtin(program: &str) -> bool {
    command_names().any(|name| name == program)
}
