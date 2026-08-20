use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Editor};

use crate::completion::ShellHelper;
use crate::executor::ExecutionResult;

pub mod ast;
pub mod builtins;
pub mod completion;
pub mod executor;
pub mod lexer;
pub mod parser;

fn main() -> rustyline::Result<()> {
    let h = ShellHelper::default();
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(h));

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

        match executor::execute_pipeline(&pipeline) {
            Ok(ExecutionResult::Exit) => break,
            Ok(ExecutionResult::Continue(_status)) => {}
            Err(error) => eprintln!("execution error: {error}"),
        }
    }
    Ok(())
}
