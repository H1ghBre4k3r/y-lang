use crate::{
    lexer::{Token, Tokens},
    parser::{ast::AstNode, combinators::Comb, FromTokens, ParseError},
};

use super::{Expression, Parameter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lambda<T> {
    pub parameters: Vec<Parameter<T>>,
    pub expression: Box<Expression<T>>,
    pub info: T,
}

impl FromTokens<Token> for Lambda<()> {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::BACKSLASH
            >> Comb::LPAREN
            // parameter list (optional)
            >> (Comb::PARAMETER % Comb::COMMA)
            >> Comb::RPAREN
            >> Comb::BIG_RIGHT_ARROW
            // return type
            >> Comb::EXPR;

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let mut parameters = vec![];

        while let Some(AstNode::Parameter(param)) =
            result.next_if(|item| matches!(item, AstNode::Parameter(_)))
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
        }
        .into())
    }
}

impl From<Lambda<()>> for AstNode {
    fn from(value: Lambda<()>) -> Self {
        AstNode::Lambda(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::ast::{BinaryExpression, Block, Id, Num, Statement},
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
                expression: Box::new(Expression::Num(Num::Integer(42, ()))),
                info: ()
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
                    Parameter {
                        name: Id {
                            name: "x".into(),
                            info: ()
                        },
                        type_name: None,
                        info: ()
                    },
                    Parameter {
                        name: Id {
                            name: "y".into(),
                            info: ()
                        },
                        type_name: None,
                        info: ()
                    }
                ],
                expression: Box::new(Expression::Binary(Box::new(BinaryExpression::Addition {
                    left: Expression::Id(Id {
                        name: "x".into(),
                        info: ()
                    }),
                    right: Expression::Id(Id {
                        name: "y".into(),
                        info: ()
                    }),
                    info: (),
                }))),
                info: ()
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
                parameters: vec![Parameter {
                    name: Id {
                        name: "x".into(),
                        info: ()
                    },
                    type_name: None,
                    info: ()
                }],
                expression: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: ()
                })),
                info: ()
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
                parameters: vec![Parameter {
                    name: Id {
                        name: "x".into(),
                        info: ()
                    },
                    type_name: None,
                    info: ()
                }],
                expression: Box::new(Expression::Block(Block {
                    statements: vec![Statement::YieldingExpression(Expression::Id(Id {
                        name: "x".into(),
                        info: ()
                    }))],
                    info: ()
                })),
                info: ()
            }
            .into()),
            result
        )
    }
}
