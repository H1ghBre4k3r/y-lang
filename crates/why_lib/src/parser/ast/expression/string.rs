use crate::grammar::{self, FromGrammar};
use crate::lexer::Span;

use super::AstNode;
use unescape::unescape;

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

impl From<AstString<()>> for AstNode {
    fn from(string: AstString<()>) -> AstNode {
        AstNode::AstString(string)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_helpers::*;

    #[test]
    fn test_empty_string_parse() {
        let result = parse_string(r#""""#).unwrap();
        assert_eq!(result.value, "");
    }

    #[test]
    fn test_simple_string_parse() {
        let result = parse_string(r#""foo""#).unwrap();
        assert_eq!(result.value, "foo");
    }

    #[test]
    fn test_string_with_spaces() {
        let result = parse_string(r#""hello world""#).unwrap();
        assert_eq!(result.value, "hello world");
    }

    #[test]
    fn test_string_with_escaped_quotes() {
        let result = parse_string(r#""this is a test\"""#).unwrap();
        assert_eq!(result.value, r#"this is a test\""#);
    }

    #[test]
    fn test_string_with_escape_sequences() {
        let result = parse_string("\"\t\n\r\"").unwrap();
        assert_eq!(result.value, "\t\n\r");
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        // Test that invalid string formats fail gracefully
        assert!(parse_string("unclosed_string").is_err());
        assert!(parse_string("").is_err());
    }
}
