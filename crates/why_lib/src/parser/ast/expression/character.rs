use crate::grammar::{self, FromGrammar};
use crate::lexer::{Span, Token};
use crate::parser::ast::AstNode;
use crate::parser::{FromTokens, ParseError, ParseState};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Character<T> {
    pub character: char,
    pub position: Span,
    pub info: T,
}

impl FromGrammar<grammar::CharacterLiteral> for Character<()> {
    fn transform(item: rust_sitter::Spanned<grammar::CharacterLiteral>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;
        Character {
            character: value.0.value, // CharacterLiteral(Spanned<char>) - extract the char value
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl FromTokens<Token> for Character<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        if let Some(Token::Character { value, .. }) = tokens.next() {
            assert_eq!(value.len(), 3);

            let character: char = value
                .strip_prefix('\'')
                .unwrap()
                .strip_suffix('\'')
                .unwrap()
                .parse()
                .unwrap();
            Ok(Character {
                character,
                position,
                info: (),
            }
            .into())
        } else {
            Err(ParseError {
                message: "Tried to parse Character from non Character token".into(),
                position: Some(position),
            })
        }
    }
}

impl From<Character<()>> for AstNode {
    fn from(character: Character<()>) -> AstNode {
        AstNode::Character(character)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::test_helpers::*;

    #[test]
    fn test_parse_simple_character() {
        let result = parse_character("'a'").unwrap();
        assert_eq!(result.character, 'a');
    }

    #[test]
    fn test_parse_escaped_character() {
        let result = parse_character("'\t'").unwrap();
        assert_eq!(result.character, '\t');
    }

    #[test]
    fn test_parse_newline_character() {
        let result = parse_character("'\n'").unwrap();
        assert_eq!(result.character, '\n');
    }

    #[test]
    fn test_parse_carriage_return_character() {
        let result = parse_character("'\r'").unwrap();
        assert_eq!(result.character, '\r');
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        // Test that invalid character formats fail gracefully
        assert!(parse_character("'unclosed").is_err());
        assert!(parse_character("''").is_err()); // Empty character
        assert!(parse_character("").is_err()); // Empty string
    }
}
