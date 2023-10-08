use crate::{
    lexer::{TokenKind, Tokens},
    parser::{ast::AstNode, FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Num(pub u64);

impl FromTokens<TokenKind> for Num {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let value = match tokens.next() {
            Some(TokenKind::Num { value, .. }) => value,
            Some(token) => {
                return Err(ParseError {
                    message: "Tried to parse Num from non Num token".into(),
                    position: Some(token.position()),
                })
            }
            None => return Err(ParseError::eof("Id")),
        };

        Ok(Num(value).into())
    }
}

impl From<Num> for AstNode {
    fn from(value: Num) -> Self {
        AstNode::Num(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![TokenKind::Num {
            value: 42,
            position: (0, 0),
        }];
        assert_eq!(Num::parse(&mut tokens.into()), Ok(AstNode::Num(Num(42))));
    }

    #[test]
    fn test_error_on_non_num() {
        let tokens = vec![TokenKind::Id {
            value: "some_id".into(),
            position: (0, 0),
        }];
        assert!(Num::parse(&mut tokens.into()).is_err());
    }

    #[test]
    fn test_error_on_eof() {
        let tokens = vec![];
        assert!(Num::parse(&mut tokens.into()).is_err());
    }
}
