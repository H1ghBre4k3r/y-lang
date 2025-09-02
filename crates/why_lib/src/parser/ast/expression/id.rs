use crate::{
    lexer::{GetPosition, Span, Token},
    parser::{ast::AstNode, FromTokens, ParseError, ParseState},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Id<T> {
    pub name: String,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Id<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, crate::parser::ParseError>
    where
        Self: Sized,
    {
        let position = tokens.span()?;
        let value = match tokens.next() {
            Some(Token::Id { value, .. }) => value,
            Some(token) => {
                return Err(ParseError {
                    message: format!("Tried to parse Id from non id token ({token:?})"),
                    position: Some(token.position()),
                })
            }
            None => return Err(ParseError::eof("Id")),
        };
        Ok(Id {
            name: value,
            info: (),
            position,
        }
        .into())
    }
}

impl From<Id<()>> for AstNode {
    fn from(value: Id<()>) -> Self {
        AstNode::Id(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Span;

    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![Token::Id {
            value: "some_id".into(),
            position: Span::default(),
        }];
        assert_eq!(
            Id::parse(&mut tokens.into()),
            Ok(AstNode::Id(Id {
                name: "some_id".into(),
                info: (),
                position: Span::default()
            }))
        );
    }

    #[test]
    fn test_error_on_non_id() {
        let tokens = vec![Token::Integer {
            value: 3,
            position: Span::default(),
        }];
        assert!(Id::parse(&mut tokens.into()).is_err());
    }

    #[test]
    fn test_error_on_eof() {
        let tokens = vec![];
        assert!(Id::parse(&mut tokens.into()).is_err());
    }
}
