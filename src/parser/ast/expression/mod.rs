mod id;
mod num;

pub use self::id::*;
pub use self::num::*;

use std::iter::Peekable;

use crate::{
    lexer::Token,
    parser::{FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Id(Id),
    Num(Num),
    Addition(Box<Expression>, Box<Expression>),
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

        let expr = match next {
            Token::Num { .. } => Expression::Num(Num::parse(tokens)?),
            Token::Id { .. } => Expression::Id(Id::parse(tokens)?),
            _ => todo!(),
        };

        let Some(next) = tokens.peek() else {
            return Ok(expr);
        };

        match next {
            Token::Semicolon { .. } => Ok(expr),
            Token::Plus { .. } => {
                tokens.next();
                Ok(Expression::Addition(
                    Box::new(expr),
                    Box::new(Expression::parse(tokens)?),
                ))
            }
            _ => todo!(),
        }
    }
}
