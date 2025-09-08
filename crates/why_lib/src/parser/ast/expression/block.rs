use crate::{
    grammar::{self, FromGrammar},
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Statement},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Block<T> {
    pub statements: Vec<Statement<T>>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::Block> for Block<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Block>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        Block {
            statements: value
                .statements
                .into_iter()
                .map(|statement| Statement::transform(statement, source))
                .collect(),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl FromTokens<Token> for Block<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::LBRACE >> (Comb::STATEMENT ^ Comb::RBRACE);

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let mut statements = vec![];

        while let Some(AstNode::Statement(statement)) = result.next() {
            statements.push(statement.clone());
            if let Statement::YieldingExpression(exp) = statement {
                if result.peek().is_some() {
                    let err = ParseError {
                        position: Some(exp.position()),
                        message: "A YieldingExpression is only allowed at the end of a block"
                            .into(),
                    };
                    tokens.add_error(err.clone());
                    return Err(err);
                }
            }
        }

        Ok(Block {
            statements,
            info: (),
            position,
        }
        .into())
    }
}

impl From<Block<()>> for AstNode {
    fn from(value: Block<()>) -> Self {
        AstNode::Block(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        ast::{Expression, Num},
        test_helpers::*,
    };

    #[test]
    fn test_empty_block() {
        let result = parse_block("{}").unwrap();
        assert_eq!(result.statements.len(), 0);
    }

    #[test]
    fn test_block_with_single_statement() {
        let result = parse_block("{ 42; }").unwrap();
        assert_eq!(result.statements.len(), 1);
        assert!(matches!(
            result.statements[0],
            Statement::Expression(Expression::Num(Num::Integer(42, (), _)))
        ));
    }

    #[test]
    fn test_block_with_yielding_expression() {
        let result = parse_block("{ 42 }").unwrap();
        assert_eq!(result.statements.len(), 1);
        assert!(matches!(
            result.statements[0],
            Statement::YieldingExpression(Expression::Num(Num::Integer(42, (), _)))
        ));
    }

    #[test]
    fn test_block_with_variable_declaration() {
        let result = parse_block("{ let x: i32 = 42; }").unwrap();
        assert_eq!(result.statements.len(), 1);
        assert!(matches!(result.statements[0], Statement::Initialization(_)));
    }

    #[test]
    fn test_complex_block() {
        let result = parse_block("{ let a: i32 = 42; a }").unwrap();
        assert_eq!(result.statements.len(), 2);
        assert!(matches!(result.statements[0], Statement::Initialization(_)));
        assert!(matches!(
            result.statements[1],
            Statement::YieldingExpression(_)
        ));
    }

    #[test]
    fn test_block_with_multiple_statements() {
        let result = parse_block("{ let x: i32 = 1; let y: i32 = 2; x + y }").unwrap();
        assert_eq!(result.statements.len(), 3);
        assert!(matches!(result.statements[0], Statement::Initialization(_)));
        assert!(matches!(result.statements[1], Statement::Initialization(_)));
        assert!(matches!(
            result.statements[2],
            Statement::YieldingExpression(_)
        ));
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        // Test that invalid block formats fail gracefully
        assert!(parse_block("{").is_err()); // Unclosed block
        assert!(parse_block("").is_err()); // Empty string
    }
}
