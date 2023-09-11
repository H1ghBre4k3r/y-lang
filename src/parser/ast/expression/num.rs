use std::iter::Peekable;

use crate::{
    lexer::Token,
    parser::{FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Num(pub u64);

impl FromTokens for Num {
    fn parse<I>(tokens: &mut Peekable<I>) -> Result<Self, ParseError>
    where
        I: Iterator<Item = Token>,
        Self: Sized,
    {
        let value = match tokens.next() {
            Some(Token::Num { value, .. }) => value,
            Some(token) => {
                return Err(ParseError {
                    message: "Tried to parse Num from non Num token".into(),
                    position: Some(token.position()),
                })
            }
            None => return Err(ParseError::eof("Id")),
        };

        Ok(Num(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![Token::Num {
            value: 42,
            position: (0, 0),
        }];
        let mut tokens = tokens.into_iter().peekable();
        assert_eq!(Num::parse(&mut tokens), Ok(Num(42)));
    }

    #[test]
    fn test_error_on_non_num() {
        let tokens = vec![Token::Id {
            value: "some_id".into(),
            position: (0, 0),
        }];
        let mut tokens = tokens.into_iter().peekable();
        assert!(Num::parse(&mut tokens).is_err());
    }

    #[test]
    fn test_error_on_eof() {
        let tokens = vec![];
        let mut tokens = tokens.into_iter().peekable();
        assert!(Num::parse(&mut tokens).is_err());
    }
}
