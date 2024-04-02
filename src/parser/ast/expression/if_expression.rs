use crate::{
    lexer::Token,
    parser::{
        ast::{AstNode, Statement},
        combinators::Comb,
        FromTokens,
    },
};

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If<T> {
    pub condition: Box<Expression<T>>,
    pub statements: Vec<Statement<T>>,
    pub else_statements: Vec<Statement<T>>,
    pub info: T,
}

impl FromTokens<Token> for If<()> {
    fn parse(
        tokens: &mut crate::parser::ParseState<Token>,
    ) -> Result<crate::parser::ast::AstNode, crate::parser::ParseError> {
        let matcher = Comb::IF_KEYWORD >> Comb::LPAREN >> Comb::EXPR >> Comb::RPAREN >> Comb::BLOCK;

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let Some(AstNode::Expression(condition)) = result.next() else {
            unreachable!()
        };

        let Some(AstNode::Block(if_block)) = result.next() else {
            unreachable!()
        };

        let matcher = !(Comb::ELSE_KEYWORD >> Comb::BLOCK);

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let else_statements = match result.next() {
            Some(AstNode::Block(else_block)) => else_block.statements,
            None => vec![],
            _ => unreachable!(),
        };

        Ok(If {
            condition: Box::new(condition),
            statements: if_block.statements,
            else_statements,
            info: (),
        }
        .into())
    }
}

impl From<If<()>> for AstNode {
    fn from(value: If<()>) -> Self {
        AstNode::If(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::ast::{BinaryExpression, Id, Num},
    };

    use super::*;

    #[test]
    fn test_simple_if() {
        let mut tokens = Lexer::new("if (x) {}").lex().expect("should work").into();

        assert_eq!(
            Ok(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                })),
                statements: vec![],
                else_statements: vec![],
                info: ()
            }
            .into()),
            If::parse(&mut tokens)
        )
    }

    #[test]
    fn test_simple_if_else() {
        let mut tokens = Lexer::new("if (x) {} else {}")
            .lex()
            .expect("should work")
            .into();

        assert_eq!(
            Ok(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                })),
                statements: vec![],
                else_statements: vec![],
                info: ()
            }
            .into()),
            If::parse(&mut tokens)
        )
    }

    #[test]
    fn test_complexer_if() {
        let mut tokens = Lexer::new("if (x) { 3 + 4 }")
            .lex()
            .expect("should work")
            .into();

        assert_eq!(
            Ok(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                })),
                statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Num(Num::Integer(3, ())),
                        right: Expression::Num(Num::Integer(4, ())),
                        info: (),
                    }
                )))],
                else_statements: vec![],
                info: ()
            }
            .into()),
            If::parse(&mut tokens)
        )
    }

    #[test]
    fn test_complexer_if_else() {
        let mut tokens = Lexer::new("if (x) { 3 + 4 } else { 42 + 1337 }")
            .lex()
            .expect("should work")
            .into();

        assert_eq!(
            Ok(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
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
            }
            .into()),
            If::parse(&mut tokens)
        )
    }
}
