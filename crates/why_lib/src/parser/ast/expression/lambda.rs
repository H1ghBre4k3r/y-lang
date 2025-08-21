use crate::{
    lexer::{Span, Token},
    parser::{ast::AstNode, direct_parsing::DirectParser, FromTokens, ParseError, ParseState},
};

use super::{Expression, Id};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Lambda<T> {
    pub parameters: Vec<LambdaParameter<T>>,
    pub expression: Box<Expression<T>>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Lambda<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        // Parse lambda syntax: \(params) => expression
        DirectParser::expect_backslash(tokens)?;
        DirectParser::expect_lparen(tokens)?;
        
        let mut parameters = vec![];
        
        // Parse optional parameters separated by commas
        if !matches!(tokens.peek(), Some(Token::RParen { .. })) {
            // Parse first parameter
            match LambdaParameter::parse(tokens)? {
                AstNode::LambdaParameter(param) => parameters.push(param),
                _ => unreachable!("LambdaParameter::parse should return LambdaParameter"),
            }
            
            // Parse additional parameters
            while DirectParser::expect_comma(tokens).is_ok() {
                match LambdaParameter::parse(tokens)? {
                    AstNode::LambdaParameter(param) => parameters.push(param),
                    _ => unreachable!("LambdaParameter::parse should return LambdaParameter"),
                }
            }
        }
        
        DirectParser::expect_rparen(tokens)?;
        DirectParser::expect_big_right_arrow(tokens)?;
        
        let expression = match Expression::parse(tokens)? {
            AstNode::Expression(expr) => expr,
            _ => unreachable!("Expression::parse should return Expression"),
        };

        Ok(Lambda {
            parameters,
            expression: Box::new(expression),
            info: (),
            position,
        }.into())
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

impl FromTokens<Token> for LambdaParameter<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;
        
        let name = match Id::parse(tokens)? {
            AstNode::Id(id) => id,
            _ => unreachable!("Id::parse should return Id"),
        };

        Ok(LambdaParameter {
            name,
            info: (),
            position,
        }.into())
    }
}

impl From<LambdaParameter<()>> for AstNode {
    fn from(value: LambdaParameter<()>) -> Self {
        AstNode::LambdaParameter(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::ast::{BinaryExpression, BinaryOperator, Block, Id, Num, Statement},
    };

    use super::*;

    #[test]
    fn test_simple_lambda() {
        let mut tokens = Lexer::new("\\() => 42")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Lambda::parse(&mut tokens);

        assert_eq!(
            Ok(Lambda {
                parameters: vec![],
                expression: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_lambda_with_multiple_params() {
        let mut tokens = Lexer::new("\\(x, y) => x + y")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Lambda::parse(&mut tokens);

        assert_eq!(
            Ok(Lambda {
                parameters: vec![
                    LambdaParameter {
                        name: Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        },
                        info: (),
                        position: Span::default()
                    },
                    LambdaParameter {
                        name: Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        },
                        info: (),
                        position: Span::default()
                    }
                ],
                expression: Box::new(Expression::Binary(Box::new(BinaryExpression {
                    left: Expression::Id(Id {
                        name: "x".into(),
                        info: (),
                        position: Span::default()
                    }),
                    right: Expression::Id(Id {
                        name: "y".into(),
                        info: (),
                        position: Span::default()
                    }),
                    operator: BinaryOperator::Add,
                    info: (),
                    position: Span::default()
                }))),
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_lambda_with_single_param() {
        let mut tokens = Lexer::new("\\(x) => x")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Lambda::parse(&mut tokens);

        assert_eq!(
            Ok(Lambda {
                parameters: vec![LambdaParameter {
                    name: Id {
                        name: "x".into(),
                        info: (),
                        position: Span::default()
                    },
                    info: (),
                    position: Span::default()
                }],
                expression: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                })),
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_lambda_with_block() {
        let mut tokens = Lexer::new("\\(x) => { x }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Lambda::parse(&mut tokens);

        assert_eq!(
            Ok(Lambda {
                parameters: vec![LambdaParameter {
                    name: Id {
                        name: "x".into(),
                        info: (),
                        position: Span::default()
                    },
                    info: (),
                    position: Span::default()
                }],
                expression: Box::new(Expression::Block(Block {
                    statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                        name: "x".into(),
                        info: (),
                        position: Span::default()
                    }))],
                    info: (),
                    position: Span::default()
                })),
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }
}
