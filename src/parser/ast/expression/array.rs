use crate::{
    lexer::{GetPosition, Token},
    parser::{ast::AstNode, combinators::Comb, FromTokens, ParseError, ParseState},
};

use super::{Expression, Num};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Array<T> {
    Literal {
        values: Vec<Expression<T>>,
        info: T,
    },
    Default {
        initial_value: Box<Expression<T>>,
        length: Num<T>,
        info: T,
    },
}

impl<T> Array<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Array::Literal { info, .. } => info.clone(),
            Array::Default { info, .. } => info.clone(),
        }
    }
}

impl FromTokens<Token> for Array<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.peek().map(|token| token.position());
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
            return Ok(Array::Literal { values, info: () }.into());
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
                info: (),
            }
            .into());
        };

        Err(ParseError {
            message: "failed to parse array initialization".into(),
            position,
        })
    }
}

impl From<Array<()>> for AstNode {
    fn from(value: Array<()>) -> Self {
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
        assert_eq!(
            Ok(Array::Literal {
                values: vec![],
                info: ()
            }
            .into()),
            result
        );
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
                    Expression::Num(Num::Integer(42, ())),
                    Expression::Num(Num::Integer(1337, ())),
                ],
                info: ()
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
                initial_value: Box::new(Expression::Num(Num::Integer(42, ()))),
                length: Num::Integer(5, ()),
                info: ()
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
