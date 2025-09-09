use std::{error::Error, fmt::Display};

pub mod ast;

#[cfg(test)]
pub mod test_helpers;

use crate::{
    grammar::{FromGrammar, Program},
    lexer::Span,
};

use self::ast::TopLevelStatement;

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

pub fn parse_program(program: Program, source: &str) -> Vec<TopLevelStatement<()>> {
    let mut statements = vec![];

    for statement in program.statements {
        statements.push(TopLevelStatement::transform(statement, source));
    }

    statements
}
