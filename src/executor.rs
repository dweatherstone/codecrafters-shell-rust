use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    os::{fd::OwnedFd, unix::process::CommandExt},
    process::{Child, Command as ProcessCommand, ExitStatus, Stdio},
};

use nix::{
    sys::wait::{WaitStatus, waitpid},
    unistd::{ForkResult, Pid, dup2_stdin, dup2_stdout, fork, pipe},
};

use crate::{
    ast::{Command, Pipeline, Redirect, RedirectMode},
    builtins::{find_in_path, handle_cd, handle_echo, handle_history, handle_type, is_builtin},
    history::History,
    jobs::Jobs,
};

pub enum ExecutionResult {
    Continue(i32),
    Exit,
}

enum PipelineChild {
    External(Child),
    Builtin(Pid),
}

pub fn execute_pipeline(
    pipeline: &Pipeline,
    history: &mut History,
    jobs: &mut Jobs,
) -> io::Result<ExecutionResult> {
    if pipeline.background {
        execute_background(pipeline, jobs)?;
        return Ok(ExecutionResult::Continue(0));
    }

    if pipeline.commands.len() == 1 {
        return execute_command(&pipeline.commands[0], history);
    }

    let mut children = Vec::with_capacity(pipeline.commands.len());

    // Read end of the previous pipe
    let mut previous_read: Option<OwnedFd> = None;

    for (index, command) in pipeline.commands.iter().enumerate() {
        let program = command
            .program()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty command"))?;

        let is_last = index == pipeline.commands.len() - 1;

        // Create a pipe if this isn't the final command
        let (next_read, next_write) = if !is_last {
            let (read, write) = pipe().map_err(io::Error::other)?;
            (Some(read), Some(write))
        } else {
            (None, None)
        };

        // Builtin
        if is_builtin(program) {
            let pid = spawn_builtin(
                command,
                history,
                previous_read.as_ref(),
                next_write.as_ref(),
            )
            .map_err(io::Error::other)?;

            children.push(PipelineChild::Builtin(pid));

            // The child now has its own copies of these file descriptors.
            // The parent doesn't need them anymore.
            drop(previous_read.take());
            drop(next_write);
        }
        // External command
        else {
            let full_path = find_in_path(program).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("{program}: command not found"),
                )
            })?;

            let mut process = ProcessCommand::new(full_path);

            process.arg0(program).args(command.args());

            // Connect the previous command's output to this command's stdin
            if let Some(reader) = previous_read.take() {
                process.stdin(Stdio::from(reader));
            }

            // Connect this command's stdout to the next command's stdin.
            if let Some(writer) = next_write {
                process.stdout(Stdio::from(writer));
            }

            apply_external_redirects(&mut process, command)?;

            let child = process.spawn()?;

            children.push(PipelineChild::External(child));
        }

        // The read end of this command's output pipe
        // becomes the input to the next command
        previous_read = next_read;
    }

    // Wait for every process in the pipeline
    let mut final_status = 0;

    for (index, child) in children.into_iter().enumerate() {
        let is_final = index == pipeline.commands.len() - 1;

        match child {
            PipelineChild::External(mut child) => {
                let status = child.wait()?;

                if is_final {
                    final_status = status.code().unwrap_or(1);
                }
            }

            PipelineChild::Builtin(pid) => {
                let status = waitpid(pid, None).map_err(io::Error::other)?;

                if is_final {
                    final_status = match status {
                        WaitStatus::Exited(_, code) => code,
                        WaitStatus::Signaled(_, signal, _) => 128 + signal as i32,
                        _ => 1,
                    };
                }
            }
        }
    }

    Ok(ExecutionResult::Continue(final_status))
}

pub fn execute_command(command: &Command, history: &mut History) -> io::Result<ExecutionResult> {
    let mut stdout = make_stdout_writer(command)?;
    let mut stderr = make_stderr_writer(command)?;

    execute_builtin(command, history, &mut stdout, &mut stderr)
}

fn spawn_builtin(
    command: &Command,
    history: &mut History,
    stdin: Option<&OwnedFd>,
    stdout: Option<&OwnedFd>,
) -> nix::Result<nix::unistd::Pid> {
    match unsafe { fork()? } {
        ForkResult::Parent { child } => Ok(child),

        ForkResult::Child => {
            if let Some(stdin) = stdin {
                dup2_stdin(stdin)?;
            }

            if let Some(stdout) = stdout {
                dup2_stdout(stdout)?;
            }

            let mut stdout = io::stdout();
            let mut stderr = io::stderr();

            match execute_builtin(command, history, &mut stdout, &mut stderr) {
                Ok(ExecutionResult::Continue(status)) => {
                    std::process::exit(status);
                }
                Ok(ExecutionResult::Exit) => {
                    std::process::exit(0);
                }
                Err(_) => {
                    std::process::exit(1);
                }
            }
        }
    }
}

fn execute_builtin(
    command: &Command,
    history: &mut History,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<ExecutionResult> {
    let program = match command.program() {
        Some(program) => program,
        None => return Ok(ExecutionResult::Continue(0)),
    };

    match program {
        "exit" => Ok(ExecutionResult::Exit),
        "echo" => {
            handle_echo(command.args(), stdout)?;
            Ok(ExecutionResult::Continue(0))
        }
        "type" => {
            let target = command.args().first().map(String::as_str).unwrap_or("");
            handle_type(target, stdout)?;
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
        "history" => {
            handle_history(command.args(), history, stdout)?;
            Ok(ExecutionResult::Continue(0))
        }
        "jobs" => Ok(ExecutionResult::Continue(0)),
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

fn execute_background(pipeline: &Pipeline, jobs: &mut Jobs) -> io::Result<()> {
    let command = pipeline
        .commands
        .first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty background command"))?;

    let program = command
        .program()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty command"))?;

    let full_path = find_in_path(program).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{program}: command not found"),
        )
    })?;

    let mut process = ProcessCommand::new(full_path);

    process.arg0(program).args(command.args());

    apply_external_redirects(&mut process, command)?;

    let child = process.spawn()?;
    let pid = child.id();

    let job_number = jobs.add(child);

    println!("[{}] {}", job_number, pid);

    Ok(())
}
