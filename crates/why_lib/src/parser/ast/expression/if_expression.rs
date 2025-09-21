use crate::{
    grammar::{self, FromGrammar},
    lexer::Span,
    parser::ast::AstNode,
};

use super::{Block, Expression};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct If<T> {
    pub condition: Box<Expression<T>>,
    pub then_block: Block<T>,
    pub else_block: Block<T>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::IfExpression> for If<()> {
    fn transform(item: rust_sitter::Spanned<grammar::IfExpression>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        // Keep then block intact
        let then_block = Block::transform(value.then_block.value, source);

        // Keep else block intact, or create empty block if no else clause
        let else_block = if let Some(else_clause) = value.else_block {
            Block::transform(else_clause.value.block.value, source)
        } else {
            // Create empty block for missing else clause
            Block {
                statements: vec![],
                info: (),
                position: Span::new(span, source),
            }
        };

        If {
            condition: Box::new(Expression::transform(*value.condition, source)),
            then_block,
            else_block,
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
    use crate::parser::{ast::Statement, test_helpers::*};

    #[test]
    fn test_simple_if() {
        let result = parse_if("if (true) {}").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.then_block.statements.len(), 0);
        assert_eq!(result.else_block.statements.len(), 0);
    }

    #[test]
    fn test_if_with_identifier_condition() {
        let result = parse_if("if (x) {}").unwrap();
        assert!(matches!(*result.condition, Expression::Id(_)));
        assert_eq!(result.then_block.statements.len(), 0);
        assert_eq!(result.else_block.statements.len(), 0);
    }

    #[test]
    fn test_simple_if_else() {
        let result = parse_if("if (true) {} else {}").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.then_block.statements.len(), 0);
        assert_eq!(result.else_block.statements.len(), 0);
    }

    #[test]
    fn test_if_with_statements() {
        let result = parse_if("if (true) { 42; }").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.then_block.statements.len(), 1);
        assert!(matches!(
            result.then_block.statements[0],
            Statement::Expression(_)
        ));
        assert_eq!(result.else_block.statements.len(), 0);
    }

    #[test]
    fn test_if_else_with_statements() {
        let result = parse_if("if (true) { 42; } else { 1337; }").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.then_block.statements.len(), 1);
        assert!(matches!(
            result.then_block.statements[0],
            Statement::Expression(_)
        ));
        assert_eq!(result.else_block.statements.len(), 1);
        assert!(matches!(
            result.else_block.statements[0],
            Statement::Expression(_)
        ));
    }

    #[test]
    fn test_if_with_yielding_expression() {
        let result = parse_if("if (true) { 42 }").unwrap();
        // assert!(matches!(*result.condition, Expression::Bool(_)));
        assert_eq!(result.then_block.statements.len(), 1);
        assert!(matches!(
            result.then_block.statements[0],
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
