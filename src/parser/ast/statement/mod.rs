mod initialization;

pub use self::initialization::*;

use crate::{
    lexer::{Token, Tokens},
    parser::{combinators::Comb, FromTokens, ParseError},
};

use super::{AstNode, Expression, Function};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Function(Function),
    Initialization(Initialization),
    Expression(Expression),
    Return(Expression),
}

impl FromTokens<Token> for Statement {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let Some(next) = tokens.peek() else {
            todo!();
        };

        match next {
            Token::FnKeyword { .. } => {
                let matcher = Comb::FUNCTION;
                let result = matcher.parse(tokens)?;
                let [AstNode::Function(function)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Function(function.clone()).into())
            }
            Token::Let { .. } => {
                let matcher = Comb::INITIALIZATION;
                let result = matcher.parse(tokens)?;
                let [AstNode::Initialization(init)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Initialization(init.clone()).into())
            }
            Token::ReturnKeyword { .. } => {
                let matcher = Comb::RETURN_KEYWORD >> Comb::EXPR >> Comb::SEMI;
                let result = matcher.parse(tokens)?;
                let [AstNode::Expression(expr)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Return(expr.clone()).into())
            }
            token => {
                let matcher = Comb::EXPR;
                let result = matcher.parse(tokens).map_err(|_| ParseError {
                    message: format!("Unexpected token {token:?} while trying to parse Statement"),
                    position: Some(token.position()),
                })?;
                let [AstNode::Expression(expr)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Expression(expr.clone()).into())
            }
        }
    }
}

impl From<Statement> for AstNode {
    fn from(value: Statement) -> Self {
        AstNode::Statement(value)
    }
}
