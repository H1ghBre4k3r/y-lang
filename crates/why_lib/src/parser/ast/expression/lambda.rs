use crate::{
    grammar::{self, FromGrammar},
    lexer::{Span, Token},
    parser::{ast::AstNode, combinators::Comb, FromTokens, ParseError, ParseState},
};

use super::{Expression, Id};

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

impl FromTokens<Token> for Lambda<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::BACKSLASH
            >> Comb::LPAREN
            // parameter list (optional)
            >> (Comb::LAMBDA_PARAMETER % Comb::COMMA)
            >> Comb::RPAREN
            >> Comb::BIG_RIGHT_ARROW
            // return type
            >> Comb::EXPR;

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let mut parameters = vec![];

        while let Some(AstNode::LambdaParameter(param)) =
            result.next_if(|item| matches!(item, AstNode::LambdaParameter(_)))
        {
            parameters.push(param);
        }

        let Some(AstNode::Expression(expression)) = result.next() else {
            unreachable!()
        };

        Ok(Lambda {
            parameters,
            expression: Box::new(expression),
            info: (),
            position,
        }
        .into())
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

impl FromTokens<Token> for LambdaParameter<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;
        let matcher = Comb::ID;
        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(name)) = result.first() else {
            unreachable!()
        };

        Ok(LambdaParameter {
            name: name.clone(),
            info: (),
            position,
        }
        .into())
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
