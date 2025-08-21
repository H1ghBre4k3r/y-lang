use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Statement},
        direct_parsing::DirectParser,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Block<T> {
    pub statements: Vec<Statement<T>>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Block<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        // Parse opening brace
        DirectParser::expect_lbrace(tokens)?;
        
        let mut statements = vec![];
        
        // Parse statements until we hit the closing brace
        loop {
            // Check if we hit the closing brace
            if DirectParser::parse_terminal(tokens, |t| matches!(t, Token::RBrace { .. }), "right brace").is_ok() {
                break;
            }
            
            // Try to parse a statement
            match Statement::parse(tokens) {
                Ok(AstNode::Statement(statement)) => {
                    // Check if this is a yielding expression not at the end
                    if let Statement::YieldingExpression(exp) = &statement {
                        // Peek to see if there are more statements coming
                        if !matches!(tokens.peek(), Some(Token::RBrace { .. }) | None) {
                            let err = ParseError {
                                position: Some(exp.position()),
                                message: "A YieldingExpression is only allowed at the end of a block".into(),
                            };
                            tokens.add_error(err.clone());
                            return Err(err);
                        }
                    }
                    statements.push(statement);
                }
                Ok(_) => unreachable!("Statement::parse should return Statement"),
                Err(e) => {
                    // If we failed to parse a statement and we're not at a closing brace,
                    // this is an error
                    if !matches!(tokens.peek(), Some(Token::RBrace { .. })) {
                        return Err(e);
                    }
                    // Otherwise, we might be at the end of the block
                    break;
                }
            }
        }
        
        // Make sure we consumed the closing brace (it should have been consumed in the loop)
        // If we exited the loop without consuming it, consume it now
        if matches!(tokens.peek(), Some(Token::RBrace { .. })) {
            DirectParser::expect_rbrace(tokens)?;
        }

        Ok(Block {
            statements,
            info: (),
            position,
        }.into())
    }
}

impl From<Block<()>> for AstNode {
    fn from(value: Block<()>) -> Self {
        AstNode::Block(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::ast::{Expression, Id, Initialisation, Num},
    };

    use super::*;

    #[test]
    fn test_empty_block() {
        let mut tokens = Lexer::new("{ }").lex().expect("something is wrong").into();

        let result = Block::parse(&mut tokens);

        assert_eq!(
            Ok(Block {
                statements: vec![],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_simple_block() {
        let mut tokens = Lexer::new("{ x }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Block::parse(&mut tokens);

        assert_eq!(
            Ok(Block {
                statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                }))],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_complex_block() {
        let mut tokens = Lexer::new(
            "{
                let a = 42;
                a
            }",
        )
        .lex()
        .expect("something is wrong")
        .into();

        let result = Block::parse(&mut tokens);

        assert_eq!(
            Ok(Block {
                statements: vec![
                    Statement::Initialization(Initialisation {
                        id: Id {
                            name: "a".into(),
                            info: (),
                            position: Span::default()
                        },
                        mutable: false,
                        value: Expression::Num(Num::Integer(42, (), Span::default())),
                        type_name: None,
                        info: (),
                        position: Span::default()
                    }),
                    Statement::YieldingExpression(Expression::Id(Id {
                        name: "a".into(),
                        info: (),
                        position: Span::default()
                    }))
                ],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_error_with_yielding_expression_not_at_end() {
        let mut tokens = Lexer::new(
            "{
                42
                42
            }",
        )
        .lex()
        .expect("something is wrong")
        .into();

        assert!(Block::parse(&mut tokens).is_err());
    }
}
