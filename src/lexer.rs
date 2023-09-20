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
    // TODO: think about lexing comments
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

impl LexMap {
    pub fn insert(&mut self, key: &'static str, value: Terminal) {
        self.map.insert(key, value);
    }

    pub fn can_match(&self, key: &str) -> bool {
        for map_key in self.map.keys() {
            if map_key.starts_with(key) {
                return true;
            }
        }
        false
    }

    pub fn get(&self, key: &str) -> Option<Terminal> {
        self.map.get(key).cloned()
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

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    tokens: Vec<Token>,
    iterator: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let iterator = input.chars().peekable();

        Self {
            tokens: vec![],
            iterator,
            line: 1,
            col: 1,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.iterator.peek()
    }

    fn next(&mut self) -> Option<char> {
        self.iterator.next()
    }

    fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        self.iterator.next_if(func)
    }

    fn eat_whitespace(&mut self) {
        while let Some(next) = self.next_if(|item| item.is_whitespace()) {
            match next {
                ' ' | '\t' => self.col += 1,
                '\n' => {
                    self.col = 1;
                    self.line += 1;
                }
                _ => {}
            }
        }
    }

    pub fn lex(mut self) -> Result<Vec<Token>, LexError> {
        self.lex_internal()?;

        Ok(self.tokens)
    }

    pub fn lex_internal(&mut self) -> Result<(), LexError> {
        self.eat_whitespace();

        let Some(next) = self.peek() else {
            return Ok(());
        };

        match next {
            'a'..='z' | 'A'..='Z' => self.lex_alphanumeric()?,
            '0'..='9' => self.lex_numeric()?,
            _ => self.lex_special()?,
        };

        Ok(())
    }

    fn lex_special(&mut self) -> Result<(), LexError> {
        let mut stack = vec![];

        let position = (self.line, self.col);

        while let Some(next) = self.next() {
            self.col += 1;
            stack.push(next);

            let read = stack.iter().collect::<String>();

            let can_read_next = self
                .peek()
                .map(|item| {
                    let mut stack = stack.clone();
                    stack.push(*item);
                    let read = stack.iter().collect::<String>();
                    LEX_MAP.can_match(read.as_str())
                })
                .unwrap_or(false);

            if can_read_next {
                continue;
            }

            let Some(current_match) = LEX_MAP.get(read.as_str()) else {
                return Err(LexError(format!("failed to lex '{read}'")));
            };

            self.tokens.push(current_match.to_token(position));
            break;
        }

        self.lex_internal()
    }

    fn lex_alphanumeric(&mut self) -> Result<(), LexError> {
        let mut stack = vec![];

        let position = (self.line, self.col);

        while let Some(next) = self.next_if(|item| item.is_alphanumeric()) {
            self.col += 1;
            stack.push(next);
        }

        let read = stack.iter().collect::<String>();

        if let Some(token) = LEX_MAP.get(read.as_str()) {
            self.tokens.push(token.to_token(position));
        } else {
            self.tokens.push(Token::Id {
                value: read,
                position,
            })
        }

        self.lex_internal()
    }

    fn lex_numeric(&mut self) -> Result<(), LexError> {
        let mut stack = vec![];

        let position = (self.line, self.col);

        while let Some(next) = self.next_if(|item| item.is_numeric()) {
            self.col += 1;
            stack.push(next)
        }

        let read = stack.iter().collect::<String>();

        let num = read
            .parse::<u64>()
            .map(|num| Token::Num {
                value: num,
                position,
            })
            .map_err(|_| LexError("failed to parse numeric".into()))?;

        self.tokens.push(num);

        self.lex_internal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_alphabetic_id() {
        let lexer = Lexer::new("letter");

        assert_eq!(
            Ok(vec![Token::Id {
                value: "letter".into(),
                position: (1, 1)
            }]),
            lexer.lex()
        )
    }

    #[test]
    fn test_lex_numeric() {
        let lexer = Lexer::new("1337");

        assert_eq!(
            Ok(vec![Token::Num {
                value: 1337,
                position: (1, 1)
            }]),
            lexer.lex()
        )
    }

    #[test]
    fn test_lex_function() {
        let lexer = Lexer::new("fn () {}");

        assert_eq!(
            Ok(vec![
                Token::FnKeyword { position: (0, 0) },
                Token::LParen { position: (0, 0) },
                Token::RParen { position: (0, 0) },
                Token::LBrace { position: (0, 0) },
                Token::RBrace { position: (0, 0) }
            ]),
            lexer.lex()
        );
    }

    #[test]
    fn test_lex_let() {
        let lexer = Lexer::new("let foo = 42;");

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
            lexer.lex()
        );
    }
}
