use std::{error::Error, fmt::Display};

mod ast;
pub mod combinators;

use crate::lexer::{TokenKind, Tokens};

use self::{ast::AstNode, combinators::Comb};

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

pub trait FromTokens<T> {
    fn parse(tokens: &mut Tokens<T>) -> Result<AstNode, ParseError>;
}

pub fn parse(tokens: &mut Tokens<TokenKind>) -> Result<Vec<AstNode>, Box<dyn Error>> {
    let mut statements = vec![];

    let matcher = Comb::STATEMENT;
    while tokens.peek().is_some() {
        let result = matcher.parse(tokens)?;
        let [AstNode::Statement(statement)] = result.as_slice() else {
            unreachable!()
        };
        statements.push(statement.clone().into());
    }

    Ok(statements)
}
