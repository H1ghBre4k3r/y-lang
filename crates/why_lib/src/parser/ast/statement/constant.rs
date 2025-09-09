use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Expression, Id, TypeName},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Constant<T> {
    pub id: Id<T>,
    pub type_name: TypeName,
    pub value: Expression<T>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::Constant> for Constant<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Constant>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        Constant {
            id: Id::transform(value.identifier, source),
            type_name: TypeName::transform(value.type_annotation.type_name, source),
            value: Expression::transform(value.value, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<Constant<()>> for AstNode {
    fn from(value: Constant<()>) -> Self {
        AstNode::Constant(value)
    }
}
