use std::{error::Error, iter::Peekable};

use crate::{
    lexer::Token,
    parser::{ast::Expression, FromTokens},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Initialization {
    id: String,
    value: Expression,
}

impl FromTokens for Initialization {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, Box<dyn Error>>
    where
        I: Iterator<Item = Token>,
        Self: Sized,
    {
        let Some(Token::Let { .. }) = tokens.next() else {
            todo!()
        };

        let Some(Token::Id { value: id, .. }) = tokens.next() else {
            todo!()
        };

        let Some(Token::Eq { .. }) = tokens.next() else {
            todo!()
        };

        let value = Expression::parse(tokens)?;

        let Some(Token::Semicolon { .. }) = tokens.next() else {
            todo!()
        };

        Ok(Initialization { id, value })
    }
}
