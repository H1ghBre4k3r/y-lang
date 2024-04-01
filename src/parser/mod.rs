use colored::Colorize;
use std::{error::Error, fmt::Display};

pub mod ast;
pub mod combinators;
mod parse_state;

pub use self::parse_state::*;

use crate::lexer::{Span, Token};

use self::{
    ast::{AstNode, Statement},
    combinators::Comb,
};

#[derive(Debug, Clone, PartialEq, Eq)]
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
            let Span { line, col, source } = pos;
            let lines = source.lines().collect::<Vec<_>>();
            let prev_line = if *line > 2 { lines[*line - 2] } else { "" };
            let line_str = lines[*line - 1];

            let left_margin = format!("{line}").len();
            let left_margin_fill = vec![' '; left_margin].iter().collect::<String>();

            let left_padding_fill = vec![' '; col.start - 1].iter().collect::<String>();

            let error_len = vec!['^'; col.end - col.start]
                .iter()
                .collect::<String>()
                .red();

            f.write_fmt(format_args!(
                "{left_margin_fill} |{prev_line} \n{line} |{line_str} \n{left_margin_fill} |{left_padding_fill}{error_len}   {}",
                self.message
            ))
        } else {
            f.write_str(&self.message)
        }
    }
}

impl Error for ParseError {}

pub trait FromTokens<T> {
    fn parse(tokens: &mut ParseState<T>) -> Result<AstNode, ParseError>;
}

pub fn parse(tokens: &mut ParseState<Token>) -> Result<Vec<Statement<()>>, Box<dyn Error>> {
    let mut statements = vec![];

    let matcher = Comb::STATEMENT;
    while tokens.peek().is_some() {
        match matcher.parse(tokens) {
            Ok(result) => {
                let [AstNode::Statement(statement)] = result.as_slice() else {
                    unreachable!()
                };
                statements.push(statement.clone());
            }
            Err(e) => {
                if let Some(e) = tokens.errors.first() {
                    return Err(Box::new(e.clone()));
                }
                return Err(Box::new(e.clone()));
            }
        }
    }

    if let Some(e) = tokens.errors.first() {
        return Err(Box::new(e.clone()));
    }

    Ok(statements)
}
