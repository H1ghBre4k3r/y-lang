use rust_sitter::Spanned;

use crate::{
    grammar::{self, FromGrammar},
    lexer::{GetPosition, Span, Token},
    parser::{ast::AstNode, FromTokens, ParseError, ParseState},
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Num<T> {
    Integer(u64, T, Span),
    FloatingPoint(f64, T, Span),
}

impl<T> Eq for Num<T> where T: Eq {}

impl FromGrammar<grammar::Number> for Num<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Number>, source: &str) -> Self {
        let Spanned { value, span } = item;

        match value {
            grammar::Number::Integer(grammar::Integer(integer)) => {
                Num::Integer(integer, (), Span::new(span, source))
            }
            grammar::Number::Floating(grammar::Floating(floating)) => {
                Num::FloatingPoint(floating, (), Span::new(span, source))
            }
        }
    }
}

impl FromTokens<Token> for Num<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let position = tokens.span()?;
        match tokens.next() {
            Some(Token::Integer { value, .. }) => Ok(Num::Integer(value, (), position).into()),
            Some(Token::FloatingPoint { value, .. }) => {
                Ok(Num::FloatingPoint(value, (), position).into())
            }
            Some(token) => Err(ParseError {
                message: "Tried to parse Num from non Num token".into(),
                position: Some(token.position()),
            }),
            None => Err(ParseError::eof("Id")),
        }
    }
}

impl<T> Num<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Num::Integer(_, info, ..) => info.clone(),
            Num::FloatingPoint(_, info, ..) => info.clone(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Num::Integer(_, _, position) => position.clone(),
            Num::FloatingPoint(_, _, position) => position.clone(),
        }
    }
}

impl From<Num<()>> for AstNode {
    fn from(value: Num<()>) -> Self {
        AstNode::Num(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_helpers::*;
    use super::*;

    #[test]
    fn test_parse_integer() {
        let result = parse_number("42").unwrap();
        assert!(matches!(result, Num::Integer(42, (), _)));
    }

    #[test]
    fn test_parse_floating_point() {
        let result = parse_number("1337.42").unwrap();
        assert!(matches!(result, Num::FloatingPoint(value, (), _) if (value - 1337.42).abs() < 0.01));
    }

    #[test]
    fn test_parse_zero() {
        let result = parse_number("0").unwrap();
        assert!(matches!(result, Num::Integer(0, (), _)));
    }

    #[test]
    fn test_parse_large_integer() {
        let result = parse_number("1000000").unwrap();
        assert!(matches!(result, Num::Integer(1000000, (), _)));
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        // Test that invalid number formats fail gracefully
        assert!(parse_number("abc").is_err());
        assert!(parse_number("").is_err());
    }
}
