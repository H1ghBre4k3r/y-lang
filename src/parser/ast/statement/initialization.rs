use crate::{
    lexer::Tokens,
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

impl FromTokens for Initialization {
    fn parse(tokens: &mut Tokens) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let matcher = Comb::LET >> Comb::ID >> Comb::EQ >> Comb::EXPR >> Comb::SEMI;

        let results = matcher.parse(tokens)?;

        let AstNode::Id(id) = results[0].clone() else {
            unreachable!()
        };

        let AstNode::Expression(value) = results[1].clone() else {
            unreachable!()
        };

        Ok(AstNode::Initialization(Initialization { id, value }))
    }
}
