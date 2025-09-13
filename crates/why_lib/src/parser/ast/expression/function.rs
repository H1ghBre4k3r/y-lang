use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Statement, TypeName},
};

use super::{Block, Id};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Function<T> {
    pub id: Id<T>,
    pub parameters: Vec<FunctionParameter<T>>,
    pub return_type: TypeName,
    pub body: Block<T>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::FunctionDeklaration> for Function<()> {
    fn transform(item: rust_sitter::Spanned<grammar::FunctionDeklaration>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        // Keep the block intact instead of extracting statements
        let block = Block::transform(value.block, source);

        Function {
            id: Id::transform(value.ident, source),
            parameters: value
                .parameters
                .into_iter()
                .map(|param| FunctionParameter::transform(param, source))
                .collect(),
            return_type: TypeName::transform(value.type_annotation.type_name, source),
            body: block,
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<Function<()>> for AstNode {
    fn from(value: Function<()>) -> Self {
        AstNode::Function(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FunctionParameter<T> {
    pub name: Id<T>,
    pub type_name: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::FunctionParameter> for FunctionParameter<()> {
    fn transform(item: rust_sitter::Spanned<grammar::FunctionParameter>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        FunctionParameter {
            name: Id::transform(value.ident, source),
            type_name: TypeName::transform(value.type_annotation.type_name, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<FunctionParameter<()>> for AstNode {
    fn from(value: FunctionParameter<()>) -> Self {
        AstNode::FunctionParameter(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::test_helpers::*;

    #[test]
    fn test_simple_function() {
        let result = parse_function("fn foo(): i32 {}").unwrap();
        assert_eq!(result.id.name, "foo");
        assert_eq!(result.parameters.len(), 0);
        assert!(matches!(result.return_type, TypeName::Literal(ref name, _) if name == "i32"));
        assert_eq!(result.body.statements.len(), 0);
    }

    #[test]
    fn test_function_with_single_parameter() {
        let result = parse_function("fn add(x: i32): i32 { x }").unwrap();
        assert_eq!(result.id.name, "add");
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].name.name, "x");
        assert!(
            matches!(result.parameters[0].type_name, TypeName::Literal(ref name, _) if name == "i32")
        );
        assert!(matches!(result.return_type, TypeName::Literal(ref name, _) if name == "i32"));
    }

    #[test]
    fn test_function_with_multiple_parameters() {
        let result = parse_function("fn add(x: i32, y: i32): i32 { x + y }").unwrap();
        assert_eq!(result.id.name, "add");
        assert_eq!(result.parameters.len(), 2);

        assert_eq!(result.parameters[0].name.name, "x");
        assert!(
            matches!(result.parameters[0].type_name, TypeName::Literal(ref name, _) if name == "i32")
        );

        assert_eq!(result.parameters[1].name.name, "y");
        assert!(
            matches!(result.parameters[1].type_name, TypeName::Literal(ref name, _) if name == "i32")
        );

        assert!(matches!(result.return_type, TypeName::Literal(ref name, _) if name == "i32"));
    }

    #[test]
    fn test_function_with_void_return() {
        let result = parse_function("fn main(): void {}").unwrap();
        assert_eq!(result.id.name, "main");
        assert!(matches!(result.return_type, TypeName::Literal(ref name, _) if name == "void"));
    }

    #[test]
    fn test_function_with_body_statements() {
        let result = parse_function("fn test(): void { let x: i32 = 42; }").unwrap();
        assert_eq!(result.id.name, "test");
        assert_eq!(result.body.statements.len(), 1);
        assert!(matches!(result.body.statements[0], Statement::Initialization(_)));
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        // Test that invalid function formats fail gracefully
        assert!(parse_function("fn").is_err()); // Incomplete function
        assert!(parse_function("function foo() {}").is_err()); // Wrong keyword
        assert!(parse_function("").is_err()); // Empty string
    }
}
