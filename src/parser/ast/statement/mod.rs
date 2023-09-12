mod initialization;

pub use self::initialization::*;

use crate::{
    lexer::{Token, Tokens},
    parser::{FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Initialization(Initialization),
}

impl FromTokens for Statement {
    fn parse(tokens: &mut Tokens) -> Result<Self, ParseError>
    where
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
