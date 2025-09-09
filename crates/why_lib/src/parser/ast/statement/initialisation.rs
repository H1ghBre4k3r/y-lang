use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Expression, Id, TypeName},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Initialisation<T> {
    pub id: Id<T>,
    pub mutable: bool,
    pub type_name: Option<TypeName>,
    pub value: Expression<T>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::VariableDeclaration> for Initialisation<()> {
    fn transform(item: rust_sitter::Spanned<grammar::VariableDeclaration>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        Initialisation {
            id: Id::transform(value.identifier, source),
            mutable: value.mutability.is_some(),
            type_name: value
                .type_annotation
                .map(|ta| TypeName::transform(ta.type_name, source)),
            value: Expression::transform(value.value, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<Initialisation<()>> for AstNode {
    fn from(value: Initialisation<()>) -> Self {
        AstNode::Initialization(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::{Expression, TypeName};
    use crate::parser::test_helpers::*;

    #[test]
    fn test_simple_initialisation() {
        let result = parse_initialization("let foo = 42;").unwrap();

        assert_eq!(result.id.name, "foo");
        assert!(!result.mutable);
        assert_eq!(result.type_name, None);
        assert!(matches!(
            result.value,
            Expression::Num(crate::parser::ast::Num::Integer(42, (), _))
        ));
    }

    #[test]
    fn test_initialisation_with_typename() {
        let result = parse_initialization("let foo: i32 = 42;").unwrap();

        assert_eq!(result.id.name, "foo");
        assert!(!result.mutable);
        assert!(matches!(result.type_name, Some(TypeName::Literal(ref name, _)) if name == "i32"));
        assert!(matches!(
            result.value,
            Expression::Num(crate::parser::ast::Num::Integer(42, (), _))
        ));
    }

    #[test]
    fn test_mutable_initialisation() {
        let result = parse_initialization("let mut foo = 42;").unwrap();

        assert_eq!(result.id.name, "foo");
        assert!(result.mutable);
        assert_eq!(result.type_name, None);
        assert!(matches!(
            result.value,
            Expression::Num(crate::parser::ast::Num::Integer(42, (), _))
        ));
    }

    #[test]
    fn test_string_initialisation() {
        let result = parse_initialization("let message = \"hello\";").unwrap();

        assert_eq!(result.id.name, "message");
        assert!(!result.mutable);
        assert_eq!(result.type_name, None);
        assert!(matches!(result.value, Expression::AstString(_)));
    }

    #[test]
    fn test_expression_initialisation() {
        let result = parse_initialization("let sum = x + y;").unwrap();

        assert_eq!(result.id.name, "sum");
        assert!(!result.mutable);
        assert_eq!(result.type_name, None);
        assert!(matches!(result.value, Expression::Binary(_)));
    }

    #[test]
    fn test_typed_mutable_initialisation() {
        let result = parse_initialization("let mut counter: i32 = 0;").unwrap();

        assert_eq!(result.id.name, "counter");
        assert!(result.mutable);
        assert!(matches!(result.type_name, Some(TypeName::Literal(ref name, _)) if name == "i32"));
        assert!(matches!(
            result.value,
            Expression::Num(crate::parser::ast::Num::Integer(0, (), _))
        ));
    }
}
