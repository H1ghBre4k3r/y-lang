mod id;
mod num;

pub use self::id::*;
pub use self::num::*;

use crate::lexer::Tokens;
use crate::parser::combinators::Comb;
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
                let AstNode::Num(num) = Comb::NUM.parse(tokens)?[0].clone() else {
                    unreachable!()
                };
                Expression::Num(num)
            }
            Token::Id { .. } => {
                let AstNode::Id(id) = Comb::ID.parse(tokens)?[0].clone() else {
                    unreachable!()
                };
                Expression::Id(id)
            }
            _ => todo!(),
        };

        let Some(next) = tokens.peek() else {
            return Ok(expr.into());
        };

        match next {
            Token::Semicolon { .. } => Ok(expr.into()),
            Token::Times { .. } => {
                tokens.next();
                let AstNode::Expression(rhs) = Comb::EXPR.parse(tokens)?[0].clone() else {
                    unreachable!()
                };
                Ok(Expression::Multiplication(Box::new(expr), Box::new(rhs)).into())
            }
            Token::Plus { .. } => {
                tokens.next();
                let AstNode::Expression(rhs) = Comb::EXPR.parse(tokens)?[0].clone() else {
                    unreachable!()
                };
                Ok(Expression::Addition(Box::new(expr), Box::new(rhs)).into())
            }
            _ => todo!(),
        }
    }
}

impl From<Expression> for AstNode {
    fn from(value: Expression) -> Self {
        AstNode::Expression(value)
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
