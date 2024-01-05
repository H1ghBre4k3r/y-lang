use crate::{
    lexer::{Token, Tokens},
    parser::{ast::AstNode, combinators::Comb, FromTokens, ParseError},
};

use super::{Expression, Num};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Array {
    Literal {
        values: Vec<Expression>,
    },
    Default {
        initial_value: Box<Expression>,
        length: Num,
    },
}

impl FromTokens<Token> for Array {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let start = tokens.get_index();
        let matcher = Comb::LBRACKET >> (Comb::EXPR % Comb::COMMA) >> Comb::RBRACKET;

        if let Ok(result) = matcher.parse(tokens) {
            let mut values = vec![];

            for node in result {
                let AstNode::Expression(value) = node else {
                    unreachable!();
                };
                values.push(value);
            }
            return Ok(Array::Literal { values }.into());
        }
        tokens.set_index(start);

        let matcher = Comb::LBRACKET >> Comb::EXPR >> Comb::SEMI >> Comb::NUM >> Comb::RBRACKET;
        if let Ok(result) = matcher.parse(tokens) {
            let Some(AstNode::Expression(initial_value)) = result.first().cloned() else {
                unreachable!()
            };

            let Some(AstNode::Num(length)) = result.get(1).cloned() else {
                unreachable!()
            };

            return Ok(Array::Default {
                initial_value: Box::new(initial_value),
                length,
            }
            .into());
        };

        Err(ParseError {
            message: "failed to parse array initialization".into(),
            position: None,
        })
    }
}

impl From<Array> for AstNode {
    fn from(value: Array) -> Self {
        Self::Array(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn test_empty_array() {
        let mut tokens = Lexer::new("[]").lex().expect("something is wrong").into();

        let result = Array::parse(&mut tokens);
        assert_eq!(Ok(Array::Literal { values: vec![] }.into()), result);
    }

    #[test]
    fn test_simple_literal() {
        let mut tokens = Lexer::new("[42, 1337]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Array::parse(&mut tokens);
        assert_eq!(
            Ok(Array::Literal {
                values: vec![
                    Expression::Num(Num::Integer(42)),
                    Expression::Num(Num::Integer(1337))
                ]
            }
            .into()),
            result
        );
    }

    #[test]
    fn test_simple_default() {
        let mut tokens = Lexer::new("[42; 5]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Array::parse(&mut tokens);
        assert_eq!(
            Ok(Array::Default {
                initial_value: Box::new(Expression::Num(Num::Integer(42))),
                length: Num::Integer(5)
            }
            .into()),
            result
        );
    }

    #[test]
    fn test_faulty_default() {
        let mut tokens = Lexer::new("[42; ]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Array::parse(&mut tokens);
        assert!(result.is_err());
    }
}
