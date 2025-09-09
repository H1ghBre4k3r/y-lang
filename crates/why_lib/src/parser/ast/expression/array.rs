use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
};

use super::{AstNode, Expression, Num};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Array<T> {
    Literal {
        values: Vec<Expression<T>>,
        info: T,
        position: Span,
    },
    Default {
        initial_value: Box<Expression<T>>,
        length: Num<T>,
        info: T,
        position: Span,
    },
}

impl<T> Array<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Array::Literal { info, .. } => info.clone(),
            Array::Default { info, .. } => info.clone(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Array::Literal { position, .. } => position.clone(),
            Array::Default { position, .. } => position.clone(),
        }
    }
}

impl FromGrammar<grammar::Array> for Array<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Array>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        // Convert each element from grammar::Expression to Expression<()>
        // Note: elements are not wrapped in Spanned, so we create our own with the overall span
        let values = value
            .elements
            .into_iter()
            .map(|expr| Expression::transform(expr, source))
            .collect();

        Array::Literal {
            values,
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<Array<()>> for AstNode {
    fn from(value: Array<()>) -> Self {
        Self::Array(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::test_helpers::*;

    #[test]
    fn test_empty_array() {
        let result = parse_array("&[]").unwrap();
        match result {
            Array::Literal { values, .. } => {
                assert_eq!(values.len(), 0);
            }
            _ => panic!("Expected literal array"),
        }
    }

    #[test]
    fn test_simple_literal() {
        let result = parse_array("&[42, 1337]").unwrap();
        match result {
            Array::Literal { values, .. } => {
                assert_eq!(values.len(), 2);
                assert!(matches!(
                    values[0],
                    Expression::Num(Num::Integer(42, (), _))
                ));
                assert!(matches!(
                    values[1],
                    Expression::Num(Num::Integer(1337, (), _))
                ));
            }
            _ => panic!("Expected literal array"),
        }
    }

    #[test]
    fn test_single_element_array() {
        let result = parse_array("&[42]").unwrap();
        match result {
            Array::Literal { values, .. } => {
                assert_eq!(values.len(), 1);
                assert!(matches!(
                    values[0],
                    Expression::Num(Num::Integer(42, (), _))
                ));
            }
            _ => panic!("Expected literal array"),
        }
    }

    #[test]
    fn test_mixed_expression_array() {
        let result = parse_array(r#"&[42, "hello", x]"#).unwrap();
        match result {
            Array::Literal { values, .. } => {
                assert_eq!(values.len(), 3);
                assert!(matches!(
                    values[0],
                    Expression::Num(Num::Integer(42, (), _))
                ));
                assert!(matches!(values[1], Expression::AstString(_)));
                assert!(matches!(values[2], Expression::Id(_)));
            }
            _ => panic!("Expected literal array"),
        }
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        // Test that invalid array formats fail gracefully
        assert!(parse_array("&[").is_err()); // Unclosed array
        assert!(parse_array("42, 43]").is_err()); // Missing open bracket
        assert!(parse_array("").is_err()); // Empty string
    }
}
