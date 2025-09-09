use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
};

use super::{AstNode, Expression, Id};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Lambda<T> {
    pub parameters: Vec<LambdaParameter<T>>,
    pub expression: Box<Expression<T>>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::Lambda> for Lambda<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Lambda>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        // Convert parameters
        let parameters = value
            .params
            .into_iter()
            .map(|param| LambdaParameter::transform(param, source))
            .collect();

        let expression = Box::new(Expression::transform(*value.expression, source));

        Lambda {
            parameters,
            expression,
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<Lambda<()>> for AstNode {
    fn from(value: Lambda<()>) -> Self {
        AstNode::Lambda(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LambdaParameter<T> {
    pub name: Id<T>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::LambdaParameter> for LambdaParameter<()> {
    fn transform(item: rust_sitter::Spanned<grammar::LambdaParameter>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        LambdaParameter {
            name: Id::transform(value.ident, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<LambdaParameter<()>> for AstNode {
    fn from(value: LambdaParameter<()>) -> Self {
        AstNode::LambdaParameter(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::{BinaryOperator, Expression, Statement};
    use crate::parser::test_helpers::*;

    #[test]
    fn test_simple_lambda() {
        let result = parse_lambda("\\() => 42").unwrap();

        assert_eq!(result.parameters.len(), 0);
        assert!(matches!(
            &*result.expression,
            Expression::Num(crate::parser::ast::Num::Integer(42, (), _))
        ));
    }

    #[test]
    fn test_lambda_with_multiple_params() {
        let result = parse_lambda("\\(x, y) => x + y").unwrap();

        assert_eq!(result.parameters.len(), 2);
        assert_eq!(result.parameters[0].name.name, "x");
        assert_eq!(result.parameters[1].name.name, "y");

        if let Expression::Binary(binary) = &*result.expression {
            assert!(matches!(binary.operator, BinaryOperator::Add));
            assert!(matches!(&binary.left, Expression::Id(id) if id.name == "x"));
            assert!(matches!(&binary.right, Expression::Id(id) if id.name == "y"));
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_lambda_with_single_param() {
        let result = parse_lambda("\\(x) => x").unwrap();

        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].name.name, "x");
        assert!(matches!(&*result.expression, Expression::Id(id) if id.name == "x"));
    }

    #[test]
    fn test_lambda_with_block() {
        let result = parse_lambda("\\(x) => { x }").unwrap();

        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].name.name, "x");

        if let Expression::Block(block) = &*result.expression {
            assert_eq!(block.statements.len(), 1);
            assert!(matches!(
                block.statements[0],
                Statement::YieldingExpression(Expression::Id(ref id)) if id.name == "x"
            ));
        } else {
            panic!("Expected block expression");
        }
    }

    #[test]
    fn test_complex_lambda_expression() {
        let result = parse_lambda("\\(a, b) => a * b + 1").unwrap();

        assert_eq!(result.parameters.len(), 2);
        assert_eq!(result.parameters[0].name.name, "a");
        assert_eq!(result.parameters[1].name.name, "b");

        // Should be a + (b * c) due to operator precedence
        if let Expression::Binary(binary) = &*result.expression {
            assert!(matches!(binary.operator, BinaryOperator::Add));
            // Left side should be a * b
            if let Expression::Binary(left_binary) = &binary.left {
                assert!(matches!(left_binary.operator, BinaryOperator::Multiply));
            }
            // Right side should be 1
            assert!(matches!(
                &binary.right,
                Expression::Num(crate::parser::ast::Num::Integer(1, (), _))
            ));
        } else {
            panic!("Expected binary expression");
        }
    }
}
