pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote_state = QuoteState::Normal;
    let mut input_chars = input.chars();

    while let Some(ch) = input_chars.next() {
        match quote_state {
            QuoteState::Normal => match ch {
                '\'' => quote_state = QuoteState::InSingleQuote,
                '"' => quote_state = QuoteState::InDoubleQuote,
                '\\' => {
                    let next_ch = input_chars.next().unwrap_or_default();
                    current.push(next_ch);
                }
                ' ' | '\t' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            },
            QuoteState::InSingleQuote => match ch {
                '\'' => quote_state = QuoteState::Normal,
                _ => current.push(ch),
            },
            QuoteState::InDoubleQuote => match ch {
                '\\' => {
                    let next_char = input_chars.next().unwrap_or_default();
                    let escapable_chars = ['"', '\\', '$', '`', '\n'];
                    if !escapable_chars.contains(&next_char) {
                        current.push('\\');
                    }
                    current.push(next_char);
                }
                '"' => quote_state = QuoteState::Normal,
                _ => current.push(ch),
            },
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

enum QuoteState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
}
