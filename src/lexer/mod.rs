mod lexmap;
mod token_kind;
mod tokens;

pub use lexmap::*;
pub use token_kind::*;
pub use tokens::*;

use lazy_static::lazy_static;
use std::{error::Error, fmt::Display, iter::Peekable, str::Chars};

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
        terminal!(m, Const, "const");
        terminal!(m, Mut, "mut");
        terminal!(m, Semicolon, ";");
        terminal!(m, Plus, "+");
        terminal!(m, Minus, "-");
        terminal!(m, Times, "*");
        terminal!(m, LParen, "(");
        terminal!(m, RParen, ")");
        terminal!(m, LBrace, "{");
        terminal!(m, RBrace, "}");
        terminal!(m, LBracket, "[");
        terminal!(m, RBracket, "]");
        terminal!(m, FnKeyword, "fn");
        terminal!(m, IfKeyword, "if");
        terminal!(m, ElseKeyword, "else");
        terminal!(m, WhileKeyword, "while");
        terminal!(m, ReturnKeyword, "return");
        terminal!(m, Colon, ":");
        terminal!(m, Comma, ",");
        terminal!(m, Dot, ".");
        terminal!(m, SmallRightArrow, "->");
        terminal!(m, BigRightArrow, "=>");
        terminal!(m, Backslash, "\\");
        terminal!(m, Equal, "==");
        terminal!(m, GreaterThan, ">");
        terminal!(m, LessThan, "<");
        terminal!(m, GreaterOrEqual, ">=");
        terminal!(m, LessOrEqual, "<=");
        terminal!(m, Ampersand, "&");
        terminal!(m, DeclareKeyword, "declare");
        terminal!(m, StructKeyword, "struct");

        m
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError(String);

pub type LexResult<T> = Result<T, LexError>;

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Error for LexError {}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    tokens: Vec<TokenKind>,
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

    pub fn lex(mut self) -> LexResult<Vec<TokenKind>> {
        self.lex_internal()?;

        Ok(self.tokens)
    }

    pub fn lex_internal(&mut self) -> LexResult<()> {
        self.eat_whitespace();

        let Some(next) = self.peek() else {
            return Ok(());
        };

        match next {
            'a'..='z' | 'A'..='Z' | '_' => self.lex_alphanumeric()?,
            '0'..='9' => self.lex_numeric()?,
            _ => self.lex_special()?,
        };

        Ok(())
    }

    fn lex_comment(&mut self) -> LexResult<()> {
        assert_eq!(Some('/'), self.next());

        let mut stack = vec![];
        let position = (self.line, self.col);

        while let Some(next) = self.next_if(|c| *c != '\n') {
            stack.push(next);
        }

        self.tokens.push(TokenKind::Comment {
            value: stack.iter().collect(),
            position,
        });

        self.lex_internal()
    }

    fn lex_special(&mut self) -> LexResult<()> {
        let mut stack = vec![];

        let position = (self.line, self.col);

        while let Some(next) = self.next() {
            if next == '/' && stack.is_empty() {
                return self.lex_comment();
            }

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

    fn lex_alphanumeric(&mut self) -> LexResult<()> {
        let mut stack = vec![];

        let position = (self.line, self.col);

        while let Some(next) = self.next_if(|item| item.is_alphanumeric() || *item == '_') {
            self.col += 1;
            stack.push(next);
        }

        let read = stack.iter().collect::<String>();

        if let Some(token) = LEX_MAP.get(read.as_str()) {
            self.tokens.push(token.to_token(position));
        } else {
            self.tokens.push(TokenKind::Id {
                value: read,
                position,
            })
        }

        self.lex_internal()
    }

    fn lex_numeric(&mut self) -> LexResult<()> {
        let mut stack = vec![];

        let position = (self.line, self.col);

        while let Some(next) = self.next_if(|item| item.is_numeric()) {
            self.col += 1;
            stack.push(next)
        }

        let read = stack.iter().collect::<String>();

        let num = read
            .parse::<u64>()
            .map(|num| TokenKind::Num {
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
            Ok(vec![TokenKind::Id {
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
            Ok(vec![TokenKind::Num {
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
                TokenKind::FnKeyword { position: (0, 0) },
                TokenKind::LParen { position: (0, 0) },
                TokenKind::RParen { position: (0, 0) },
                TokenKind::LBrace { position: (0, 0) },
                TokenKind::RBrace { position: (0, 0) }
            ]),
            lexer.lex()
        );
    }

    #[test]
    fn test_lex_let() {
        let lexer = Lexer::new("let foo = 42;");

        assert_eq!(
            Ok(vec![
                TokenKind::Let { position: (0, 0) },
                TokenKind::Id {
                    value: "foo".into(),
                    position: (0, 0)
                },
                TokenKind::Eq { position: (0, 0) },
                TokenKind::Num {
                    value: 42,
                    position: (0, 0)
                },
                TokenKind::Semicolon { position: (0, 0) }
            ]),
            lexer.lex()
        );
    }
}
