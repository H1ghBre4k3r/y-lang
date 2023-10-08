use crate::{
    lexer::{TokenKind, Tokens},
    parser::{ast::AstNode, combinators::Comb, FromTokens, ParseError},
};

use super::{Expression, Parameter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lambda {
    pub parameters: Vec<Parameter>,
    pub expression: Box<Expression>,
}

impl FromTokens<TokenKind> for Lambda {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError> {
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
        }
        .into())
    }
}

impl From<Lambda> for AstNode {
    fn from(value: Lambda) -> Self {
        AstNode::Lambda(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::ast::{Block, Id, Num, Statement},
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
                expression: Box::new(Expression::Num(Num(42)))
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
                    Box::new(Expression::Id(Id("y".into()))),
                ))
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
                    name: Id("x".into()),
                    type_name: None
                }],
                expression: Box::new(Expression::Id(Id("x".into())))
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
                    name: Id("x".into()),
                    type_name: None
                }],
                expression: Box::new(Expression::Block(Block {
                    statements: vec![Statement::YieldingExpression(Expression::Id(
                        Id("x".into())
                    ))]
                }))
            }
            .into()),
            result
        )
    }
}
