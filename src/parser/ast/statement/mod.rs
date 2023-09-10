mod initialization;

pub use self::initialization::*;

use std::iter::Peekable;

use crate::{
    lexer::Token,
    parser::{FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Initialization(Initialization),
}

impl FromTokens for Statement {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, ParseError>
    where
        I: Iterator<Item = Token>,
        Self: Sized,
    {
        let Some(next) = tokens.peek() else {
            todo!();
        };

        match next {
            Token::Let { .. } => Ok(Statement::Initialization(Initialization::parse(tokens)?)),
            _ => todo!(),
        }
    }
}
