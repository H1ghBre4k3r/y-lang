mod array;
mod binary;
mod block;
mod function;
mod id;
mod if_expression;
mod lambda;
mod num;
mod postfix;
mod prefix;
mod struct_initialisation;

pub use self::array::*;
pub use self::binary::*;
pub use self::block::*;
pub use self::function::*;
pub use self::id::*;
pub use self::if_expression::*;
pub use self::lambda::*;
pub use self::num::*;
pub use self::postfix::*;
pub use self::prefix::*;
pub use self::struct_initialisation::*;

use crate::parser::combinators::Comb;
use crate::{
    lexer::Token,
    parser::{FromTokens, ParseError, ParseState},
};

use super::AstNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression<T> {
    Id(Id<T>),
    Num(Num<T>),
    Function(Function<T>),
    Lambda(Lambda<T>),
    If(If<T>),
    Block(Block<T>),
    Parens(Box<Expression<T>>),
    Postfix(Postfix<T>),
    Prefix(Prefix<T>),
    Binary(Box<BinaryExpression<T>>),
    Array(Array<T>),
    StructInitialisation(StructInitialisation<T>),
}

impl<T> Expression<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Expression::Id(Id { name: _, info, .. }) => info.clone(),
            Expression::Num(num) => num.get_info(),
            Expression::Function(Function { info, .. }) => info.clone(),
            Expression::Lambda(Lambda { info, .. }) => info.clone(),
            Expression::If(If { info, .. }) => info.clone(),
            Expression::Block(Block { info, .. }) => info.clone(),
            Expression::Parens(expr) => expr.get_info(),
            Expression::Postfix(postfix) => postfix.get_info(),
            Expression::Prefix(prefix) => prefix.get_info(),
            Expression::Binary(binary) => binary.get_info(),
            Expression::Array(arr) => arr.get_info(),
            Expression::StructInitialisation(StructInitialisation { info, .. }) => info.clone(),
        }
    }
}

impl FromTokens<Token> for Expression<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let mut expr = match tokens.peek() {
            Some(Token::LParen { .. }) => {
                let matcher = Comb::LPAREN >> Comb::EXPR >> Comb::RPAREN;
                let result = matcher.parse(tokens)?;
                let expr = match result.first() {
                    Some(AstNode::Expression(rhs)) => rhs.clone(),
                    None | Some(_) => unreachable!(),
                };
                Expression::Parens(Box::new(expr))
            }
            Some(Token::Minus { .. }) => {
                let matcher = Comb::MINUS >> Comb::EXPR;
                let result = matcher.parse(tokens)?;

                let Some(AstNode::Expression(expr)) = result.first() else {
                    unreachable!();
                };

                Expression::Prefix(Prefix::Minus {
                    expr: Box::new(expr.clone()),
                })
            }
            Some(Token::ExclamationMark { .. }) => {
                let matcher = Comb::EXCLAMATION_MARK >> Comb::EXPR;
                let result = matcher.parse(tokens)?;

                let Some(AstNode::Expression(expr)) = result.first() else {
                    unreachable!();
                };

                Expression::Prefix(Prefix::Negation {
                    expr: Box::new(expr.clone()),
                })
            }
            Some(Token::LBrace { .. }) => {
                let matcher = Comb::BLOCK;
                let result = matcher.parse(tokens)?;

                let Some(AstNode::Block(expr)) = result.first() else {
                    unreachable!();
                };

                Expression::Block(expr.clone())
            }
            _ => {
                let matcher = Comb::FUNCTION
                    | Comb::IF
                    | Comb::NUM
                    | Comb::STRUCT_INITILISATION
                    | Comb::ID
                    | Comb::LAMBDA
                    | Comb::ARRAY;
                let result = matcher.parse(tokens)?;
                match result.first() {
                    Some(AstNode::Id(id)) => Expression::Id(id.clone()),
                    Some(AstNode::Num(num)) => Expression::Num(num.clone()),
                    Some(AstNode::Function(func)) => {
                        return Ok(Expression::Function(func.clone()).into())
                    }
                    Some(AstNode::Lambda(lambda)) => {
                        return Ok(Expression::Lambda(lambda.clone()).into())
                    }
                    Some(AstNode::If(if_expression)) => Expression::If(if_expression.clone()),
                    Some(AstNode::Block(block)) => Expression::Block(block.clone()),
                    Some(AstNode::Array(array)) => Expression::Array(array.clone()),
                    Some(AstNode::StructInitialisation(initialisation)) => {
                        Expression::StructInitialisation(initialisation.clone())
                    }
                    None | Some(_) => unreachable!(),
                }
            }
        };

        loop {
            let Some(next) = tokens.peek() else {
                return Ok(expr.into());
            };

            match next {
                Token::LParen { .. } => {
                    expr = Expression::Postfix(Self::parse_call(expr, tokens)?);
                    continue;
                }
                Token::LBracket { .. } => {
                    expr = Expression::Postfix(Self::parse_index(expr, tokens)?);
                    continue;
                }
                Token::Dot { .. } => {
                    expr = Expression::Postfix(Self::parse_property_access(expr, tokens)?);
                    continue;
                }
                Token::Plus { .. }
                | Token::Minus { .. }
                | Token::Times { .. }
                | Token::Equal { .. }
                | Token::GreaterThan { .. }
                | Token::LessThan { .. }
                | Token::GreaterOrEqual { .. }
                | Token::LessOrEqual { .. } => {
                    return Ok(Self::parse_binary(expr, tokens)?.into());
                }
                _ => return Ok(expr.into()),
            };
        }
    }
}

