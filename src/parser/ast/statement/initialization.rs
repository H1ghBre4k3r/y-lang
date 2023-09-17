use crate::{
    lexer::{Token, Tokens},
    parser::{
        ast::{AstNode, Expression, Id},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Initialization {
    id: Id,
    value: Expression,
}

impl FromTokens<Token> for Initialization {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let matcher = Comb::LET >> Comb::ID >> Comb::EQ >> Comb::EXPR >> Comb::SEMI;

        let result = matcher.parse(tokens)?;
        let [AstNode::Id(id), AstNode::Expression(value)] = result.as_slice() else {
            unreachable!();
        };

        Ok(Initialization {
            id: id.clone(),
            value: value.clone(),
        }
        .into())
    }
}

impl From<Initialization> for AstNode {
    fn from(value: Initialization) -> Self {
        AstNode::Initialization(value)
    }
}
