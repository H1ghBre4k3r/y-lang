mod function;
mod id;
mod if_expression;
mod num;

pub use self::function::*;
pub use self::id::*;
pub use self::if_expression::*;
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
    Function(Function),
    If(If),
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Parens(Box<Expression>),
}

impl FromTokens<Token> for Expression {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let expr = if let Some(Token::LParen { .. }) = tokens.peek() {
            let matcher = Comb::LPAREN >> Comb::EXPR >> Comb::RPAREN;
            let result = matcher.parse(tokens)?;
            let expr = match result.get(0) {
                Some(AstNode::Expression(rhs)) => rhs.clone(),
                None | Some(_) => unreachable!(),
            };
            Expression::Parens(Box::new(expr))
        } else {
            let matcher = Comb::FUNCTION | Comb::IF | Comb::NUM | Comb::ID;
            let result = matcher.parse(tokens)?;
            match result.get(0) {
                Some(AstNode::Id(id)) => Expression::Id(id.clone()),
                Some(AstNode::Num(num)) => Expression::Num(num.clone()),
                Some(AstNode::Function(func)) => {
                    return Ok(Expression::Function(func.clone()).into())
                }
                Some(AstNode::If(if_expression)) => Expression::If(if_expression.clone()),
                None | Some(_) => unreachable!(),
            }
        };

        let Some(next) = tokens.peek() else {
            return Ok(expr.into());
        };

        let tuple = match next {
            Token::Times { .. } => {
                tokens.next();
                Expression::Multiplication
            }
            Token::Plus { .. } => {
                tokens.next();
                Expression::Addition
            }
            _ => return Ok(expr.into()),
        };

        let matcher = Comb::EXPR;
        let result = matcher.parse(tokens)?;
        let rhs = match result.get(0) {
            Some(AstNode::Expression(rhs)) => rhs.clone(),
            None | Some(_) => unreachable!(),
        };

        Ok(tuple(Box::new(expr), Box::new(rhs)).into())
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