impl Expression<()> {
    fn parse_call(
        expr: Expression<()>,
        tokens: &mut ParseState<Token>,
    ) -> Result<Postfix<()>, ParseError> {
        let matcher = Comb::LPAREN >> (Comb::EXPR % Comb::COMMA) >> Comb::RPAREN;

        let result = matcher.parse(tokens)?.into_iter();

        let mut args = vec![];

        for arg in result {
            let AstNode::Expression(arg) = arg else {
                unreachable!()
            };

            args.push(arg);
        }

        Ok(Postfix::Call {
            expr: Box::new(expr),
            args,
            info: (),
        })
    }

    fn parse_index(
        expr: Expression<()>,
        tokens: &mut ParseState<Token>,
    ) -> Result<Postfix<()>, ParseError> {
        let matcher = Comb::LBRACKET >> Comb::EXPR >> Comb::RBRACKET;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Expression(index)) = result.first().cloned() else {
            unreachable!()
        };

        Ok(Postfix::Index {
            expr: Box::new(expr),
            index: Box::new(index),
            info: (),
        })
    }

    fn parse_property_access(
        expr: Expression<()>,
        tokens: &mut ParseState<Token>,
    ) -> Result<Postfix<()>, ParseError> {
        let matcher = Comb::DOT >> Comb::ID;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(property)) = result.first().cloned() else {
            unreachable!()
        };

        Ok(Postfix::PropertyAccess {
            expr: Box::new(expr),
            property,
            info: (),
        })
    }

    fn parse_binary(
        lhs: Expression<()>,
        tokens: &mut ParseState<Token>,
    ) -> Result<Expression<()>, ParseError> {
        let Some(operation) = tokens.next() else {
            unreachable!()
        };

        let matcher = Comb::EXPR;
        let result = matcher.parse(tokens)?;
        let rhs = match result.first() {
            Some(AstNode::Expression(rhs)) => rhs.clone(),
            None | Some(_) => unreachable!(),
        };

        let binary = match operation {
            Token::Plus { .. } => BinaryExpression::Addition {
                left: lhs,
                right: rhs,

                info: (),
            },
            Token::Minus { .. } => BinaryExpression::Substraction {
                left: lhs,
                right: rhs,
                info: (),
            },
            Token::Times { .. } => BinaryExpression::Multiplication {
                left: lhs,
                right: rhs,
                info: (),
            },
            Token::Equal { .. } => BinaryExpression::Equal {
                left: lhs,
                right: rhs,
                info: (),
            },
            Token::GreaterThan { .. } => BinaryExpression::GreaterThan {
                left: lhs,
                right: rhs,
                info: (),
            },
            Token::LessThan { .. } => BinaryExpression::LessThen {
                left: lhs,
                right: rhs,
                info: (),
            },
            Token::GreaterOrEqual { .. } => BinaryExpression::GreaterOrEqual {
                left: lhs,
                right: rhs,
                info: (),
            },
            Token::LessOrEqual { .. } => BinaryExpression::LessOrEqual {
                left: lhs,
                right: rhs,
                info: (),
            },
            _ => unreachable!(),
        };

        Ok(Expression::Binary(Box::new(binary.balance())))
    }
}

