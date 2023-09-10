mod num;

pub use self::num::*;

use std::iter::Peekable;

use crate::{
    lexer::Token,
    parser::{FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Num(Num),
}

impl FromTokens for Expression {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, ParseError>
    where
        I: Iterator<Item = Token>,
        Self: Sized,
    {
        let Some(next) = tokens.peek() else {
            todo!();
        };

        match next {
            Token::Num { .. } => Ok(Expression::Num(Num::parse(tokens)?)),
            _ => todo!(),
        }
    }
}
