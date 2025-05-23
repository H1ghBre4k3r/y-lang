mod token;

pub use token::*;

use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LexError(String);

pub type LexResult<T> = Result<T, LexError>;

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Error for LexError {}

pub struct Lexer<'a> {
    tokens: Vec<Token>,
    lexikon: Lexikon,
    position: usize,
    col: usize,
    line: usize,
    input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: vec![],
            lexikon: Lexikon::new(),
            position: 0,
            col: 0,
            line: 0,
            input,
        }
    }

    fn eat_whitespace(&mut self) {
        while let Some(c) = self.input.as_bytes().get(self.position) {
            if !c.is_ascii_whitespace() {
                return;
            }

            if *c == b'\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
            self.position += 1;
        }
    }

    pub fn lex(mut self) -> LexResult<Vec<Token>> {
        while self.position != self.input.len() {
            self.eat_whitespace();
            let (len, res) = self
                .lexikon
                .find_longest_match(
                    &self.input[self.position..],
                    (self.line, self.col),
                    self.input.to_string(),
                )
                .clone();

            match res {
                Some(t) => self.tokens.push(t),
                None => {
                    if self.position == self.input.len() {
                        return Ok(self.tokens);
                    }
                    return Err(LexError(format!(
                        "Failed to lex '{}' at position {}; remaining '{}'",
                        self.input,
                        self.position,
                        &self.input[self.position..]
                    )));
                }
            };
            self.position += len;
            self.col += len;
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
            Ok(vec![Token::Id {
                value: "letter".into(),
                position: Span::default(),
            }]),
            lexer.lex()
        )
    }

    #[test]
    fn test_lex_numeric() {
        let lexer = Lexer::new("1337");

        assert_eq!(
            Ok(vec![Token::Integer {
                value: 1337,
                position: Span::default(),
            }]),
            lexer.lex()
        )
    }

    #[test]
    fn test_lex_function() {
        let lexer = Lexer::new("fn () {}");

        assert_eq!(
            Ok(vec![
                Token::FnKeyword {
                    position: Span::default(),
                },
                Token::LParen {
                    position: Span::default(),
                },
                Token::RParen {
                    position: Span::default(),
                },
                Token::LBrace {
                    position: Span::default(),
                },
                Token::RBrace {
                    position: Span::default(),
                }
            ]),
            lexer.lex()
        );
    }

    #[test]
    fn test_lex_let() {
        let lexer = Lexer::new("let foo = 42;");

        assert_eq!(
            Ok(vec![
                Token::Let {
                    position: Span::default(),
                },
                Token::Id {
                    value: "foo".into(),
                    position: Span::default(),
                },
                Token::Assign {
                    position: Span::default(),
                },
                Token::Integer {
                    value: 42,
                    position: Span::default(),
                },
                Token::Semicolon {
                    position: Span::default(),
                }
            ]),
            lexer.lex()
        );
    }
}
