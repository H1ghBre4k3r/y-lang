use crate::{
    lexer::{GetPosition, Span, Token},
    parser::{ast::AstNode, FromTokens, ParseError, ParseState},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Num<T> {
    Integer(u64, T, Span),
    FloatingPoint(f64, T, Span),
}

impl<T> Eq for Num<T> where T: Eq {}

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
}

impl From<Num<()>> for AstNode {
    fn from(value: Num<()>) -> Self {
        AstNode::Num(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, Span};

    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![Token::Integer {
            value: 42,
            position: Span::default(),
        }];
        assert_eq!(
            Num::parse(&mut tokens.into()),
            Ok(AstNode::Num(Num::Integer(42, (), Span::default())))
        );
    }

    #[test]
    fn test_error_on_non_num() {
        let tokens = vec![Token::Id {
            value: "some_id".into(),
            position: Span::default(),
        }];
        assert!(Num::parse(&mut tokens.into()).is_err());
    }

    #[test]
    fn test_error_on_eof() {
        let tokens = vec![];
        assert!(Num::parse(&mut tokens.into()).is_err());
    }

    #[test]
    fn test_parse_floatingpoint() {
        let mut tokens = Lexer::new("1337.42").lex().expect("should work").into();

        let result = Num::parse(&mut tokens);

        assert_eq!(
            Ok(Num::FloatingPoint(1337.42, (), Span::default()).into()),
            result
        );
    }
}
