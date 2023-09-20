use lazy_static::lazy_static;
use pesca_parser_derive::Token as ParseToken;
use std::{collections::HashMap, error::Error, fmt::Display, iter::Peekable, str::Chars};

type Position = (usize, usize);

#[derive(Debug, Clone, ParseToken)]
pub enum Token {
    #[terminal]
    Eq {
        position: Position,
    },
    #[terminal]
    Let {
        position: Position,
    },
    Id {
        value: String,
        position: Position,
    },
    Num {
        value: u64,
        position: Position,
    },
    #[terminal]
    Semicolon {
        position: Position,
    },
    Comment {
        value: String,
        position: Position,
    },
    #[terminal]
    Plus {
        position: Position,
    },
    #[terminal]
    Times {
        position: Position,
    },
    #[terminal]
    LParen {
        position: Position,
    },
    #[terminal]
    RParen {
        position: Position,
    },
    #[terminal]
    LBrace {
        position: Position,
    },
    #[terminal]
    RBrace {
        position: Position,
    },
    #[terminal]
    FnKeyword {
        position: Position,
    },
    #[terminal]
    ReturnKeyword {
        position: Position,
    },
    #[terminal]
    Colon {
        position: Position,
    },
    #[terminal]
    Comma {
        position: Position,
    },
}

impl Terminal {
    pub fn to_token(&self, position: Position) -> Token {
        match self {
            Terminal::Eq => Token::Eq { position },
            Terminal::Let => Token::Let { position },
            Terminal::Semicolon => Token::Semicolon { position },
            Terminal::Plus => Token::Plus { position },
            Terminal::Times => Token::Times { position },
            Terminal::LParen => Token::LParen { position },
            Terminal::RParen => Token::RParen { position },
            Terminal::LBrace => Token::LBrace { position },
            Terminal::RBrace => Token::RBrace { position },
            Terminal::FnKeyword => Token::FnKeyword { position },
            Terminal::ReturnKeyword => Token::ReturnKeyword { position },
            Terminal::Colon => Token::Colon { position },
            Terminal::Comma => Token::Comma { position },
        }
    }
}

// TODO: move this to own derive macro
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
                | (LParen { .. }, LParen { .. })
                | (RParen { .. }, RParen { .. })
                | (LBrace { .. }, LBrace { .. })
                | (RBrace { .. }, RBrace { .. })
                | (FnKeyword { .. }, FnKeyword { .. })
                | (ReturnKeyword { .. }, ReturnKeyword { .. })
                | (Colon { .. }, Colon { .. })
                | (Comma { .. }, Comma { .. })
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
            Token::LParen { position } => *position,
            Token::RParen { position } => *position,
            Token::LBrace { position } => *position,
            Token::RBrace { position } => *position,
            Token::FnKeyword { position } => *position,
            Token::ReturnKeyword { position } => *position,
            Token::Colon { position } => *position,
            Token::Comma { position } => *position,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokens<T> {
    tokens: Vec<T>,
    index: usize,
}

