use std::{collections::HashMap, iter::Peekable, str::Chars};

pub(crate) fn is_variable_name_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

pub(crate) fn is_variable_name_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn read_variable_name(chars: &mut Peekable<Chars<'_>>) -> Option<String> {
    if !chars.peek().is_some_and(|&ch| is_variable_name_start(ch)) {
        return None;
    }

    let mut name = String::new();
    while chars
        .peek()
        .is_some_and(|&ch| is_variable_name_continue(ch))
    {
        name.push(chars.next().expect("character was just peeked"));
    }
    Some(name)
}

pub(crate) fn expand_word(word: &str, values: &HashMap<String, String>) -> String {
    let mut chars = word.chars().peekable();
    let mut expanded = String::new();

    while let Some(ch) = chars.next() {
        if ch != '$' {
            expanded.push(ch);
            continue;
        }

        let braced = chars.peek() == Some(&'{');
        if braced {
            chars.next(); // consume '{'
        }

        let Some(name) = read_variable_name(&mut chars) else {
            expanded.push('$');
            if braced {
                expanded.push('{');
            }
            continue;
        };

        if braced {
            if chars.peek() != Some(&'}') {
                // Keep malformed "${NAME" text literal.
                expanded.push_str("${");
                expanded.push_str(&name);
                continue;
            }
            chars.next(); // consume '}'
        }

        if let Some(value) = values.get(&name) {
            expanded.push_str(value);
        }
    }
    expanded
}
