use std::iter::Peekable;

use crate::{
    lexer::Token,
    parser::{FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Num(u64);

impl FromTokens for Num {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, ParseError>
    where
        I: Iterator<Item = Token>,
        Self: Sized,
    {
        let Some(Token::Num { value, .. }) = tokens.next() else {
            todo!()
        };

        Ok(Num(value))
    }
}
