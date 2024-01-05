mod lexmap;
mod token_kind;
mod tokens;

pub use lexmap::*;
use regex::{Match, Regex};
pub use token_kind::*;
pub use tokens::*;

use std::{error::Error, fmt::Display};

struct Lexikon {
    entries: Vec<(Regex, Box<dyn Fn(Match) -> TokenKind>)>,
}

#[macro_export]
macro_rules! terminal {
    ($entries:ident, $name:ident, $value:expr) => {
        Self::insert(
            &mut $entries,
            Regex::new(&$value.escape_unicode().to_string()).unwrap(),
            |_| TokenKind::$name { position: (0, 0) },
        );
    };
}

#[macro_export]
macro_rules! literal {
    ($entries:ident, $name:ident, $value:expr) => {
        Self::insert(&mut $entries, Regex::new($value).unwrap(), |matched| {
            TokenKind::$name {
                position: (0, 0),
                value: matched.as_str().parse().unwrap(),
            }
        });
    };
}

impl<'a> Lexikon {
    pub fn new() -> Lexikon {
        let mut m = vec![];

        terminal!(m, Assign, "=");
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
        terminal!(m, ExclamationMark, "!");
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

        literal!(m, Id, "[a-zA-Z_][a-zA-Z0-9_]*");
        literal!(m, Num, "[0-9]*");

        Lexikon { entries: m }
    }

    fn insert<F: Fn(Match) -> TokenKind + 'static>(
        entries: &mut Vec<(Regex, Box<dyn Fn(Match) -> TokenKind>)>,
        reg: Regex,
        f: F,
    ) {
        entries.push((reg, Box::new(f)))
    }

    pub fn find_longest_match(&self, pattern: &'a str) -> (usize, Option<TokenKind>) {
        let mut longest = (0, None);

        for (reg, mapper) in &self.entries {
            let Some(res) = reg.captures_at(pattern, 0).and_then(|res| res.get(0)) else {
                continue;
            };

            let len = res.len();

            if len > longest.0 && res.start() == 0 {
                longest = (len, Some(mapper(res)));
            }
        }

        longest
    }
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

pub struct Lexer<'a> {
    tokens: Vec<TokenKind>,
    lexikon: Lexikon,
    position: usize,
    input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: vec![],
            lexikon: Lexikon::new(),
            position: 0,
            input,
        }
    }

    fn eat_whitespace(&mut self) {
        while self
            .input
            .as_bytes()
            .get(self.position)
            .map(|c| c.is_ascii_whitespace())
            .unwrap_or(false)
        {
            self.position += 1;
        }
    }

    pub fn lex(mut self) -> LexResult<Vec<TokenKind>> {
        while self.position != self.input.len() {
            self.eat_whitespace();
            let (len, res) = self
                .lexikon
                .find_longest_match(&self.input[self.position..])
                .clone();

            match res {
                Some(t) => self.tokens.push(t),
                None => {
                    if self.position == self.input.len() {
                        return Ok(self.tokens);
                    } else {
                        panic!(
                            "Failed to lex '{}' at position {}; remaining '{}'",
                            self.input,
                            self.position,
                            &self.input[self.position..]
                        );
                    }
                }
            };
            self.position += len;
        }

        Ok(self.tokens)
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
                TokenKind::Assign { position: (0, 0) },
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
