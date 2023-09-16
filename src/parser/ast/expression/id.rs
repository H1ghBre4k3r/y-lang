use crate::{
    lexer::{Token, Tokens},
    parser::{ast::AstNode, FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id(pub String);

impl FromTokens for Id {
    fn parse(tokens: &mut Tokens) -> Result<AstNode, crate::parser::ParseError>
    where
        Self: Sized,
    {
        let value = match tokens.next() {
            Some(Token::Id { value, .. }) => value,
            Some(token) => {
                return Err(ParseError {
                    message: "Tried to parse Id from non id token".into(),
                    position: Some(token.position()),
                })
            }
            None => return Err(ParseError::eof("Id")),
        };
        Ok(Id(value).into())
    }
}

impl From<Id> for AstNode {
    fn from(value: Id) -> Self {
        AstNode::Id(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![Token::Id {
            value: "some_id".into(),
            position: (0, 0),
        }];
        let tokens = tokens;
        assert_eq!(
            Id::parse(&mut tokens.into()),
            Ok(AstNode::Id(Id("some_id".into())))
        );
    }

    #[test]
    fn test_error_on_non_id() {
        let tokens = vec![Token::Num {
            value: 3,
            position: (0, 0),
        }];
        let tokens = tokens;
        assert!(Id::parse(&mut tokens.into()).is_err());
    }

    #[test]
    fn test_error_on_eof() {
        let tokens = vec![];
        let tokens = tokens;
        assert!(Id::parse(&mut tokens.into()).is_err());
    }
}
