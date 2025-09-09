use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Id, TypeName},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MethodDeclaration<T> {
    pub id: Id<T>,
    pub parameter_types: Vec<TypeName>,
    pub return_type: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::MethodDeclaration> for MethodDeclaration<()> {
    fn transform(item: rust_sitter::Spanned<grammar::MethodDeclaration>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        MethodDeclaration {
            id: Id::transform(value.id, source),
            parameter_types: value
                .parameter_types
                .into_iter()
                .map(|param| TypeName::transform(param, source))
                .collect(),
            return_type: TypeName::transform(value.return_type_annotation.type_name, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<MethodDeclaration<()>> for AstNode {
    fn from(value: MethodDeclaration<()>) -> Self {
        AstNode::MethodDeclaration(value)
    }
}
