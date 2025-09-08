use crate::{
    grammar::{self, FromGrammar},
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Statement},
        combinators::Comb,
        FromTokens,
    },
};

use super::{Block, Expression};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct If<T> {
    pub condition: Box<Expression<T>>,
    // TODO: This should/could just be a block
    pub statements: Vec<Statement<T>>,
    pub else_statements: Vec<Statement<T>>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::IfExpression> for If<()> {
    fn transform(item: rust_sitter::Spanned<grammar::IfExpression>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        // Extract then block statements
        let then_block = Block::transform(value.then_block.value, source);

        // Extract else block statements if present
        let else_statements = if let Some(else_clause) = value.else_block {
            let else_block = Block::transform(else_clause.value.block.value, source);
            else_block.statements
        } else {
            vec![]
        };

        If {
            condition: Box::new(Expression::transform(*value.condition, source)),
            statements: then_block.statements,
            else_statements,
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl FromTokens<Token> for If<()> {
    fn parse(
        tokens: &mut crate::parser::ParseState<Token>,
    ) -> Result<crate::parser::ast::AstNode, crate::parser::ParseError> {
        let position = tokens.span()?;

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

        let Span { end, .. } = tokens.prev_span()?;
        Ok(If {
            condition: Box::new(condition),
            statements: if_block.statements,
            else_statements,
            info: (),
            position: Span {
                start: position.start,
                end,
                source: position.source,
            },
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
        parser::ast::{BinaryExpression, BinaryOperator, Id, Num},
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
                info: (),
                position: Span::default()
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
                info: (),
                position: Span::default()
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
                    BinaryExpression {
                        left: Expression::Num(Num::Integer(3, (), Span::default())),
                        right: Expression::Num(Num::Integer(4, (), Span::default())),
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
                    }
                )))],
                else_statements: vec![],
                info: (),
                position: Span::default()
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
                    BinaryExpression {
                        left: Expression::Num(Num::Integer(3, (), Span::default())),
                        right: Expression::Num(Num::Integer(4, (), Span::default())),
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
                    }
                )))],
                else_statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression {
                        left: Expression::Num(Num::Integer(42, (), Span::default())),
                        right: Expression::Num(Num::Integer(1337, (), Span::default())),
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
                    }
                )))],
                info: (),
                position: Span::default()
            }
            .into()),
            If::parse(&mut tokens)
        )
    }
}
