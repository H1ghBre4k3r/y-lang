mod assignment;
mod constant;
mod declaration;
mod initialisation;
mod struct_declaration;
mod while_loop;

pub use self::assignment::*;
pub use self::constant::*;
pub use self::declaration::*;
pub use self::initialisation::*;
pub use self::struct_declaration::*;
pub use self::while_loop::*;

use crate::{
    lexer::{Token, Tokens},
    parser::{combinators::Comb, FromTokens, ParseError},
};

use super::{AstNode, Expression, Function, If};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<T> {
    Function(Function<T>),
    If(If<T>),
    WhileLoop(WhileLoop<T>),
    Initialization(Initialisation<T>),
    Constant(Constant<T>),
    Assignment(Assignment<T>),
    Expression(Expression<T>),
    YieldingExpression(Expression<T>),
    Return(Expression<T>),
    Comment(String),
    Declaration(Declaration<T>),
    StructDeclaration(StructDeclaration<T>),
}

impl FromTokens<Token> for Statement<()> {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let Some(next) = tokens.peek() else {
            todo!();
        };

        match next {
            Token::IfKeyword { .. } => {
                let matcher = Comb::IF >> !Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::If(if_statement)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::If(if_statement.clone()).into())
            }
            Token::FnKeyword { .. } => {
                let matcher = Comb::FUNCTION >> !Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::Function(function)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Function(function.clone()).into())
            }
            Token::WhileKeyword { .. } => {
                let matcher = Comb::WHILE_LOOP >> !Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::WhileLoop(while_loop_statement)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::WhileLoop(while_loop_statement.clone()).into())
            }
            Token::Let { .. } => {
                let matcher = Comb::INITIALISATION >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::Initialization(init)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Initialization(init.clone()).into())
            }
            Token::Const { .. } => {
                let matcher = Comb::CONSTANT >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::Constant(constant)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Constant(constant.clone()).into())
            }
            Token::ReturnKeyword { .. } => {
                let matcher = Comb::RETURN_KEYWORD >> Comb::EXPR >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::Expression(expr)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Return(expr.clone()).into())
            }
            Token::DeclareKeyword { .. } => {
                let matcher = Comb::DECLARATION >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let Some(AstNode::Declaration(declaration)) = result.first().cloned() else {
                    unreachable!()
                };
                Ok(Statement::Declaration(declaration).into())
            }
            Token::Comment { value, .. } => {
                tokens.next();
                Ok(Statement::Comment(value).into())
            }
            Token::StructKeyword { .. } => {
                let matcher = Comb::STRUCT_DECLARATION >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let Some(AstNode::StructDeclaration(declaration)) = result.first().cloned() else {
                    unreachable!()
                };
                Ok(Statement::StructDeclaration(declaration).into())
            }
            _ => {
                if let Ok(assignment) = Self::parse_assignment(tokens) {
                    return Ok(assignment);
                };

                if let Ok(expr) = Self::parse_expression(tokens) {
                    return Ok(expr);
                };

                Err(ParseError {
                    message: "could not parse statement".into(),
                    position: None,
                })
            }
        }
    }
}

impl Statement<()> {
    fn parse_assignment(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let matcher = Comb::ASSIGNMENT >> Comb::SEMI;
        let result = matcher.parse(tokens).map_err(|e| {
            tokens.set_index(index);
            e
        })?;

        let [AstNode::Assignment(assignment)] = result.as_slice() else {
            unreachable!()
        };

        Ok(Statement::Assignment(assignment.clone()).into())
    }

    fn parse_expression(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let matcher = Comb::EXPR;
        let result = matcher.parse(tokens).map_err(|e| {
            tokens.set_index(index);
            e
        })?;

        let [AstNode::Expression(expr)] = result.as_slice() else {
            unreachable!()
        };
        match tokens.peek() {
            Some(Token::Semicolon { .. }) => {
                tokens.next();
                Ok(Statement::Expression(expr.clone()).into())
            }
            _ => Ok(Statement::YieldingExpression(expr.clone()).into()),
        }
    }
}

impl From<Statement<()>> for AstNode {
    fn from(value: Statement<()>) -> Self {
        AstNode::Statement(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::ast::{BinaryExpression, Id, Num, TypeName},
    };

    use super::*;

    #[test]
    fn test_basic_constant() {
        let mut tokens = Lexer::new("const foo: i32 = 42;")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::Constant(Constant {
                id: Id {
                    name: "foo".into(),
                    info: ()
                },
                type_name: TypeName::Literal("i32".into()),
                value: Expression::Num(Num::Integer(42, ())),
                info: ()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_basic_return() {
        let mut tokens = Lexer::new("return 42;").lex().expect("should work").into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::Return(Expression::Num(Num::Integer(42, ()))).into()),
            result
        );
    }

    #[test]
    fn test_if_else_without_semicolon() {
        let mut tokens = Lexer::new("if (x) { 3 + 4 } else { 42 + 1337 }")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::If(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: ()
                })),
                statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Num(Num::Integer(3, ())),
                        right: Expression::Num(Num::Integer(4, ())),
                        info: (),
                    }
                )))],
                else_statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Num(Num::Integer(42, ())),
                        right: Expression::Num(Num::Integer(1337, ())),
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
    fn test_if_else_with_semicolon() {
        let mut tokens = Lexer::new("if (x) { 3 + 4 } else { 42 + 1337 };")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::If(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: ()
                })),
                statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Num(Num::Integer(3, ())),
                        right: Expression::Num(Num::Integer(4, ())),
                        info: (),
                    }
                )))],
                else_statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Num(Num::Integer(42, ())),
                        right: Expression::Num(Num::Integer(1337, ())),
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
    fn test_if_else_ignores_call() {
        let mut tokens = Lexer::new("if (x) { 3 + 4 } else { 42 + 1337 }()")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::If(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: ()
                })),
                statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Num(Num::Integer(3, ())),
                        right: Expression::Num(Num::Integer(4, ())),
                        info: (),
                    }
                )))],
                else_statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Num(Num::Integer(42, ())),
                        right: Expression::Num(Num::Integer(1337, ())),
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
    fn test_simple_assignment() {
        let mut tokens = Lexer::new("x = 42;").lex().expect("should work").into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::Assignment(Assignment {
                id: Id {
                    name: "x".into(),
                    info: ()
                },
                value: Expression::Num(Num::Integer(42, ())),
                info: ()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_assignment_needs_semicolon() {
        let mut tokens = Lexer::new("x = 42").lex().expect("should work").into();

        let result = Statement::parse_assignment(&mut tokens);

        assert!(result.is_err())
    }
}
