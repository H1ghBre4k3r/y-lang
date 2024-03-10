use crate::{
    lexer::{Token, Tokens},
    parser::{
        ast::{AstNode, Expression, Id},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<T> {
    pub id: Id<T>,
    pub value: Expression<T>,
    pub info: T,
}

impl FromTokens<Token> for Assignment<()> {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
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
            info: (),
        }
        .into())
    }
}

impl From<Assignment<()>> for AstNode {
    fn from(value: Assignment<()>) -> Self {
        AstNode::Assignment(value)
    }
}
