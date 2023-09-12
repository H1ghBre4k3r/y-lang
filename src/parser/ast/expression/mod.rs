mod id;
mod num;

pub use self::id::*;
pub use self::num::*;

use crate::lexer::Tokens;
use crate::{
    lexer::Token,
    parser::{FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Id(Id),
    Num(Num),
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
}

impl FromTokens for Expression {
    fn parse(tokens: &mut Tokens) -> Result<Self, ParseError> {
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
            Token::Times { .. } => {
                tokens.next();
                Ok(Expression::Multiplication(
                    Box::new(expr),
                    Box::new(Expression::parse(tokens)?),
                ))
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_id() {
        let tokens = vec![Token::Id {
            value: "some_id".into(),
            position: (0, 0),
        }];
        let tokens = tokens;

        assert_eq!(
            Expression::parse(&mut tokens.into()),
            Ok(Expression::Id(Id("some_id".into())))
        )
    }

    #[test]
    fn test_parse_num() {
        let tokens = vec![Token::Num {
            value: 42,
            position: (0, 0),
        }];
        let tokens = tokens;

        assert_eq!(
            Expression::parse(&mut tokens.into()),
            Ok(Expression::Num(Num(42)))
        )
    }
}
