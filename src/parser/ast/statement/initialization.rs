use std::iter::Peekable;

use crate::{
    lexer::Token,
    parser::{ast::Expression, FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Initialization {
    id: String,
    value: Expression,
}

impl FromTokens for Initialization {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, ParseError>
    where
        I: Iterator<Item = Token>,
        Self: Sized,
    {
        let next = tokens.next().ok_or(ParseError {
            message: "Expeting 'let' in Initialization".into(),
            position: None,
        })?;
        let Token::Let { .. } = next else {
            return Err(ParseError {
                message: "Expeting 'let' in Initialization".into(),
                position: Some(next.position()),
            });
        };

        let next = tokens.next().ok_or(ParseError {
            message: "Expecting identifier in Initialization".into(),
            position: None,
        })?;
        let Token::Id { value: id, .. } = next else {
            return Err(ParseError {
                message: "Expecting identifier in Initialization".into(),
                position: Some(next.position()),
            });
        };

        let next = tokens.next().ok_or(ParseError {
            message: "Expecting '=' in Initialization".into(),
            position: None,
        })?;
        let Token::Eq { .. } = next else {
            return Err(ParseError {
                message: "Expecting '=' in Initialization".into(),
                position: Some(next.position()),
            });
        };

        let value = Expression::parse(tokens)?;

        let next = tokens.next().ok_or(ParseError {
            message: "Expecting ';' in Initialization".into(),
            position: None,
        })?;
        let Token::Semicolon { .. } = next else {
            return Err(ParseError {
                message: "Expecting ';' in Initialization".into(),
                position: Some(next.position()),
            });
        };

        Ok(Initialization { id, value })
    }
}
