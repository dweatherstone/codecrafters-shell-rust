use crate::ast::RedirectMode;

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut word_started = false;
    let mut quote_state = QuoteState::Normal;
    let mut input_chars = input.chars().peekable();

    while let Some(ch) = input_chars.next() {
        match quote_state {
            QuoteState::Normal => match ch {
                '\'' => {
                    quote_state = QuoteState::InSingleQuote;
                    word_started = true;
                }
                '"' => {
                    quote_state = QuoteState::InDoubleQuote;
                    word_started = true;
                }
                '\\' => {
                    let next_ch = input_chars.next().ok_or(LexError::TrailingEscape)?;
                    current.push(next_ch);
                    word_started = true;
                }
                '1' | '2' if !word_started && input_chars.peek() == Some(&'>') => {
                    let fd = if ch == '1' { 1 } else { 2 };
                    input_chars.next();

                    let mode = if input_chars.peek() == Some(&'>') {
                        input_chars.next();
                        RedirectMode::Append
                    } else {
                        RedirectMode::Write
                    };

                    tokens.push(Token::Redirect { fd, mode });
                }
                '>' => {
                    let mode = if input_chars.peek() == Some(&'>') {
                        input_chars.next();
                        RedirectMode::Append
                    } else {
                        RedirectMode::Write
                    };

                    flush_word(&mut tokens, &mut current, &mut word_started);

                    tokens.push(Token::Redirect { fd: 1, mode });
                }
                '|' => {
                    flush_word(&mut tokens, &mut current, &mut word_started);
                    tokens.push(Token::Pipe);
                }
                ch if ch.is_whitespace() => {
                    flush_word(&mut tokens, &mut current, &mut word_started);
                }
                _ => {
                    current.push(ch);
                    word_started = true;
                }
            },
            QuoteState::InSingleQuote => match ch {
                '\'' => {
                    quote_state = QuoteState::Normal;
                }
                _ => {
                    current.push(ch);
                }
            },
            QuoteState::InDoubleQuote => match ch {
                '\\' => {
                    let next_ch = input_chars.next().ok_or(LexError::TrailingEscape)?;
                    let escapable_chars = ['"', '\\', '$', '`', '\n'];
                    if !escapable_chars.contains(&next_ch) {
                        current.push('\\');
                    }
                    current.push(next_ch);
                }
                '"' => {
                    quote_state = QuoteState::Normal;
                }
                _ => {
                    current.push(ch);
                }
            },
        }
    }

    match quote_state {
        QuoteState::Normal => {
            flush_word(&mut tokens, &mut current, &mut word_started);
            Ok(tokens)
        }
        QuoteState::InSingleQuote => Err(LexError::UnterminatedSingleQuote),
        QuoteState::InDoubleQuote => Err(LexError::UnterminatedDoubleQuote),
    }
}

fn flush_word(tokens: &mut Vec<Token>, current: &mut String, word_started: &mut bool) {
    if *word_started {
        tokens.push(Token::Word(std::mem::take(current)));
        *word_started = false;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Pipe,
    Redirect { fd: u8, mode: RedirectMode },
}

enum QuoteState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnterminatedSingleQuote,
    UnterminatedDoubleQuote,
    TrailingEscape,
}
