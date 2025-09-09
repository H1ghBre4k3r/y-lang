use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Function, TypeName},
};

use super::MethodDeclaration;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Instance<T> {
    pub name: TypeName,
    pub functions: Vec<Function<T>>,
    pub declarations: Vec<MethodDeclaration<T>>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::Instance> for Instance<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Instance>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        let mut functions = Vec::new();
        let mut declarations = Vec::new();

        for method in value.methods {
            match method {
                grammar::InstanceMethod::FunctionDeclaration(function_decl) => {
                    functions.push(Function::transform(function_decl, source));
                }
                grammar::InstanceMethod::MethodDeclaration(method_decl) => {
                    declarations.push(MethodDeclaration::transform(method_decl, source));
                }
            }
        }

        Instance {
            name: TypeName::transform(value.name, source),
            functions,
            declarations,
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<Instance<()>> for AstNode {
    fn from(value: Instance<()>) -> Self {
        AstNode::Instance(value)
    }
}
