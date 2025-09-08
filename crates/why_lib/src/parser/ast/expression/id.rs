use crate::{
    grammar::{self, FromGrammar},
    lexer::{GetPosition, Span, Token},
    parser::{ast::AstNode, FromTokens, ParseError, ParseState},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Id<T> {
    pub name: String,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::Identifier> for Id<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Identifier>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;
        Id {
            name: value.0.value,
            info: (),
            position: Span::new(span, source),
        }
    }
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
    use crate::parser::test_helpers::*;
    use super::*;

    #[test]
    fn test_parse_simple_identifier() {
        let result = parse_id("some_id").unwrap();
        assert_eq!(result.name, "some_id");
    }

    #[test]
    fn test_parse_underscore_identifier() {
        let result = parse_id("_private").unwrap();
        assert_eq!(result.name, "_private");
    }

    #[test]
    fn test_parse_mixed_identifier() {
        let result = parse_id("variable_name123").unwrap();
        assert_eq!(result.name, "variable_name123");
    }

    #[test]
    fn test_parse_single_letter() {
        let result = parse_id("x").unwrap();
        assert_eq!(result.name, "x");
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        // Test that invalid identifier formats fail gracefully
        assert!(parse_id("123invalid").is_err()); // Can't start with number
        assert!(parse_id("").is_err()); // Empty string
        assert!(parse_id("with-dash").is_err()); // Contains invalid character
    }
}
