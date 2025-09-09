use std::fmt::Display;

use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
};

use super::{AstNode, Id};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TypeName {
    Literal(String, Span),
    Fn {
        params: Vec<TypeName>,
        return_type: Box<TypeName>,
        position: Span,
    },
    Tuple(Vec<TypeName>, Span),
    Array(Box<TypeName>, Span),
    Reference(Box<TypeName>, Span),
}

impl TypeName {
    pub fn position(&self) -> Span {
        match self {
            TypeName::Literal(_, position) => position.clone(),
            TypeName::Fn { position, .. } => position.clone(),
            TypeName::Tuple(_, position) => position.clone(),
            TypeName::Array(_, position) => position.clone(),
            TypeName::Reference(_, position) => position.clone(),
        }
    }
}

impl Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeName::Literal(lit, _) => f.write_str(lit.as_str()),
            TypeName::Fn {
                params,
                return_type,
                ..
            } => f.write_fmt(format_args!(
                "({}) -> {return_type}",
                params
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            TypeName::Tuple(lits, _) => f.write_fmt(format_args!(
                "({})",
                lits.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            TypeName::Array(el, _) => f.write_fmt(format_args!("[{el}]")),
            TypeName::Reference(el, _) => f.write_fmt(format_args!("&{el}")),
        }
    }
}

impl From<&TypeName> for TypeName {
    fn from(value: &TypeName) -> Self {
        value.clone()
    }
}

impl FromGrammar<grammar::TypeName> for TypeName {
    fn transform(item: rust_sitter::Spanned<grammar::TypeName>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        match value {
            grammar::TypeName::LiteralType(literal_type) => {
                let id = Id::transform(literal_type.typename, source);
                TypeName::Literal(id.name, Span::new(span, source))
            }
            grammar::TypeName::ArrayType(array_type) => TypeName::Array(
                Box::new(TypeName::transform(*array_type.inner, source)),
                Span::new(span, source),
            ),
            grammar::TypeName::ReferenceType(reference_type) => TypeName::Reference(
                Box::new(TypeName::transform(*reference_type.inner, source)),
                Span::new(span, source),
            ),
            grammar::TypeName::FunctionType(function_type) => TypeName::Fn {
                params: function_type
                    .params
                    .types
                    .into_iter()
                    .map(|param| TypeName::transform(param, source))
                    .collect(),
                return_type: Box::new(TypeName::transform(*function_type.return_type, source)),
                position: Span::new(span, source),
            },
            grammar::TypeName::TupleType(tuple_type) => TypeName::Tuple(
                tuple_type
                    .types
                    .into_iter()
                    .map(|t| TypeName::transform(t, source))
                    .collect(),
                Span::new(span, source),
            ),
        }
    }
}

impl From<TypeName> for AstNode {
    fn from(value: TypeName) -> Self {
        Self::TypeName(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_helpers::*;

    use super::TypeName;

    #[test]
    fn test_parse_simple_literal() {
        let result = parse_type_name("i32").unwrap();
        assert!(matches!(result, TypeName::Literal(ref name, _) if name == "i32"));
    }

    #[test]
    fn test_parse_simple_tuple() {
        let result = parse_type_name("(i32, i32)").unwrap();
        if let TypeName::Tuple(types, _) = result {
            assert_eq!(types.len(), 2);
            assert!(matches!(types[0], TypeName::Literal(ref name, _) if name == "i32"));
            assert!(matches!(types[1], TypeName::Literal(ref name, _) if name == "i32"));
        } else {
            panic!("Expected tuple type");
        }
    }

    #[test]
    fn test_parse_simple_function() {
        let result = parse_type_name("() -> i32").unwrap();
        if let TypeName::Fn {
            params,
            return_type,
            ..
        } = result
        {
            assert_eq!(params.len(), 0);
            assert!(matches!(*return_type, TypeName::Literal(ref name, _) if name == "i32"));
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_parse_simple_reference() {
        let result = parse_type_name("&i32").unwrap();
        if let TypeName::Reference(inner, _) = result {
            assert!(matches!(*inner, TypeName::Literal(ref name, _) if name == "i32"));
        } else {
            panic!("Expected reference type");
        }
    }

    #[test]
    fn test_parse_reference_of_tuple() {
        let result = parse_type_name("&(i32, i32)").unwrap();
        if let TypeName::Reference(inner, _) = result {
            if let TypeName::Tuple(types, _) = inner.as_ref() {
                assert_eq!(types.len(), 2);
                assert!(matches!(types[0], TypeName::Literal(ref name, _) if name == "i32"));
                assert!(matches!(types[1], TypeName::Literal(ref name, _) if name == "i32"));
            } else {
                panic!("Expected tuple inside reference");
            }
        } else {
            panic!("Expected reference type");
        }
    }

    #[test]
    fn test_parse_tuple_of_references() {
        let result = parse_type_name("(&i32, &i32)").unwrap();
        if let TypeName::Tuple(types, _) = result {
            assert_eq!(types.len(), 2);
            for type_ref in types {
                if let TypeName::Reference(inner, _) = type_ref {
                    assert!(matches!(*inner, TypeName::Literal(ref name, _) if name == "i32"));
                } else {
                    panic!("Expected reference type in tuple");
                }
            }
        } else {
            panic!("Expected tuple type");
        }
    }

    #[test]
    fn test_parse_array_type() {
        let result = parse_type_name("&[i32]").unwrap();
        if let TypeName::Array(inner, _) = result {
            assert!(matches!(*inner, TypeName::Literal(ref name, _) if name == "i32"));
        } else {
            panic!("Expected array type");
        }
    }

    #[test]
    fn test_parse_complex_function_type() {
        let result = parse_type_name("(i32, &str) -> bool").unwrap();
        if let TypeName::Fn {
            params,
            return_type,
            ..
        } = result
        {
            assert_eq!(params.len(), 2);
            assert!(matches!(params[0], TypeName::Literal(ref name, _) if name == "i32"));
            assert!(
                matches!(params[1], TypeName::Reference(ref inner, _) if matches!(**inner, TypeName::Literal(ref name, _) if name == "str"))
            );
            assert!(matches!(*return_type, TypeName::Literal(ref name, _) if name == "bool"));
        } else {
            panic!("Expected function type");
        }
    }
}
