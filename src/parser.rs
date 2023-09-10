use std::{error::Error, iter::Peekable};

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Statement {
    Initialization(Initialization),
}

impl FromTokens for Statement {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, Box<dyn Error>>
    where
        I: Iterator<Item = Token>,
        Self: Sized,
    {
        let Some(next) = tokens.peek() else {
            todo!();
        };

        match next {
            Token::Let => Ok(Statement::Initialization(Initialization::parse(tokens)?)),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Initialization {
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expression {
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
            Token::Num(_) => Ok(Expression::Num(Num::parse(tokens)?)),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Num(u64);

impl FromTokens for Num {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, Box<dyn Error>>
    where
        I: Iterator<Item = Token>,
        Self: Sized,
    {
        let Some(Token::Num(num)) = tokens.next() else {
            todo!()
        };

        Ok(Num(num))
    }
}

trait FromTokens {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, Box<dyn Error>>
    where
        I: Iterator<Item = Token>,
        Self: Sized;
}

pub fn parse(tokens: Vec<Token>) {
    let mut tokens = tokens.into_iter().peekable();

    while tokens.peek().is_some() {
        let result = Statement::parse(&mut tokens);
        println!("{result:#?}");
    }
}
