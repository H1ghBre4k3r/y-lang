use crate::{
    lexer::Token,
    parser::{
        ast::{AstNode, Block, Expression},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileLoop<T> {
    pub condition: Expression<T>,
    pub block: Block<T>,
    pub info: T,
}

impl FromTokens<Token> for WhileLoop<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let matcher =
            Comb::WHILE_KEYWORD >> Comb::LPAREN >> Comb::EXPR >> Comb::RPAREN >> Comb::BLOCK;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Expression(condition)) = result.first() else {
            unreachable!()
        };

        let Some(AstNode::Block(block)) = result.get(1) else {
            unreachable!()
        };

        Ok(WhileLoop {
            condition: condition.clone(),
            block: block.clone(),
            info: (),
        }
        .into())
    }
}

impl From<WhileLoop<()>> for AstNode {
    fn from(value: WhileLoop<()>) -> Self {
        AstNode::WhileLoop(value)
    }
}
