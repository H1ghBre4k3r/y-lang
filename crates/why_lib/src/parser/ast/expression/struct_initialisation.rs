use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
};

use super::{AstNode, Expression, Id};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StructInitialisation<T> {
    pub id: Id<T>,
    pub fields: Vec<StructFieldInitialisation<T>>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::StructInitialisation> for StructInitialisation<()> {
    fn transform(item: rust_sitter::Spanned<grammar::StructInitialisation>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        StructInitialisation {
            id: Id::transform(value.id, source),
            fields: value
                .fields
                .into_iter()
                .map(|field| StructFieldInitialisation::transform(field, source))
                .collect(),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<StructInitialisation<()>> for AstNode {
    fn from(value: StructInitialisation<()>) -> Self {
        Self::StructInitialisation(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StructFieldInitialisation<T> {
    pub name: Id<T>,
    pub value: Expression<T>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::StructFieldInitialisation> for StructFieldInitialisation<()> {
    fn transform(
        item: rust_sitter::Spanned<grammar::StructFieldInitialisation>,
        source: &str,
    ) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        StructFieldInitialisation {
            name: Id::transform(value.name, source),
            value: Expression::transform(value.value, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<StructFieldInitialisation<()>> for AstNode {
    fn from(value: StructFieldInitialisation<()>) -> Self {
        Self::StructFieldInitialisation(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        ast::{AstString, Num},
        test_helpers::*,
    };

    #[test]
    fn test_empty_struct_initialization() {
        let result = parse_struct_init("Point {}").unwrap();
        assert_eq!(result.id.name, "Point");
        assert_eq!(result.fields.len(), 0);
    }

    #[test]
    fn test_struct_initialization_with_one_field() {
        let result = parse_struct_init("Point { x: 42 }").unwrap();
        assert_eq!(result.id.name, "Point");
        assert_eq!(result.fields.len(), 1);
        assert_eq!(result.fields[0].name.name, "x");
        assert!(matches!(
            result.fields[0].value,
            Expression::Num(Num::Integer(42, (), _))
        ));
    }

    #[test]
    fn test_struct_initialization_with_multiple_fields() {
        let result = parse_struct_init("Point { x: 1, y: 2 }").unwrap();
        assert_eq!(result.id.name, "Point");
        assert_eq!(result.fields.len(), 2);

        assert_eq!(result.fields[0].name.name, "x");
        assert!(matches!(
            result.fields[0].value,
            Expression::Num(Num::Integer(1, (), _))
        ));

        assert_eq!(result.fields[1].name.name, "y");
        assert!(matches!(
            result.fields[1].value,
            Expression::Num(Num::Integer(2, (), _))
        ));
    }

    #[test]
    fn test_struct_initialization_with_string_field() {
        let result = parse_struct_init(r#"Person { name: "Alice" }"#).unwrap();
        assert_eq!(result.id.name, "Person");
        assert_eq!(result.fields.len(), 1);
        assert_eq!(result.fields[0].name.name, "name");
        assert!(matches!(
            result.fields[0].value,
            Expression::AstString(AstString {
                ref value, ..
            })
            if value == "Alice"
        ));
    }

    #[test]
    fn test_struct_initialization_with_mixed_fields() {
        let result = parse_struct_init(r#"Person { name: "Alice", age: 30 }"#).unwrap();
        assert_eq!(result.id.name, "Person");
        assert_eq!(result.fields.len(), 2);

        assert_eq!(result.fields[0].name.name, "name");
        assert!(matches!(
            result.fields[0].value,
            Expression::AstString(AstString {
                ref value, ..
            })
            if value == "Alice"
        ));

        assert_eq!(result.fields[1].name.name, "age");
        assert!(matches!(
            result.fields[1].value,
            Expression::Num(Num::Integer(30, (), _))
        ));
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        // Test that invalid struct initialization formats fail gracefully
        assert!(parse_struct_init("Point {").is_err()); // Unclosed struct
        assert!(parse_struct_init("Point { x }").is_err()); // Missing value
        assert!(parse_struct_init("").is_err()); // Empty string
    }
}
