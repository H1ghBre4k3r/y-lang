use std::{error::Error, fmt::Display, iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Eq,
    Let,
    Id(String),
    Num(u64),
    Semicolon,
    Comment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError(String);

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Error for LexError {}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = vec![];

    let mut iterator = input.chars().peekable();

    loop {
        let Some(next) = iterator.next() else {
            break;
        };

        match next {
            '=' => tokens.push(Token::Eq),
            ';' => tokens.push(Token::Semicolon),
            '/' => {
                let token = lex_comment(next, &mut iterator)?;
                tokens.push(token);
            }
            'a'..='z' | 'A'..='Z' => {
                let token = lex_alphabetic(next, &mut iterator);
                tokens.push(token);
            }
            '0'..='9' => {
                let token = lex_numeric(next, &mut iterator)?;
                tokens.push(token);
            }
            _ => continue,
        }
    }

    Ok(tokens)
}

fn lex_comment(current: char, iterator: &mut Peekable<Chars>) -> Result<Token, LexError> {
    assert_eq!(current, '/');

    let Some('/') = iterator.next() else {
        return Err(LexError("Comment without second slash!".into()));
    };

    let mut read = vec![];

    while let Some(next) = iterator.next_if(|item| *item != '\n') {
        read.push(next);
    }

    Ok(Token::Comment(read.iter().collect()))
}

fn lex_alphabetic(current: char, iterator: &mut Peekable<Chars>) -> Token {
    assert!(current.is_alphabetic());
    let mut read = vec![current];

    while let Some(next) = iterator.next_if(|item| item.is_alphabetic()) {
        read.push(next)
    }

    let read = read.iter().collect::<String>();

    match read.as_str() {
        "let" => Token::Let,
        _ => Token::Id(read),
    }
}

fn lex_numeric(current: char, iterator: &mut Peekable<Chars>) -> Result<Token, LexError> {
    assert!(current.is_numeric());
    let mut read = vec![current];

    while let Some(next) = iterator.next_if(|item| item.is_numeric()) {
        read.push(next)
    }

    let read = read.iter().collect::<String>();

    read.parse::<u64>()
        .map(Token::Num)
        .map_err(|_| LexError("failed to parse numeric".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_alphabetic_keyword() {
        let mut iterator = "let".chars().peekable();
        let next = iterator.next().unwrap();

        assert_eq!(Token::Let, lex_alphabetic(next, &mut iterator))
    }

    #[test]
    fn test_lex_alphabetic_id() {
        let mut iterator = "letter".chars().peekable();
        let next = iterator.next().unwrap();

        assert_eq!(
            Token::Id("letter".into()),
            lex_alphabetic(next, &mut iterator)
        )
    }

    #[test]
    fn test_lex_numeric() {
        let mut iterator = "1337".chars().peekable();
        let next = iterator.next().unwrap();

        assert_eq!(Ok(Token::Num(1337)), lex_numeric(next, &mut iterator))
    }

    #[test]
    fn test_lex_comment() {
        let mut iterator = "// some comment".chars().peekable();
        let next = iterator.next().unwrap();

        assert_eq!(
            Ok(Token::Comment(" some comment".into())),
            lex_comment(next, &mut iterator)
        )
    }
}
