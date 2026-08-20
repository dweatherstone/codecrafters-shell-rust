use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    os::unix::process::CommandExt,
    process::{Command as ProcessCommand, ExitStatus, Stdio},
};

use crate::{
    ast::{Command, Pipeline, Redirect, RedirectMode},
    builtins::{find_in_path, handle_cd, handle_echo, handle_type},
};

pub enum ExecutionResult {
    Continue(i32),
    Exit,
}

pub fn execute_pipeline(pipeline: &Pipeline) -> io::Result<ExecutionResult> {
    if pipeline.commands.len() == 1 {
        return execute_command(&pipeline.commands[0]);
    }

    let mut children = Vec::with_capacity(pipeline.commands.len());
    let mut previous_stdout = None;

    for (index, command) in pipeline.commands.iter().enumerate() {
        let program = command
            .program()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty command"))?;
        let full_path = find_in_path(program)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "command not found"))?;
        let mut process = ProcessCommand::new(full_path);

        process.arg0(program).args(command.args());

        if let Some(stdout) = previous_stdout.take() {
            process.stdin(Stdio::from(stdout));
        }

        if index < pipeline.commands.len() - 1 {
            process.stdout(Stdio::piped());
        }

        apply_external_redirects(&mut process, command)?;
        children.push(process.spawn()?);

        if index < pipeline.commands.len() - 1 {
            previous_stdout = children.last_mut().and_then(|child| child.stdout.take());
        }
    }

    let mut final_status = 0;
    for (index, mut child) in children.into_iter().enumerate() {
        let status = child.wait()?;
        if index == pipeline.commands.len() - 1 {
            final_status = status.code().unwrap_or(1);
        }
    }

    Ok(ExecutionResult::Continue(final_status))
}

pub fn execute_command(command: &Command) -> io::Result<ExecutionResult> {
    let program = match command.program() {
        Some(program) => program,
        None => return Ok(ExecutionResult::Continue(0)),
    };

    let mut stdout = make_stdout_writer(command)?;
    let mut stderr = make_stderr_writer(command)?;

    match program {
        "exit" => Ok(ExecutionResult::Exit),
        "echo" => {
            handle_echo(command.args(), &mut stdout)?;
            Ok(ExecutionResult::Continue(0))
        }
        "type" => {
            let target = command.args().first().map(String::as_str).unwrap_or("");
            handle_type(target, &mut stdout)?;
            Ok(ExecutionResult::Continue(0))
        }
        "pwd" => {
            writeln!(stdout, "{}", std::env::current_dir()?.display())?;
            Ok(ExecutionResult::Continue(0))
        }
        "cd" => {
            let path = command.args().first().map(String::as_str).unwrap_or("");
            match handle_cd(path) {
                Ok(()) => Ok(ExecutionResult::Continue(0)),
                Err(error) => {
                    writeln!(stderr, "cd: {path}: {error}")?;
                    Ok(ExecutionResult::Continue(1))
                }
            }
        }
        _ => match execute_external(command) {
            Ok(status) => Ok(ExecutionResult::Continue(status.code().unwrap_or(1))),
            Err(error) => {
                writeln!(stderr, "{program}: {error}")?;
                Ok(ExecutionResult::Continue(127))
            }
        },
    }
}

fn make_stdout_writer(command: &Command) -> io::Result<Box<dyn Write>> {
    match command.redirects.iter().find(|redirect| redirect.fd == 1) {
        Some(redirect) => {
            let file = open_redirect_file(redirect)?;
            Ok(Box::new(file))
        }
        None => Ok(Box::new(io::stdout())),
    }
}

fn make_stderr_writer(command: &Command) -> io::Result<Box<dyn Write>> {
    match command.redirects.iter().find(|redirect| redirect.fd == 2) {
        Some(redirect) => {
            let file = open_redirect_file(redirect)?;
            Ok(Box::new(file))
        }
        None => Ok(Box::new(io::stderr())),
    }
}

fn execute_external(command: &Command) -> io::Result<ExitStatus> {
    let program = command
        .program()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty command"))?;
    let full_path = find_in_path(program)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "command not found"))?;
    let mut process = ProcessCommand::new(&full_path);

    process.arg0(program).args(command.args());

    apply_external_redirects(&mut process, command)?;
    process.status()
}

fn apply_external_redirects(process: &mut ProcessCommand, command: &Command) -> io::Result<()> {
    for redirect in &command.redirects {
        let file = open_redirect_file(redirect)?;

        match redirect.fd {
            1 => {
                process.stdout(Stdio::from(file));
            }
            2 => {
                process.stderr(Stdio::from(file));
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unsupported file descriptor",
                ));
            }
        }
    }
    Ok(())
}

fn open_redirect_file(redirect: &Redirect) -> io::Result<File> {
    match redirect.mode {
        RedirectMode::Write => File::create(&redirect.target),
        RedirectMode::Append => OpenOptions::new()
            .create(true)
            .append(true)
            .open(&redirect.target),
    }
}
