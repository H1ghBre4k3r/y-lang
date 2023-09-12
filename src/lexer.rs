use std::{error::Error, fmt::Display, iter::Peekable, str::Chars};

type Position = (usize, usize);

#[derive(Debug, Clone)]
pub enum Token {
    Eq { position: Position },
    Let { position: Position },
    Id { value: String, position: Position },
    Num { value: u64, position: Position },
    Semicolon { position: Position },
    Comment { value: String, position: Position },
    Plus { position: Position },
    Times { position: Position },
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        use Token::*;
        matches!(
            (self, other),
            (Eq { .. }, Eq { .. })
                | (Let { .. }, Let { .. })
                | (Id { .. }, Id { .. })
                | (Num { .. }, Num { .. })
                | (Semicolon { .. }, Semicolon { .. })
                | (Comment { .. }, Comment { .. })
                | (Plus { .. }, Plus { .. })
                | (Times { .. }, Times { .. })
        )
    }
}

impl Eq for Token {}

impl Token {
    pub fn position(&self) -> Position {
        match self {
            Token::Eq { position } => *position,
            Token::Let { position } => *position,
            Token::Id { position, .. } => *position,
            Token::Num { position, .. } => *position,
            Token::Semicolon { position } => *position,
            Token::Comment { position, .. } => *position,
            Token::Plus { position } => *position,
            Token::Times { position } => *position,
        }
    }
}

pub struct Tokens {
    tokens: Vec<Token>,
    index: usize,
}

impl Tokens {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.index < self.tokens.len() {
            let item = self.tokens.get(self.index).cloned();
            self.index += 1;
            return item;
        }

        None
    }

    pub fn peek(&mut self) -> Option<Token> {
        return self.tokens.get(self.index).cloned();
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

impl From<Vec<Token>> for Tokens {
    fn from(value: Vec<Token>) -> Self {
        Self::new(value)
    }
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

    let mut line = 1;
    let mut col = 1;

    while let Some(next) = iterator.peek() {
        match next {
            '=' => {
                tokens.push(Token::Eq {
                    position: (line, col),
                });
                iterator.next();
            }
            ';' => {
                tokens.push(Token::Semicolon {
                    position: (line, col),
                });
                iterator.next();
            }
            '/' => {
                let token = lex_comment(&mut iterator, &mut line, &mut col)?;
                tokens.push(token);
            }
            '*' => {
                tokens.push(Token::Times {
                    position: (line, col),
                });
                iterator.next();
            }
            '+' => {
                tokens.push(Token::Plus {
                    position: (line, col),
                });
                iterator.next();
            }
            'a'..='z' | 'A'..='Z' => {
                let token = lex_alphabetic(&mut iterator, &mut line, &mut col);
                tokens.push(token);
            }
            '0'..='9' => {
                let token = lex_numeric(&mut iterator, &mut line, &mut col)?;
                tokens.push(token);
            }
            ' ' => {
                col += 1;
                iterator.next();
            }
            '\n' => {
                line += 1;
                col = 1;
                iterator.next();
            }
            _ => {
                iterator.next();
            }
        }
    }

    Ok(tokens)
}

fn lex_comment(
    iterator: &mut Peekable<Chars>,
    line: &mut usize,
    col: &mut usize,
) -> Result<Token, LexError> {
    let position = (*line, *col);

    *col += 1;
    let Some('/') = iterator.next() else {
        return Err(LexError("Comment without second slash!".into()));
    };
    let Some('/') = iterator.next() else {
        return Err(LexError("Comment without second slash!".into()));
    };

    let mut read = vec![];

    while let Some(next) = iterator.next_if(|item| *item != '\n') {
        *col += 1;
        read.push(next);
    }

    Ok(Token::Comment {
        value: read.iter().collect(),
        position,
    })
}

fn lex_alphabetic(iterator: &mut Peekable<Chars>, line: &mut usize, col: &mut usize) -> Token {
    let mut read = vec![];

    let position = (*line, *col);

    while let Some(next) = iterator.next_if(|item| item.is_alphabetic()) {
        *col += 1;
        read.push(next)
    }

    let read = read.iter().collect::<String>();

    match read.as_str() {
        "let" => Token::Let { position },
        _ => Token::Id {
            value: read,
            position,
        },
    }
}

fn lex_numeric(
    iterator: &mut Peekable<Chars>,
    line: &mut usize,
    col: &mut usize,
) -> Result<Token, LexError> {
    let mut read = vec![];

    let position = (*line, *col);

    *col += 1;
    while let Some(next) = iterator.next_if(|item| item.is_numeric()) {
        *col += 1;
        read.push(next)
    }

    let read = read.iter().collect::<String>();

    read.parse::<u64>()
        .map(|num| Token::Num {
            value: num,
            position,
        })
        .map_err(|_| LexError("failed to parse numeric".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_alphabetic_keyword() {
        let mut iterator = "let".chars().peekable();

        let mut line = 1;
        let mut col = 1;

        assert_eq!(
            Token::Let { position: (1, 1) },
            lex_alphabetic(&mut iterator, &mut line, &mut col)
        )
    }

    #[test]
    fn test_lex_alphabetic_id() {
        let mut iterator = "letter".chars().peekable();

        let mut line = 1;
        let mut col = 1;

        assert_eq!(
            Token::Id {
                value: "letter".into(),
                position: (1, 1)
            },
            lex_alphabetic(&mut iterator, &mut line, &mut col)
        )
    }

    #[test]
    fn test_lex_numeric() {
        let mut iterator = "1337".chars().peekable();

        let mut line = 1;
        let mut col = 1;

        assert_eq!(
            Ok(Token::Num {
                value: 1337,
                position: (1, 1)
            }),
            lex_numeric(&mut iterator, &mut line, &mut col)
        )
    }

    #[test]
    fn test_lex_comment() {
        let mut iterator = "// some comment".chars().peekable();

        let mut line = 1;
        let mut col = 1;

        assert_eq!(
            Ok(Token::Comment {
                value: " some comment".into(),
                position: (1, 1)
            }),
            lex_comment(&mut iterator, &mut line, &mut col)
        )
    }
}
