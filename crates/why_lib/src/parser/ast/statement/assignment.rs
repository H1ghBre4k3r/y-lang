use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Expression, Id},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<T> {
    pub id: Id<T>,
    pub rvalue: Expression<T>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Assignment<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::ID >> Comb::ASSIGN;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(id)) = result.first() else {
            unreachable!()
        };

        let matcher = Comb::EXPR;

        let result = matcher.parse(tokens).inspect_err(|e| {
            tokens.add_error(e.clone());
        })?;

        let Some(AstNode::Expression(rvalue)) = result.first() else {
            unreachable!()
        };

        Ok(Assignment {
            id: id.clone(),
            rvalue: rvalue.clone(),
            info: (),
            position,
        }
        .into())
    }
}

impl From<Assignment<()>> for AstNode {
    fn from(value: Assignment<()>) -> Self {
        AstNode::Assignment(value)
    }
}
