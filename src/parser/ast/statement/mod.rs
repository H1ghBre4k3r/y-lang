mod initialization;

pub use self::initialization::*;

use crate::{
    lexer::{Token, Tokens},
    parser::{combinators::Comb, FromTokens, ParseError},
};

use super::{AstNode, Expression, Function, If};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Function(Function),
    If(If),
    Initialization(Initialization),
    Expression(Expression),
    Return(Expression),
}

impl FromTokens<Token> for Statement {
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
                let [AstNode::If(function)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::If(function.clone()).into())
            }
            Token::FnKeyword { .. } => {
                let matcher = Comb::FUNCTION >> !Comb::SEMI;
                let result = matcher.parse(tokens)?;
                let [AstNode::Function(function)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Function(function.clone()).into())
            }
            Token::Let { .. } => {
                let matcher = Comb::INITIALIZATION;
                let result = matcher.parse(tokens)?;
                let [AstNode::Initialization(init)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Initialization(init.clone()).into())
            }
            Token::ReturnKeyword { .. } => {
                let matcher = Comb::RETURN_KEYWORD >> Comb::EXPR >> Comb::SEMI;
                let result = matcher.parse(tokens)?;
                let [AstNode::Expression(expr)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Return(expr.clone()).into())
            }
            token => {
                let matcher = Comb::EXPR;
                let result = matcher.parse(tokens).map_err(|_| ParseError {
                    message: format!("Unexpected token {token:?} while trying to parse Statement"),
                    position: Some(token.position()),
                })?;
                let [AstNode::Expression(expr)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Expression(expr.clone()).into())
            }
        }
    }
}

impl From<Statement> for AstNode {
    fn from(value: Statement) -> Self {
        AstNode::Statement(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::ast::{Id, Num},
    };

    use super::*;

    #[test]
    fn test_basic_return() {
        let mut tokens = Lexer::new("return 42;").lex().expect("should work").into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::Return(Expression::Num(Num(42))).into()),
            result
        );
    }

    #[test]
    fn test_if_else_without_semicolon() {
        let mut tokens = Lexer::new("if x { 3 + 4 } else { 42 + 1337 }")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::If(If {
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
            result
        )
    }

    #[test]
    fn test_if_else_with_semicolon() {
        let mut tokens = Lexer::new("if x { 3 + 4 } else { 42 + 1337 };")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::If(If {
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
            result
        )
    }

    #[test]
    fn test_if_else_ignores_call() {
        let mut tokens = Lexer::new("if x { 3 + 4 } else { 42 + 1337 }()")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::If(If {
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
            result
        )
    }
}
