use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Id, TypeName},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StructDeclaration<T> {
    pub id: Id<T>,
    pub fields: Vec<StructFieldDeclaration<T>>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::StructDeclaration> for StructDeclaration<()> {
    fn transform(item: rust_sitter::Spanned<grammar::StructDeclaration>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        StructDeclaration {
            id: Id::transform(value.id, source),
            fields: value
                .fields
                .into_iter()
                .map(|field| StructFieldDeclaration::transform(field, source))
                .collect(),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<StructDeclaration<()>> for AstNode {
    fn from(value: StructDeclaration<()>) -> Self {
        Self::StructDeclaration(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StructFieldDeclaration<T> {
    pub name: Id<T>,
    pub type_name: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::StructFieldDeclaration> for StructFieldDeclaration<()> {
    fn transform(
        item: rust_sitter::Spanned<grammar::StructFieldDeclaration>,
        source: &str,
    ) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        StructFieldDeclaration {
            name: Id::transform(value.name, source),
            type_name: TypeName::transform(value.type_annotation.type_name, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<StructFieldDeclaration<()>> for AstNode {
    fn from(value: StructFieldDeclaration<()>) -> Self {
        Self::StructFieldDeclaration(value)
    }
}
