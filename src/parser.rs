use std::path::PathBuf;

use crate::{
    ast::{Command, Pipeline, Redirect},
    lexer::Token,
};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    EmptyCommand,
    MissingCommandAfterPipe,
    MissingRedirectTarget,
    UnexpectedToken,
}

pub fn parse(tokens: &[Token]) -> Result<Pipeline, ParseError> {
    let mut commands = Vec::new();
    let mut current = Vec::new();
    let mut background = false;

    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::Pipe => {
                if current.is_empty() {
                    return Err(ParseError::MissingCommandAfterPipe);
                }

                commands.push(parse_command(&current)?);
                current.clear();
            }
            Token::Background => {
                // '&' must be the final token.
                if index != tokens.len() - 1 {
                    return Err(ParseError::UnexpectedToken);
                }
                background = true;
            }
            token => current.push(token.clone()),
        }
    }

    if current.is_empty() {
        return if commands.is_empty() {
            Err(ParseError::EmptyCommand)
        } else {
            Err(ParseError::MissingCommandAfterPipe)
        };
    }

    commands.push(parse_command(&current)?);

    Ok(Pipeline {
        commands,
        background,
    })
}

fn parse_command(tokens: &[Token]) -> Result<Command, ParseError> {
    let mut words = Vec::new();
    let mut redirects = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        match &tokens[index] {
            Token::Word(word) => {
                words.push(word.clone());
                index += 1;
            }
            Token::Redirect { fd, mode } => {
                let target = match tokens.get(index + 1) {
                    Some(Token::Word(path)) => PathBuf::from(path),
                    _ => return Err(ParseError::MissingRedirectTarget),
                };

                redirects.push(Redirect {
                    fd: *fd,
                    target,
                    mode: *mode,
                });

                index += 2;
            }
            Token::Pipe | Token::Background => {
                return Err(ParseError::UnexpectedToken);
            }
        }
    }

    if words.is_empty() {
        return Err(ParseError::EmptyCommand);
    }

    Ok(Command { words, redirects })
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_input(input: &str) -> Result<Pipeline, ParseError> {
        let tokens = lex(input).expect("test input should lex successfully");
        parse(&tokens)
    }

    #[test]
    fn parses_command_words() {
        let pipeline = parse_input("tail -f /tmp/foo").unwrap();

        assert_eq!(
            pipeline,
            Pipeline {
                commands: vec![Command {
                    words: vec!["tail".into(), "-f".into(), "/tmp/foo".into()],
                    redirects: vec![],
                }],
                background: false,
            }
        );
    }

    #[test]
    fn parses_quoted_words() {
        let pipeline = parse_input("echo \"hello world\"").unwrap();

        assert_eq!(pipeline.commands[0].words, vec!["echo", "hello world"]);
    }

    #[test]
    fn preserves_empty_quoted_words() {
        let pipeline = parse_input("echo \"\"").unwrap();

        assert_eq!(pipeline.commands[0].words, vec!["echo", ""]);
    }

    #[test]
    fn parses_multiple_commands_in_a_pipeline() {
        let pipeline = parse_input("cat file.txt | grep hello | wc -l").unwrap();

        assert_eq!(pipeline.commands.len(), 3);
        assert_eq!(pipeline.commands[0].words, vec!["cat", "file.txt"]);
        assert_eq!(pipeline.commands[1].words, vec!["grep", "hello"]);
        assert_eq!(pipeline.commands[2].words, vec!["wc", "-l"]);
    }

    #[test]
    fn parses_stdout_and_stderr_redirects() {
        let pipeline = parse_input("echo hello > output.txt 2>> errors.txt").unwrap();

        assert_eq!(
            pipeline.commands[0].redirects,
            vec![
                Redirect {
                    fd: 1,
                    target: "output.txt".into(),
                    mode: crate::ast::RedirectMode::Write,
                },
                Redirect {
                    fd: 2,
                    target: "errors.txt".into(),
                    mode: crate::ast::RedirectMode::Append,
                },
            ]
        );
        assert_eq!(pipeline.commands[0].words, vec!["echo", "hello"]);
    }

    #[test]
    fn accepts_redirects_before_the_command() {
        let pipeline = parse_input("> output.txt echo hello").unwrap();

        assert_eq!(pipeline.commands[0].words, vec!["echo", "hello"]);
        assert_eq!(pipeline.commands[0].redirects[0].fd, 1);
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(parse_input(""), Err(ParseError::EmptyCommand));
    }

    #[test]
    fn rejects_missing_command_after_pipe() {
        assert_eq!(
            parse_input("echo hello |"),
            Err(ParseError::MissingCommandAfterPipe)
        );
        assert_eq!(
            parse_input("| echo hello"),
            Err(ParseError::MissingCommandAfterPipe)
        );
    }

    #[test]
    fn rejects_missing_redirect_target() {
        assert_eq!(
            parse_input("echo hello >"),
            Err(ParseError::MissingRedirectTarget)
        );
    }

    #[test]
    fn rejects_redirect_followed_by_another_operator() {
        assert_eq!(
            parse_input("echo hello > | cat"),
            Err(ParseError::MissingRedirectTarget)
        );
    }

    #[test]
    fn keeps_a_pipe_inside_quotes_in_one_command() {
        let pipeline = parse_input("echo \"left|right\"").unwrap();

        assert_eq!(pipeline.commands.len(), 1);
        assert_eq!(pipeline.commands[0].words, vec!["echo", "left|right"]);
    }

    #[test]
    fn parse_background_command() {
        let pipeline = parse_input("sleep 500 &").unwrap();
        assert!(pipeline.background);
        assert_eq!(pipeline.commands[0].words, vec!["sleep", "500"]);
    }

    #[test]
    fn parses_foreground_command_without_background() {
        let pipeline = parse_input("sleep 500").unwrap();
        assert!(!pipeline.background);
        assert_eq!(pipeline.commands[0].words, vec!["sleep", "500"]);
    }

    #[test]
    fn rejected_background_token_before_end() {
        assert_eq!(parse_input("sleep & 500"), Err(ParseError::UnexpectedToken));
    }

    #[test]
    fn parses_background_pipeline() {
        let pipeline = parse_input("echo hello | wc &").unwrap();
        assert!(pipeline.background);
        assert_eq!(pipeline.commands.len(), 2);
    }
}
