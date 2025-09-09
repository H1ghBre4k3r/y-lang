use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Id, TypeName},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Declaration<T> {
    pub name: Id<T>,
    pub type_name: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::Declaration> for Declaration<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Declaration>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        Declaration {
            name: Id::transform(value.name, source),
            type_name: TypeName::transform(value.type_annotation.type_name, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<Declaration<()>> for AstNode {
    fn from(value: Declaration<()>) -> Self {
        AstNode::Declaration(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::TypeName;
    use crate::parser::test_helpers::*;

    #[test]
    fn test_parse_simple_declaration() {
        let result = parse_declaration("declare foo: i32;").unwrap();

        assert_eq!(result.name.name, "foo");
        assert!(matches!(result.type_name, TypeName::Literal(ref name, _) if name == "i32"));
    }

    #[test]
    fn test_parse_tuple_declaration() {
        let result = parse_declaration("declare foo: (i32, i32);").unwrap();

        assert_eq!(result.name.name, "foo");
        if let TypeName::Tuple(types, _) = result.type_name {
            assert_eq!(types.len(), 2);
            assert!(matches!(types[0], TypeName::Literal(ref name, _) if name == "i32"));
            assert!(matches!(types[1], TypeName::Literal(ref name, _) if name == "i32"));
        } else {
            panic!("Expected tuple type");
        }
    }

    #[test]
    fn test_parse_function_declaration() {
        let result = parse_declaration("declare foo: (i32, i32) -> i32;").unwrap();

        assert_eq!(result.name.name, "foo");
        if let TypeName::Fn {
            params,
            return_type,
            ..
        } = result.type_name
        {
            assert_eq!(params.len(), 2);
            assert!(matches!(params[0], TypeName::Literal(ref name, _) if name == "i32"));
            assert!(matches!(params[1], TypeName::Literal(ref name, _) if name == "i32"));
            assert!(matches!(*return_type, TypeName::Literal(ref name, _) if name == "i32"));
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_parse_reference_declaration() {
        let result = parse_declaration("declare data: &str;").unwrap();

        assert_eq!(result.name.name, "data");
        if let TypeName::Reference(inner, _) = result.type_name {
            assert!(matches!(*inner, TypeName::Literal(ref name, _) if name == "str"));
        } else {
            panic!("Expected reference type");
        }
    }

    #[test]
    fn test_parse_array_declaration() {
        let result = parse_declaration("declare nums: &[i32];").unwrap();

        assert_eq!(result.name.name, "nums");
        if let TypeName::Array(inner, _) = result.type_name {
            assert!(matches!(*inner, TypeName::Literal(ref name, _) if name == "i32"));
        } else {
            panic!("Expected array type");
        }
    }
}
