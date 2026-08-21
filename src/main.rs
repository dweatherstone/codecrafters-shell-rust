use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Editor};

use crate::completion::ShellHelper;
use crate::executor::ExecutionResult;
use crate::history::History;

pub mod ast;
pub mod builtins;
pub mod completion;
pub mod executor;
pub mod history;
pub mod lexer;
pub mod parser;

fn main() -> rustyline::Result<()> {
    let h = ShellHelper::default();
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(h));

    let mut history = History::default();

    loop {
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

        match executor::execute_pipeline(&pipeline, &history) {
            Ok(ExecutionResult::Exit) => break,
            Ok(ExecutionResult::Continue(_status)) => {}
            Err(error) => eprintln!("execution error: {error}"),
        }
    }
    Ok(())
}
