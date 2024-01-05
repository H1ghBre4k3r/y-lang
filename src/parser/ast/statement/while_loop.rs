use crate::{
    lexer::{Token, Tokens},
    parser::{
        ast::{AstNode, Block, Expression},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileLoop {
    pub condition: Expression,
    pub block: Block,
}

impl FromTokens<Token> for WhileLoop {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
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
        }
        .into())
    }
}

impl From<WhileLoop> for AstNode {
    fn from(value: WhileLoop) -> Self {
        AstNode::WhileLoop(value)
    }
}
