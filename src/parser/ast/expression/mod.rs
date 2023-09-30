mod block;
mod function;
mod id;
mod if_expression;
mod lambda;
mod num;
mod postfix;

pub use self::block::*;
pub use self::function::*;
pub use self::id::*;
pub use self::if_expression::*;
pub use self::lambda::*;
pub use self::num::*;
pub use self::postfix::*;

use crate::lexer::Tokens;
use crate::parser::combinators::Comb;
use crate::{
    lexer::Token,
    parser::{FromTokens, ParseError},
};

use super::AstNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Id(Id),
    Num(Num),
    Function(Function),
    Lambda(Lambda),
    If(If),
    Block(Block),
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Parens(Box<Expression>),
    Postfix(Postfix),
}

impl FromTokens<Token> for Expression {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let expr = if let Some(Token::LParen { .. }) = tokens.peek() {
            let matcher = Comb::LPAREN >> Comb::EXPR >> Comb::RPAREN;
            let result = matcher.parse(tokens)?;
            let expr = match result.get(0) {
                Some(AstNode::Expression(rhs)) => rhs.clone(),
                None | Some(_) => unreachable!(),
            };
            Expression::Parens(Box::new(expr))
        } else {
            let matcher =
                Comb::FUNCTION | Comb::IF | Comb::NUM | Comb::ID | Comb::LAMBDA | Comb::BLOCK;
            let result = matcher.parse(tokens)?;
            match result.get(0) {
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
                None | Some(_) => unreachable!(),
            }
        };

        let Some(next) = tokens.peek() else {
            return Ok(expr.into());
        };

        let tuple = match next {
            Token::Times { .. } => {
                tokens.next();
                Expression::Multiplication
            }
            Token::Plus { .. } => {
                tokens.next();
                Expression::Addition
            }
            Token::LParen { .. } => {
                return Ok(Expression::Postfix(Self::parse_call(expr, tokens)?).into())
            }
            _ => return Ok(expr.into()),
        };

        let matcher = Comb::EXPR;
        let result = matcher.parse(tokens)?;
        let rhs = match result.get(0) {
            Some(AstNode::Expression(rhs)) => rhs.clone(),
            None | Some(_) => unreachable!(),
        };

        Ok(tuple(Box::new(expr), Box::new(rhs)).into())
    }
}

impl Expression {
    fn parse_call(expr: Expression, tokens: &mut Tokens<Token>) -> Result<Postfix, ParseError> {
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
        })
    }
}

impl From<Expression> for AstNode {
    fn from(value: Expression) -> Self {
        AstNode::Expression(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::ast::{Statement, TypeName},
    };

    use super::*;

    #[test]
    fn test_parse_id() {
        let tokens = vec![Token::Id {
            value: "some_id".into(),
            position: (0, 0),
        }];
        let tokens = tokens;

        assert_eq!(
            Expression::parse(&mut tokens.into()),
            Ok(AstNode::Expression(Expression::Id(Id("some_id".into()))))
        )
    }

    #[test]
    fn test_parse_num() {
        let tokens = vec![Token::Num {
            value: 42,
            position: (0, 0),
        }];
        let tokens = tokens;

        assert_eq!(
            Expression::parse(&mut tokens.into()),
            Ok(AstNode::Expression(Expression::Num(Num(42))))
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
                parameters: vec![],
                statements: vec![],
                return_type: TypeName::Literal("i32".into())
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
                parameters: vec![
                    Parameter {
                        name: Id("x".into()),
                        type_name: Some(TypeName::Literal("i32".into()))
                    },
                    Parameter {
                        name: Id("y".into()),
                        type_name: Some(TypeName::Literal("i32".into()))
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![Statement::Return(Expression::Addition(
                    Box::new(Expression::Id(Id("x".into()))),
                    Box::new(Expression::Id(Id("y".into()))),
                ))]
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
                expression: Box::new(Expression::Num(Num(42)))
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
                    Parameter {
                        name: Id("x".into()),
                        type_name: None
                    },
                    Parameter {
                        name: Id("y".into()),
                        type_name: None
                    }
                ],
                expression: Box::new(Expression::Block(Block {
                    statements: vec![Statement::Expression(Expression::Addition(
                        Box::new(Expression::Id(Id("x".into()))),
                        Box::new(Expression::Id(Id("y".into()))),
                    ))]
                }))
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_if() {
        let mut tokens = Lexer::new("if x { 3 + 4 } else { 42 + 1337 }")
            .lex()
            .expect("should work")
            .into();

        assert_eq!(
            Ok(Expression::If(If {
                condition: Box::new(Expression::Id(Id("x".into()))),
                statements: vec![Statement::Expression(Expression::Addition(
                    Box::new(Expression::Num(Num(3))),
                    Box::new(Expression::Num(Num(4)))
                ))],
                else_statements: vec![Statement::Expression(Expression::Addition(
                    Box::new(Expression::Num(Num(42))),
                    Box::new(Expression::Num(Num(1337)))
                ))],
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
                expr: Box::new(Expression::Id(Id("foo".into()))),
                args: vec![]
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
                        Parameter {
                            name: Id("x".into()),
                            type_name: None
                        },
                        Parameter {
                            name: Id("y".into()),
                            type_name: None
                        }
                    ],
                    expression: Box::new(Expression::Addition(
                        Box::new(Expression::Id(Id("x".into()))),
                        Box::new(Expression::Id(Id("y".into())))
                    ))
                })))),
                args: vec![Expression::Num(Num(42)), Expression::Num(Num(1337))]
            })
            .into()),
            result
        );
    }
}
