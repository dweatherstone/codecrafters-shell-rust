use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Editor};

use crate::completion::ShellHelper;
use crate::executor::ExecutionResult;
use crate::history::History;
use crate::jobs::Jobs;

pub mod ast;
pub mod builtins;
pub mod completion;
pub mod declare;
pub mod executor;
pub mod history;
pub mod jobs;
pub mod lexer;
pub mod parser;

fn main() -> rustyline::Result<()> {
    let completions = Rc::new(RefCell::new(HashMap::new()));
    let h = ShellHelper::new(Rc::clone(&completions));
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(h));

    let mut history = History::default();
    let mut jobs = Jobs::default();
    let mut variables = HashMap::new();

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

        let mut pipeline = match parser::parse(&tokens) {
            Ok(pipeline) => pipeline,
            Err(error) => {
                eprintln!("syntax error: {error:?}");
                continue;
            }
        };

        for command in &mut pipeline.commands {
            for word in &mut command.words {
                *word = expand_variables(word, &variables);
            }
        }

        match executor::execute_pipeline(
            &pipeline,
            &mut history,
            &mut jobs,
            &mut variables,
            &completions,
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

fn expand_variables(word: &str, variables: &HashMap<String, String>) -> String {
    use crate::declare::{is_variable_name_continue, is_variable_name_start};

    let mut chars = word.chars().peekable();
    let mut expanded = String::new();

    while let Some(ch) = chars.next() {
        if ch != '$' {
            expanded.push(ch);
            continue;
        }

        if !chars
            .peek()
            .is_some_and(|&next| is_variable_name_start(next))
        {
            expanded.push('$');
            continue;
        }

        let mut name = String::new();
        while let Some(&next) = chars.peek() {
            if !is_variable_name_continue(next) {
                break;
            }
            name.push(chars.next().expect("character was just peeked"));
        }

        if let Some(value) = variables.get(&name) {
            expanded.push_str(value);
        }
        // An unset variable contributes an empty string.
    }

    expanded
}
