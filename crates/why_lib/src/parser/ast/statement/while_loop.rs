use crate::{
    grammar::{self, FromGrammar},
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Block, Expression},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WhileLoop<T> {
    pub condition: Expression<T>,
    pub block: Block<T>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::WhileStatement> for WhileLoop<()> {
    fn transform(item: rust_sitter::Spanned<grammar::WhileStatement>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        WhileLoop {
            condition: Expression::transform(*value.condition, source),
            block: Block::transform(value.block.value, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl FromTokens<Token> for WhileLoop<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

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
            position,
        }
        .into())
    }
}

impl From<WhileLoop<()>> for AstNode {
    fn from(value: WhileLoop<()>) -> Self {
        AstNode::WhileLoop(value)
    }
}
