use std::{error::Error, fmt::Display};

pub mod ast;
pub mod combinators;
pub mod direct_parsing;
mod parse_state;

pub use self::parse_state::*;

use crate::lexer::{GetPosition, Span, Token};

use self::ast::{AstNode, TopLevelStatement};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParseError {
    pub message: String,
    pub position: Option<Span>,
}

impl ParseError {
    pub fn eof(item: &str) -> ParseError {
        ParseError {
            message: format!("hit EOF while parsing {item}"),
            position: None,
        }
    }
}

impl ParseState<Token> {
    pub fn prev_span(&self) -> Result<Span, ParseError> {
        match self.peek_reverse() {
            Some(token) => Ok(token.position()),
            None => Err(ParseError {
                message: "hit EOF".into(),
                position: None,
            }),
        }
    }

    pub fn span(&self) -> Result<Span, ParseError> {
        match self.peek() {
            Some(token) => Ok(token.position()),
            None => Err(ParseError {
                message: "hit EOF".into(),
                position: None,
            }),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pos) = &self.position {
            f.write_str(pos.to_string(&self.message).as_str())
        } else {
            f.write_str(&self.message)
        }
    }
}

impl Error for ParseError {}

pub trait FromTokens<T> {
    fn parse(tokens: &mut ParseState<T>) -> Result<AstNode, ParseError>;
}

// New parsing function that uses the direct parser instead of combinators
// New parsing function that uses the direct parser instead of combinators
pub fn parse(tokens: &mut ParseState<Token>) -> Result<Vec<TopLevelStatement<()>>, ParseError> {
    let mut statements = vec![];

    while tokens.peek().is_some() {
        match TopLevelStatement::parse(tokens) {
            Ok(result) => {
                statements.push(result);
            }
            Err(e) => {
                if let Some(e) = tokens.errors.first() {
                    return Err(e.clone());
                }
                return Err(e.clone());
            }
        }
    }

    if let Some(e) = tokens.errors.first() {
        return Err(e.clone());
    }

    Ok(statements)
}
