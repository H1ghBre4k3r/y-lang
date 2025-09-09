use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Expression, Id, Postfix},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Assignment<T> {
    pub lvalue: LValue<T>,
    pub rvalue: Expression<T>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::Assignment> for Assignment<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Assignment>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        Assignment {
            lvalue: LValue::transform(value.lvalue, source),
            rvalue: Expression::transform(value.value, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<Assignment<()>> for AstNode {
    fn from(value: Assignment<()>) -> Self {
        AstNode::Assignment(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LValue<T> {
    Id(Id<T>),
    Postfix(Postfix<T>),
}

impl FromGrammar<grammar::LValue> for LValue<()> {
    fn transform(item: rust_sitter::Spanned<grammar::LValue>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span: _ } = item;

        match value {
            grammar::LValue::Identifier(identifier) => {
                LValue::Id(Id::transform(identifier, source))
            }
            grammar::LValue::PropertyAccess(prop_access) => {
                LValue::Postfix(Postfix::transform(
                    rust_sitter::Spanned {
                        value: grammar::Postfix::PropertyAccess(prop_access.value),
                        span: prop_access.span, // Use the original span
                    },
                    source,
                ))
            }
            grammar::LValue::IndexExpression(index_expr) => {
                LValue::Postfix(Postfix::transform(
                    rust_sitter::Spanned {
                        value: grammar::Postfix::Index(index_expr.value),
                        span: index_expr.span, // Use the original span
                    },
                    source,
                ))
            }
        }
    }
}

impl<T> LValue<T>
where
    T: Clone,
{
    pub fn position(&self) -> Span {
        match self {
            LValue::Id(id) => id.position.clone(),
            LValue::Postfix(postfix) => postfix.position(),
        }
    }

    pub fn get_info(&self) -> T {
        match self {
            LValue::Id(id) => id.info.clone(),
            LValue::Postfix(postfix) => postfix.get_info(),
        }
    }

    pub fn get_original_variable_name(&self) -> Id<T> {
        let postfix = match self {
            LValue::Id(id) => return id.clone(),
            LValue::Postfix(postfix) => postfix.clone(),
        };

        Self::get_original_variable_name_from_postfix(&postfix)
    }

    pub fn get_original_variable_name_from_postfix(postfix: &Postfix<T>) -> Id<T> {
        let expr = match postfix {
            Postfix::Index { expr, .. } | Postfix::PropertyAccess { expr, .. } => expr.clone(),
            _ => unreachable!(),
        };

        match *expr {
            Expression::Id(id) => id,
            Expression::Postfix(postfix) => Self::get_original_variable_name_from_postfix(&postfix),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::{Expression, LValue, Postfix};
    use crate::parser::test_helpers::*;

    #[test]
    fn test_parse_simple_assignment() {
        let result = parse_assignment("a = 42").unwrap();

        if let LValue::Id(id) = &result.lvalue {
            assert_eq!(id.name, "a");
        } else {
            panic!("Expected Id lvalue");
        }

        assert!(matches!(
            result.rvalue,
            Expression::Num(crate::parser::ast::Num::Integer(42, (), _))
        ));
    }

    #[test]
    fn test_indexed_assign() {
        let result = parse_assignment("a[123] = 42").unwrap();

        if let LValue::Postfix(Postfix::Index { expr, index, .. }) = &result.lvalue {
            assert!(matches!(**expr, Expression::Id(ref id) if id.name == "a"));
            assert!(matches!(
                **index,
                Expression::Num(crate::parser::ast::Num::Integer(123, (), _))
            ));
        } else {
            panic!("Expected Index postfix lvalue");
        }

        assert!(matches!(
            result.rvalue,
            Expression::Num(crate::parser::ast::Num::Integer(42, (), _))
        ));
    }

    #[test]
    fn test_property_access_assign() {
        let result = parse_assignment("a.b = 42").unwrap();

        if let LValue::Postfix(Postfix::PropertyAccess { expr, property, .. }) = &result.lvalue {
            assert!(matches!(**expr, Expression::Id(ref id) if id.name == "a"));
            assert_eq!(property.name, "b");
        } else {
            panic!("Expected PropertyAccess postfix lvalue");
        }

        assert!(matches!(
            result.rvalue,
            Expression::Num(crate::parser::ast::Num::Integer(42, (), _))
        ));
    }

    #[test]
    fn test_combined_index_and_property_assign() {
        let result = parse_assignment("a[123].b = 42").unwrap();

        if let LValue::Postfix(Postfix::PropertyAccess { expr, property, .. }) = &result.lvalue {
            if let Expression::Postfix(Postfix::Index {
                expr: inner_expr,
                index,
                ..
            }) = &**expr
            {
                assert!(matches!(**inner_expr, Expression::Id(ref id) if id.name == "a"));
                assert!(matches!(
                    **index,
                    Expression::Num(crate::parser::ast::Num::Integer(123, (), _))
                ));
            } else {
                panic!("Expected Index postfix inside PropertyAccess");
            }
            assert_eq!(property.name, "b");
        } else {
            panic!("Expected PropertyAccess postfix lvalue");
        }

        assert!(matches!(
            result.rvalue,
            Expression::Num(crate::parser::ast::Num::Integer(42, (), _))
        ));
    }

    #[test]
    fn test_error_on_invalid_lvalue() {
        // This test might not work exactly the same way with rust-sitter parsing
        // but we can test that function calls can't be assigned to
        assert!(parse_assignment("a() = 42").is_err());
    }

    #[test]
    fn test_string_assignment() {
        let result = parse_assignment("name = \"hello\"").unwrap();

        if let LValue::Id(id) = &result.lvalue {
            assert_eq!(id.name, "name");
        } else {
            panic!("Expected Id lvalue");
        }

        assert!(matches!(result.rvalue, Expression::AstString(_)));
    }
}
