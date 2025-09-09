use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Block, Expression},
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

impl From<WhileLoop<()>> for AstNode {
    fn from(value: WhileLoop<()>) -> Self {
        AstNode::WhileLoop(value)
    }
}
