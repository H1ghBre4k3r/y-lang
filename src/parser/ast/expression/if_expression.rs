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
pub struct If {
    pub condition: Box<Expression>,
    pub statements: Vec<Statement>,
    pub else_statements: Vec<Statement>,
}

impl FromTokens<Token> for If {
    fn parse(
        tokens: &mut crate::lexer::Tokens<Token>,
    ) -> Result<crate::parser::ast::AstNode, crate::parser::ParseError> {
        let matcher = Comb::IF_KEYWORD
            >> Comb::LPAREN
            >> Comb::EXPR
            >> Comb::RPAREN
            >> Comb::LBRACE
            >> (Comb::STATEMENT ^ ())
            >> Comb::RBRACE;

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let Some(AstNode::Expression(condition)) = result.next() else {
            unreachable!()
        };

        let mut statements = vec![];

        while let Some(AstNode::Statement(statement)) =
            result.next_if(|item| matches!(item, AstNode::Statement(_)))
        {
            statements.push(statement);
        }

        let matcher =
            !(Comb::ELSE_KEYWORD >> Comb::LBRACE >> (Comb::STATEMENT ^ ()) >> Comb::RBRACE);

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let mut else_statements = vec![];

        while let Some(AstNode::Statement(statement)) =
            result.next_if(|item| matches!(item, AstNode::Statement(_)))
        {
            else_statements.push(statement);
        }

        Ok(If {
            condition: Box::new(condition),
            statements,
            else_statements,
        }
        .into())
    }
}

impl From<If> for AstNode {
    fn from(value: If) -> Self {
        AstNode::If(value)
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
    fn test_simple_if() {
        let mut tokens = Lexer::new("if (x) {}").lex().expect("should work").into();

        assert_eq!(
            Ok(If {
                condition: Box::new(Expression::Id(Id("x".into()))),
                statements: vec![],
                else_statements: vec![]
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
                condition: Box::new(Expression::Id(Id("x".into()))),
                statements: vec![],
                else_statements: vec![]
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
                condition: Box::new(Expression::Id(Id("x".into()))),
                statements: vec![Statement::YieldingExpression(Expression::Addition(
                    Box::new(Expression::Num(Num::Integer(3))),
                    Box::new(Expression::Num(Num::Integer(4)))
                ))],
                else_statements: vec![]
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
                condition: Box::new(Expression::Id(Id("x".into()))),
                statements: vec![Statement::YieldingExpression(Expression::Addition(
                    Box::new(Expression::Num(Num::Integer(3))),
                    Box::new(Expression::Num(Num::Integer(4)))
                ))],
                else_statements: vec![Statement::YieldingExpression(Expression::Addition(
                    Box::new(Expression::Num(Num::Integer(42))),
                    Box::new(Expression::Num(Num::Integer(1337)))
                ))],
            }
            .into()),
            If::parse(&mut tokens)
        )
    }
}
