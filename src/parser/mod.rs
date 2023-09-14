use std::{error::Error, fmt::Display};

mod ast;
pub mod combinators;

use crate::Tokens;

use self::ast::{AstNode, Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub position: Option<(usize, usize)>,
}

impl ParseError {
    pub fn eof(item: &str) -> ParseError {
        ParseError {
            message: format!("hit EOF while parsing {item}"),
            position: None,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((line, col)) = self.position {
            f.write_fmt(format_args!("{} ({}:{})", self.message, line, col))
        } else {
            f.write_str(&self.message)
        }
    }
}

impl Error for ParseError {}

pub trait FromTokens {
    fn parse(tokens: &mut Tokens) -> Result<AstNode, ParseError>;
}

pub fn parse(tokens: &mut Tokens) -> Result<Vec<AstNode>, Box<dyn Error>> {
    let mut statements = vec![];

    while tokens.peek().is_some() {
        let result = Statement::parse(tokens)?;
        statements.push(result);
    }

    Ok(statements)
}
