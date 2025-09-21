use crate::grammar::{self, BooleanLiteral, FromGrammar};
use crate::lexer::Span;

use super::AstNode;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Bool<T> {
    pub value: bool,
    pub position: Span,
    pub info: T,
}

impl FromGrammar<grammar::BooleanLiteral> for Bool<()> {
    fn transform(item: rust_sitter::Spanned<grammar::BooleanLiteral>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;
        Bool {
            value: matches!(value, BooleanLiteral::True),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl From<Bool<()>> for AstNode {
    fn from(bool: Bool<()>) -> AstNode {
        AstNode::Bool(bool)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_helpers::*;

    #[test]
    fn test_parse_true() {
        let result = parse_bool("true").unwrap();
        assert_eq!(result.value, true);
    }

    #[test]
    fn test_parse_false() {
        let result = parse_bool("false").unwrap();
        assert_eq!(result.value, false);
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        assert!(parse_bool("True").is_err()); // Wrong case
        assert!(parse_bool("FALSE").is_err()); // Wrong case
        assert!(parse_bool("").is_err()); // Empty string
    }
}