impl<T> Tokens<T>
where
    T: Clone,
{
    pub fn new(tokens: Vec<T>) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn next(&mut self) -> Option<T> {
        if self.index < self.tokens.len() {
            let item = self.tokens.get(self.index).cloned();
            self.index += 1;
            return item;
        }

        None
    }

    pub fn peek(&mut self) -> Option<T> {
        return self.tokens.get(self.index).cloned();
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

impl<T> From<Vec<T>> for Tokens<T>
where
    T: Clone,
{
    fn from(value: Vec<T>) -> Self {
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

#[derive(Debug, Clone, Default)]
struct LexMap {
    map: HashMap<&'static str, Terminal>,
}

// TODO: write tests for lex map
impl LexMap {
    pub fn insert(&mut self, key: &'static str, value: Terminal) {
        self.map.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<(Terminal, bool)> {
        let mut longest_key: Option<String> = None;

        // FIXME: This does not work!
        // `f` matches `fn` and therefore, this is returned
        for map_key in self.map.keys() {
            if map_key.starts_with(key) {
                longest_key = if let Some(longest_key) = longest_key {
                    if longest_key.len() < map_key.len() {
                        Some(map_key.to_string())
                    } else {
                        Some(longest_key)
                    }
                } else {
                    Some(map_key.to_string())
                }
            }
        }

        longest_key
            .and_then(|key| self.map.get(key.as_str()))
            .cloned()
    }
}

#[macro_export]
macro_rules! terminal {
    ($map:ident, $name:ident, $value:expr) => {
        $map.insert($value, Terminal::$name);
    };
}

lazy_static! {
    static ref LEX_MAP: LexMap = {
        let mut m = LexMap::default();

        terminal!(m, Eq, "=");
        terminal!(m, Let, "let");
        terminal!(m, Semicolon, ";");
        terminal!(m, Plus, "+");
        terminal!(m, Times, "*");
        terminal!(m, LParen, "(");
        terminal!(m, RParen, ")");
        terminal!(m, LBrace, "{");
        terminal!(m, RBrace, "}");
        terminal!(m, FnKeyword, "fn");
        terminal!(m, ReturnKeyword, "return");
        terminal!(m, Colon, ":");
        terminal!(m, Comma, ",");

        m
    };
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = vec![];

    let mut iterator = input.chars().peekable();

    let mut line = 1;
    let mut col = 1;

    let mut stack = vec![];

    let mut current_matched;

    while let Some(next) = iterator.next() {
        // eat whitespaces
        match next {
            ' ' | '\t' if stack.is_empty() => {
                col += 1;
                continue;
            }
            '\n' if stack.is_empty() => {
                line += 1;
                col = 1;
                continue;
            }
            _ => (),
        }

        stack.push(next);
        col += 1;

        let read = stack.iter().collect::<String>();
        current_matched = LEX_MAP.get(read.as_str());

        let peeked = iterator.peek();
        if let Some(next) = peeked {
            let mut stack_copy = stack.clone();
            stack_copy.push(*next);

            let read = stack_copy.iter().collect::<String>();
            let next_match = LEX_MAP.get(read.as_str());
            if next_match.is_some() {
                continue;
            } else if let Some(terminal) = &current_matched {
                tokens.push(terminal.to_token((line, col)));
                stack = vec![];
                continue;
            }
        } else {
            match &current_matched {
                Some(terminal) => tokens.push(terminal.to_token((line, col))),
                None => todo!(),
            }
        }

        match stack[0] {
            '/' if stack.is_empty() => {
                let token = lex_comment(&mut iterator, &mut line, &mut col)?;
                tokens.push(token);
            }
            'a'..='z' | 'A'..='Z' => {
                let token = lex_alphabetic(&mut iterator, &stack, &mut line, &mut col);
                stack = vec![];
                tokens.push(token);
            }
            '0'..='9' => {
                let token = lex_numeric(&mut iterator, &stack, &mut line, &mut col)?;
                stack = vec![];
                tokens.push(token);
            }
            ' ' | '\t' => {
                stack.remove(0);
                col += 1;
                continue;
            }
            '\n' => {
                stack.remove(0);
                line += 1;
                col = 1;
                continue;
            }
            _ => {}
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

fn lex_alphabetic(
    iterator: &mut Peekable<Chars>,
    stack: &[char],
    line: &mut usize,
    col: &mut usize,
) -> Token {
    let mut read = stack.to_vec();

    let position = (*line, *col);

    *col += 1;

    while let Some(next) = iterator.next_if(|item| item.is_alphanumeric()) {
        *col += 1;
        read.push(next)
    }

    let read = read.iter().collect::<String>();

    Token::Id {
        value: read,
        position,
    }
}

fn lex_numeric(
    iterator: &mut Peekable<Chars>,
    stack: &[char],
    line: &mut usize,
    col: &mut usize,
) -> Result<Token, LexError> {
    let mut read = stack.to_vec();

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
    fn test_lex_alphabetic_id() {
        let mut iterator = "letter".chars().peekable();
        iterator.next();

        let mut line = 1;
        let mut col = 1;

        assert_eq!(
            Token::Id {
                value: "letter".into(),
                position: (1, 1)
            },
            lex_alphabetic(&mut iterator, &['l'], &mut line, &mut col)
        )
    }

    #[test]
    fn test_lex_numeric() {
        let mut iterator = "1337".chars().peekable();
        iterator.next();

        let mut line = 1;
        let mut col = 1;

        assert_eq!(
            Ok(Token::Num {
                value: 1337,
                position: (1, 1)
            }),
            lex_numeric(&mut iterator, &['1'], &mut line, &mut col)
        )
    }

    #[test]
    fn test_lex_comment() {
        let mut iterator = "// some comment".chars().peekable();
        iterator.next();

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

    #[test]
    fn test_lex_function() {
        let input = "fn () {}";
        let token = lex(input);

        assert_eq!(
            Ok(vec![
                Token::FnKeyword { position: (0, 0) },
                Token::LParen { position: (0, 0) },
                Token::RParen { position: (0, 0) },
                Token::LBrace { position: (0, 0) },
                Token::RBrace { position: (0, 0) }
            ]),
            token
        );
    }

    #[test]
    fn test_lex_let() {
        let input = "let foo = 42;";
        let token = lex(input);

        assert_eq!(
            Ok(vec![
                Token::Let { position: (0, 0) },
                Token::Id {
                    value: "foo".into(),
                    position: (0, 0)
                },
                Token::Eq { position: (0, 0) },
                Token::Num {
                    value: 42,
                    position: (0, 0)
                },
                Token::Semicolon { position: (0, 0) }
            ]),
            token
        );
    }
}
