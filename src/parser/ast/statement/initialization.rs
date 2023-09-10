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
        assert_eq!(tokens.next(), Some(Token::Let));

        let Some(Token::Id(id)) = tokens.next() else {
            todo!()
        };

        assert_eq!(tokens.next(), Some(Token::Eq));

        let value = Expression::parse(tokens)?;

        assert_eq!(tokens.next(), Some(Token::Semicolon));

        Ok(Initialization { id, value })
    }
}
