use crate::{
    lexer::{Token, Tokens},
    parser::{ast::AstNode, FromTokens, ParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Num(pub u64);

impl FromTokens<Token> for Num {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let value = match tokens.next() {
            Some(Token::Num { value, .. }) => value,
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
        let tokens = vec![Token::Num {
            value: 42,
            position: (0, 0),
        }];
        let tokens = tokens;
        assert_eq!(Num::parse(&mut tokens.into()), Ok(AstNode::Num(Num(42))));
    }

    #[test]
    fn test_error_on_non_num() {
        let tokens = vec![Token::Id {
            value: "some_id".into(),
            position: (0, 0),
        }];
        let tokens = tokens;
        assert!(Num::parse(&mut tokens.into()).is_err());
    }

    #[test]
    fn test_error_on_eof() {
        let tokens = vec![];
        let tokens = tokens;
        assert!(Num::parse(&mut tokens.into()).is_err());
    }
}
