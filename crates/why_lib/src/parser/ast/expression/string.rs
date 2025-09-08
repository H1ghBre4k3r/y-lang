use crate::lexer::{Span, Token};
use crate::parser::ast::AstNode;
use crate::parser::{FromTokens, ParseError, ParseState};
use crate::grammar::{self, FromGrammar};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AstString<T> {
    pub value: String,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::StringLiteral> for AstString<()> {
    fn transform(item: rust_sitter::Spanned<grammar::StringLiteral>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;
        AstString {
            value: value.0.value, // StringLiteral(Spanned<String>) - extract the string value
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl FromTokens<Token> for AstString<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        if let Some(Token::String { value, .. }) = tokens.next() {
            let value: String = value
                .strip_prefix('\"')
                .unwrap()
                .strip_suffix('\"')
                .unwrap()
                .to_string();
            Ok(AstString {
                value,
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
impl From<AstString<()>> for AstNode {
    fn from(string: AstString<()>) -> AstNode {
        AstNode::AstString(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_empty_string_parse() {
        let mut tokens = Lexer::new("\"\"").lex().expect("should work").into();
        let result = AstString::parse(&mut tokens).expect("should work");

        assert_eq!(
            result,
            AstString {
                value: "".into(),
                info: (),
                position: Span::default()
            }
            .into()
        )
    }

    #[test]
    fn test_simple_string_parse() {
        let mut tokens = Lexer::new("\"foo\"").lex().expect("should work").into();
        let result = AstString::parse(&mut tokens).expect("should work");

        assert_eq!(
            result,
            AstString {
                value: "foo".into(),
                info: (),
                position: Span::default()
            }
            .into()
        )
    }

    #[test]
    fn test_escaped_string_parse_simple() {
        let mut tokens = Lexer::new("\"\t\"").lex().expect("should work").into();
        let result = AstString::parse(&mut tokens).expect("should work");

        assert_eq!(
            result,
            AstString {
                value: "\t".into(),
                info: (),
                position: Span::default()
            }
            .into()
        )
    }

    #[test]
    fn test_escaped_string_parse_complex() {
        let mut tokens = Lexer::new("\"this is a test\"\"")
            .lex()
            .expect("should work")
            .into();
        let result = AstString::parse(&mut tokens).expect("should work");

        assert_eq!(
            result,
            AstString {
                value: "this is a test\"".into(),
                info: (),
                position: Span::default()
            }
            .into()
        )
    }
}
