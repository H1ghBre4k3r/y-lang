use crate::{
    grammar::{self, FromGrammar},
    lexer::{Span, Token},
    parser::{ast::AstNode, combinators::Comb, FromTokens, ParseError, ParseState},
};

use super::{Expression, Num};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Array<T> {
    Literal {
        values: Vec<Expression<T>>,
        info: T,
        position: Span,
    },
    Default {
        initial_value: Box<Expression<T>>,
        length: Num<T>,
        info: T,
        position: Span,
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

    pub fn position(&self) -> Span {
        match self {
            Array::Literal { position, .. } => position.clone(),
            Array::Default { position, .. } => position.clone(),
        }
    }
}

impl FromGrammar<grammar::Array> for Array<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Array>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        // Convert each element from grammar::Expression to Expression<()>
        // Note: elements are not wrapped in Spanned, so we create our own with the overall span
        let values = value
            .elements
            .into_iter()
            .map(|expr| Expression::transform(expr, source))
            .collect();

        Array::Literal {
            values,
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl FromTokens<Token> for Array<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;
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
            return Ok(Array::Literal {
                values,
                info: (),
                position,
            }
            .into());
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
                position,
            }
            .into());
        };

        Err(ParseError {
            message: "failed to parse array initialization".into(),
            position: Some(position),
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
    use crate::lexer::{Lexer, Span};

    use super::*;

    #[test]
    fn test_empty_array() {
        let mut tokens = Lexer::new("[]").lex().expect("something is wrong").into();

        let result = Array::parse(&mut tokens);
        assert_eq!(
            Ok(Array::Literal {
                values: vec![],
                info: (),
                position: Span::default()
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
                    Expression::Num(Num::Integer(42, (), Span::default())),
                    Expression::Num(Num::Integer(1337, (), Span::default())),
                ],
                info: (),
                position: Span::default()
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
                initial_value: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
                length: Num::Integer(5, (), Span::default()),
                info: (),
                position: Span::default()
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
