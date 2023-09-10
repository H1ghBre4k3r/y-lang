use std::{error::Error, iter::Peekable};

use crate::{lexer::Token, parser::FromTokens};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Num(u64);

impl FromTokens for Num {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, Box<dyn Error>>
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
