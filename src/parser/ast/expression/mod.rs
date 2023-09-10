mod num;

pub use self::num::*;

use std::{error::Error, iter::Peekable};

use crate::{lexer::Token, parser::FromTokens};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Num(Num),
}

impl FromTokens for Expression {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, Box<dyn Error>>
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
