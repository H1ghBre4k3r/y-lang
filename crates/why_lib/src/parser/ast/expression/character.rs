use crate::lexer::{Span, Token};
use crate::parser::ast::AstNode;
use crate::parser::{FromTokens, ParseError, ParseState};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Character<T> {
    pub character: char,
    pub position: Span,
    pub info: T,
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
    use crate::lexer::{Lexer, Span};
    use crate::parser::ast::Character;
    use crate::parser::FromTokens;

    #[test]
    fn test_parse_simple() {
        let mut tokens = Lexer::new("'a'").lex().expect("should work").into();
        let result = Character::parse(&mut tokens).expect("should work");

        assert_eq!(
            result,
            Character {
                character: 'a',
                info: (),
                position: Span::default()
            }
            .into()
        )
    }

    #[test]
    fn test_parse_escaped() {
        let mut tokens = Lexer::new("'\t'").lex().expect("should work").into();
        let result = Character::parse(&mut tokens).expect("should work");

        assert_eq!(
            result,
            Character {
                character: '\t',
                info: (),
                position: Span::default()
            }
            .into()
        )
    }
}
