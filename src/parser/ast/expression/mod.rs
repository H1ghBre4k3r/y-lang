mod id;
mod num;

pub use self::id::*;
pub use self::num::*;

use crate::lexer::Tokens;
use crate::{
    lexer::Token,
    parser::{FromTokens, ParseError},
};

use super::AstNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Id(Id),
    Num(Num),
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
}

impl FromTokens for Expression {
    fn parse(tokens: &mut Tokens) -> Result<AstNode, ParseError> {
        let Some(next) = tokens.peek() else {
            todo!();
        };

        let expr = match next {
            Token::Num { .. } => {
                let AstNode::Num(num) = Num::parse(tokens)? else {
                    unreachable!();
                };
                Expression::Num(num)
            }
            Token::Id { .. } => {
                let AstNode::Id(id) = Id::parse(tokens)? else {
                    unreachable!()
                };
                Expression::Id(id)
            }
            _ => todo!(),
        };

        let Some(next) = tokens.peek() else {
            return Ok(AstNode::Expression(expr));
        };

        match next {
            Token::Semicolon { .. } => Ok(AstNode::Expression(expr)),
            Token::Times { .. } => {
                tokens.next();
                let AstNode::Expression(rhs) = Expression::parse(tokens)? else {
                    unreachable!()
                };
                Ok(AstNode::Expression(Expression::Multiplication(
                    Box::new(expr),
                    Box::new(rhs),
                )))
            }
            Token::Plus { .. } => {
                tokens.next();
                let AstNode::Expression(rhs) = Expression::parse(tokens)? else {
                    unreachable!()
                };
                Ok(AstNode::Expression(Expression::Addition(
                    Box::new(expr),
                    Box::new(rhs),
                )))
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
            Ok(AstNode::Expression(Expression::Id(Id("some_id".into()))))
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
            Ok(AstNode::Expression(Expression::Num(Num(42))))
        )
    }
}
