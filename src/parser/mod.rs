use std::{error::Error, fmt::Display, iter::Peekable};

mod ast;

use crate::lexer::Token;

use self::ast::Statement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    message: String,
    position: Option<(usize, usize)>,
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
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, ParseError>
    where
        I: Iterator<Item = Token>,
        Self: Sized;
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, Box<dyn Error>> {
    let mut tokens = tokens.into_iter().peekable();

    let mut statements = vec![];

    while tokens.peek().is_some() {
        let result = Statement::parse(&mut tokens)?;
        statements.push(result);
    }

    Ok(statements)
}
