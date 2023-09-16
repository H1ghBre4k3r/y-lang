mod initialization;

pub use self::initialization::*;

use crate::{
    lexer::{Token, Tokens},
    parser::{combinators::Comb, FromTokens, ParseError},
};

use super::AstNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Initialization(Initialization),
}

impl FromTokens for Statement {
    fn parse(tokens: &mut Tokens) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let Some(next) = tokens.peek() else {
            todo!();
        };

        match next {
            Token::Let { .. } => {
                let matcher = Comb::INITIALIZATION;
                let result = matcher.parse(tokens)?;
                let [AstNode::Initialization(init)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Initialization(init.clone()).into())
            }
            token => Err(ParseError {
                message: format!("Unexpected token {token:?} while trying to parse Statement"),
                position: Some(token.position()),
            }),
        }
    }
}

impl From<Statement> for AstNode {
    fn from(value: Statement) -> Self {
        AstNode::Statement(value)
    }
}
