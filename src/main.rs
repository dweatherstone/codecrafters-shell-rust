use std::path::Path;

use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Editor};

use crate::completion::{CompletionRegistry, ShellHelper};
use crate::executor::ExecutionResult;
use crate::history::History;
use crate::jobs::Jobs;

pub mod ast;
pub mod builtins;
pub mod completion;
pub mod executor;
pub mod history;
pub mod jobs;
pub mod lexer;
pub mod parser;

fn main() -> rustyline::Result<()> {
    let h = ShellHelper::default();
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(h));

    let mut history = History::default();
    let mut jobs = Jobs::default();
    let mut completions = CompletionRegistry::default();

    if let Ok(history_path) = std::env::var("HISTFILE") {
        history.load_from_file(Path::new(&history_path))?;
    }

    loop {
        let done_jobs = match jobs.reap() {
            Ok(jobs) => jobs,
            Err(error) => {
                eprintln!("job reap error: {error}");
                Vec::new()
            }
        };
        for job in done_jobs {
            println!(
                "[{}]{}  {:<24} {}",
                job.number,
                job.marker.unwrap_or(' '),
                "Done",
                job.command
            );
        }
        let readline = rl.readline("$ ");
        let input = match readline {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        };

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        rl.add_history_entry(trimmed)?;
        history.add(trimmed.to_string());

        let tokens = match lexer::lex(trimmed) {
            Ok(tokens) => tokens,
            Err(error) => {
                eprintln!("syntax error: {error:?}");
                continue;
            }
        };

        let pipeline = match parser::parse(&tokens) {
            Ok(pipeline) => pipeline,
            Err(error) => {
                eprintln!("syntax error: {error:?}");
                continue;
            }
        };

        match executor::execute_pipeline(
            &pipeline,
            &mut history,
            &mut jobs,
            &mut completions,
            trimmed,
        ) {
            Ok(ExecutionResult::Exit) => break,
            Ok(ExecutionResult::Continue(_status)) => {}
            Err(error) => eprintln!("execution error: {error}"),
        }
    }
    if let Ok(history_path) = std::env::var("HISTFILE") {
        history.write_to_file(Path::new(&history_path))?;
    }
    Ok(())
}