impl From<Expression<()>> for AstNode {
    fn from(value: Expression<()>) -> Self {
        AstNode::Expression(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::ast::{Statement, TypeName},
    };

    use super::*;

    #[test]
    fn test_parse_id() {
        let tokens = vec![Token::Id {
            value: "some_id".into(),
            position: Span::default(),
        }];

        assert_eq!(
            Expression::parse(&mut tokens.into()),
            Ok(AstNode::Expression(Expression::Id(Id {
                name: "some_id".into(),
                info: (),
                position: Span::default()
            })))
        )
    }

    #[test]
    fn test_parse_num() {
        let tokens = vec![Token::Integer {
            value: 42,
            position: Span::default(),
        }];

        assert_eq!(
            Expression::parse(&mut tokens.into()),
            Ok(AstNode::Expression(Expression::Num(Num::Integer(
                42,
                (),
                Span::default()
            ))))
        )
    }

    #[test]
    fn test_parse_function_simple() {
        let mut tokens = Lexer::new("fn (): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Function(Function {
                id: None,
                parameters: vec![],
                statements: vec![],
                return_type: TypeName::Literal("i32".into()),
                info: ()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_function_complex() {
        let mut tokens = Lexer::new(
            "fn (x: i32, y: i32): i32 {
            return x + y;
        }",
        )
        .lex()
        .expect("something is wrong")
        .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Function(Function {
                id: None,
                parameters: vec![
                    FunctionParameter {
                        name: Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into()),
                        info: ()
                    },
                    FunctionParameter {
                        name: Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into()),
                        info: ()
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![Statement::Return(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
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
                        info: (),
                    }
                )))],
                info: ()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_lambda_simple() {
        let mut tokens = Lexer::new("\\() => 42")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Lambda(Lambda {
                parameters: vec![],
                expression: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
                info: (),
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_lambda_complex() {
        let mut tokens = Lexer::new("\\(x, y) => { x + y }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Lambda(Lambda {
                parameters: vec![
                    LambdaParameter {
                        name: Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        },
                        info: (),
                    },
                    LambdaParameter {
                        name: Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        },
                        info: (),
                    }
                ],
                expression: Box::new(Expression::Block(Block {
                    statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                        BinaryExpression::Addition {
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
                            info: (),
                        }
                    )))],
                    info: (),
                })),
                info: (),
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_if() {
        let mut tokens = Lexer::new("if (x) { 3 + 4 } else { 42 + 1337 }")
            .lex()
            .expect("should work")
            .into();

        assert_eq!(
            Ok(Expression::If(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                })),
                statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Num(Num::Integer(3, (), Span::default())),
                        right: Expression::Num(Num::Integer(4, (), Span::default())),
                        info: (),
                    }
                )))],
                else_statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Num(Num::Integer(42, (), Span::default())),
                        right: Expression::Num(Num::Integer(1337, (), Span::default())),
                        info: (),
                    }
                )))],
                info: (),
            })
            .into()),
            Expression::parse(&mut tokens)
        )
    }

    #[test]
    fn test_parse_postfix_call_simple() {
        let mut tokens = Lexer::new("foo()").lex().expect("should work").into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Postfix(Postfix::Call {
                expr: Box::new(Expression::Id(Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                })),
                args: vec![],
                info: ()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_postfix_call_complex() {
        let mut tokens = Lexer::new("(\\(x, y) => x + y)(42, 1337)")
            .lex()
            .expect("should work")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Postfix(Postfix::Call {
                expr: Box::new(Expression::Parens(Box::new(Expression::Lambda(Lambda {
                    parameters: vec![
                        LambdaParameter {
                            name: Id {
                                name: "x".into(),
                                info: (),
                                position: Span::default()
                            },
                            info: (),
                        },
                        LambdaParameter {
                            name: Id {
                                name: "y".into(),
                                info: (),
                                position: Span::default()
                            },
                            info: (),
                        }
                    ],
                    expression: Box::new(Expression::Binary(Box::new(
                        BinaryExpression::Addition {
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
                            info: (),
                        }
                    ))),
                    info: (),
                })))),
                args: vec![
                    Expression::Num(Num::Integer(42, (), Span::default())),
                    Expression::Num(Num::Integer(1337, (), Span::default()))
                ],
                info: (),
            })
            .into()),
            result
        );
    }

    #[test]
    fn test_parse_array_empty() {
        let mut tokens = Lexer::new("[]").lex().expect("something is wrong").into();

        let result = Expression::parse(&mut tokens);
        assert_eq!(
            Ok(Expression::Array(Array::Literal {
                values: vec![],
                info: ()
            })
            .into()),
            result
        );
    }

    #[test]
    fn test_parse_array_simple_literal() {
        let mut tokens = Lexer::new("[42, 1337]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);
        assert_eq!(
            Ok(Expression::Array(Array::Literal {
                values: vec![
                    Expression::Num(Num::Integer(42, (), Span::default())),
                    Expression::Num(Num::Integer(1337, (), Span::default()))
                ],
                info: ()
            })
            .into()),
            result
        );
    }

    #[test]
    fn test_parse_index_simple() {
        let mut tokens = Lexer::new("foo[42]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Postfix(Postfix::Index {
                expr: Box::new(Expression::Id(Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                })),
                index: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
                info: ()
            })
            .into()),
            result
        )
    }

    #[test]
    fn parse_struct() {
        let mut tokens = Lexer::new("Foo { bar: 42, baz: \\(x) => x + x }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::StructInitialisation(StructInitialisation {
                id: Id {
                    name: "Foo".into(),
                    info: (),
                    position: Span::default()
                },
                fields: vec![
                    StructFieldInitialisation {
                        name: Id {
                            name: "bar".into(),
                            info: (),
                            position: Span::default()
                        },
                        value: Expression::Num(Num::Integer(42, (), Span::default())),
                        info: ()
                    },
                    StructFieldInitialisation {
                        name: Id {
                            name: "baz".into(),
                            info: (),
                            position: Span::default()
                        },
                        value: Expression::Lambda(Lambda {
                            parameters: vec![LambdaParameter {
                                name: Id {
                                    name: "x".into(),
                                    info: (),
                                    position: Span::default()
                                },
                                info: ()
                            }],
                            expression: Box::new(Expression::Binary(Box::new(
                                BinaryExpression::Addition {
                                    left: Expression::Id(Id {
                                        name: "x".into(),
                                        info: (),
                                        position: Span::default()
                                    }),
                                    right: Expression::Id(Id {
                                        name: "x".into(),
                                        info: (),
                                        position: Span::default()
                                    }),
                                    info: (),
                                }
                            ))),
                            info: ()
                        }),
                        info: ()
                    }
                ],
                info: ()
            })
            .into()),
            result
        );
    }

    #[test]
    fn parse_property_access_simple() {
        let mut tokens = Lexer::new("foo.bar")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Postfix(Postfix::PropertyAccess {
                expr: Box::new(Expression::Id(Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                })),
                property: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default()
                },
                info: ()
            })
            .into()),
            result
        );
    }

    #[test]
    fn parse_property_access_complex() {
        let mut tokens = Lexer::new("foo().bar")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Postfix(Postfix::PropertyAccess {
                expr: Box::new(Expression::Postfix(Postfix::Call {
                    expr: Box::new(Expression::Id(Id {
                        name: "foo".into(),
                        info: (),
                        position: Span::default()
                    })),
                    args: vec![],
                    info: ()
                })),
                property: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default()
                },
                info: ()
            })
            .into()),
            result
        );
    }

    #[test]
    fn test_simple_minus() {
        let mut tokens = Lexer::new("-42").lex().expect("something is wrong").into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Prefix(Prefix::Minus {
                expr: Box::new(Expression::Num(Num::Integer(42, (), Span::default())))
            })
            .into()),
            result
        );
    }

    #[test]
    fn test_complex_minus() {
        let mut tokens = Lexer::new("-someFunction()")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Prefix(Prefix::Minus {
                expr: Box::new(Expression::Postfix(Postfix::Call {
                    expr: Box::new(Expression::Id(Id {
                        name: "someFunction".into(),
                        info: (),
                        position: Span::default()
                    })),
                    args: vec![],
                    info: ()
                }))
            })
            .into()),
            result
        );
    }

    #[test]
    fn test_simple_negation() {
        let mut tokens = Lexer::new("!42").lex().expect("something is wrong").into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Prefix(Prefix::Negation {
                expr: Box::new(Expression::Num(Num::Integer(42, (), Span::default())))
            })
            .into()),
            result
        );
    }

    #[test]
    fn test_complex_negation() {
        let mut tokens = Lexer::new("!someFunction()")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Expression::parse(&mut tokens);

        assert_eq!(
            Ok(Expression::Prefix(Prefix::Negation {
                expr: Box::new(Expression::Postfix(Postfix::Call {
                    expr: Box::new(Expression::Id(Id {
                        name: "someFunction".into(),
                        info: (),
                        position: Span::default()
                    })),
                    args: vec![],
                    info: ()
                }))
            })
            .into()),
            result
        );
    }
}
