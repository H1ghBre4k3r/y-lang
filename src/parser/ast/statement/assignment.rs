use crate::{
    lexer::Token,
    parser::{
        ast::{AstNode, Expression, Id},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<T> {
    pub id: Id<T>,
    pub value: Expression<T>,
    pub info: T,
}

impl FromTokens<Token> for Assignment<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::ID >> Comb::ASSIGN;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(id)) = result.first() else {
            unreachable!()
        };

        let matcher = Comb::EXPR;

        let result = matcher.parse(tokens).map_err(|e| {
            tokens.add_error(e.clone());
            e
        })?;

        let Some(AstNode::Expression(value)) = result.first() else {
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
