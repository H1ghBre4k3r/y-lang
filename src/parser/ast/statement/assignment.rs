use crate::{
    lexer::{TokenKind, Tokens},
    parser::{
        ast::{AstNode, Expression, Id},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub id: Id,
    pub value: Expression,
}

impl FromTokens<TokenKind> for Assignment {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError> {
        let matcher = Comb::ID >> Comb::ASSIGN >> Comb::EXPR;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(id)) = result.first() else {
            unreachable!()
        };

        let Some(AstNode::Expression(value)) = result.get(1) else {
            unreachable!()
        };

        Ok(Assignment {
            id: id.clone(),
            value: value.clone(),
        }
        .into())
    }
}

impl From<Assignment> for AstNode {
    fn from(value: Assignment) -> Self {
        AstNode::Assignment(value)
    }
}
