use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::{AstNode, Statement},
};

use super::{Block, Expression};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct If<T> {
    pub condition: Box<Expression<T>>,
    // TODO: This should/could just be a block
    pub statements: Vec<Statement<T>>,
    // TODO: This should/could just be a block
    pub else_statements: Vec<Statement<T>>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::IfExpression> for If<()> {
    fn transform(item: rust_sitter::Spanned<grammar::IfExpression>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        // Extract then block statements
        let then_block = Block::transform(value.then_block.value, source);

        // Extract else block statements if present
        let else_statements = if let Some(else_clause) = value.else_block {
            let else_block = Block::transform(else_clause.value.block.value, source);
            else_block.statements
        } else {
            vec![]
        };

        If {
            condition: Box::new(Expression::transform(*value.condition, source)),
            statements: then_block.statements,
            else_statements,
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<If<()>> for AstNode {
    fn from(value: If<()>) -> Self {
        AstNode::If(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::test_helpers::*;

    #[test]
    fn test_simple_if() {
        let result = parse_if("if (true) {}").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.statements.len(), 0);
        assert_eq!(result.else_statements.len(), 0);
    }

    #[test]
    fn test_if_with_identifier_condition() {
        let result = parse_if("if (x) {}").unwrap();
        assert!(matches!(*result.condition, Expression::Id(_)));
        assert_eq!(result.statements.len(), 0);
        assert_eq!(result.else_statements.len(), 0);
    }

    #[test]
    fn test_simple_if_else() {
        let result = parse_if("if (true) {} else {}").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.statements.len(), 0);
        assert_eq!(result.else_statements.len(), 0);
    }

    #[test]
    fn test_if_with_statements() {
        let result = parse_if("if (true) { 42; }").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.statements.len(), 1);
        assert!(matches!(result.statements[0], Statement::Expression(_)));
        assert_eq!(result.else_statements.len(), 0);
    }

    #[test]
    fn test_if_else_with_statements() {
        let result = parse_if("if (true) { 42; } else { 1337; }").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.statements.len(), 1);
        assert!(matches!(result.statements[0], Statement::Expression(_)));
        assert_eq!(result.else_statements.len(), 1);
        assert!(matches!(
            result.else_statements[0],
            Statement::Expression(_)
        ));
    }

    #[test]
    fn test_if_with_yielding_expression() {
        let result = parse_if("if (true) { 42 }").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.statements.len(), 1);
        assert!(matches!(
            result.statements[0],
            Statement::YieldingExpression(_)
        ));
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        // Test that invalid if formats fail gracefully
        assert!(parse_if("if").is_err()); // Incomplete if
        assert!(parse_if("if true {}").is_err()); // Missing parentheses
        assert!(parse_if("").is_err()); // Empty string
    }
}
