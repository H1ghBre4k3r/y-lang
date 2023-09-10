use std::{error::Error, iter::Peekable};

mod ast;

use crate::lexer::Token;

use self::ast::Statement;

pub trait FromTokens {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, Box<dyn Error>>
    where
        I: Iterator<Item = Token>,
        Self: Sized;
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, Box<dyn Error>> {
    let mut tokens = tokens.into_iter().peekable();

    let mut statements = vec![];

    while tokens.peek().is_some() {
        let result = Statement::parse(&mut tokens)?;
        statements.push(result);
    }

    Ok(statements)
}
