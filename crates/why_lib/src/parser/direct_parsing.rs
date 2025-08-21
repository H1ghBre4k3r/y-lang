// Direct parsing utilities to replace the combinator system
// This provides a simpler approach without the complexity of the combinator system

use super::{ast::*, ParseError, ParseState, FromTokens};
use crate::lexer::{Token, GetPosition};

/// Simple parsing utilities to replace combinator functionality
pub struct DirectParser;

impl DirectParser {
    /// Parse a terminal token of specific type
    pub fn parse_terminal<F>(tokens: &mut ParseState<Token>, predicate: F, description: &str) -> Result<(), ParseError> 
    where
        F: Fn(&Token) -> bool,
    {
        match tokens.peek() {
            Some(token) if predicate(&token) => {
                tokens.next(); // Consume the token
                Ok(())
            },
            Some(token) => Err(ParseError {
                message: format!("Expected {}, got {:?}", description, token),
                position: Some(token.position()),
            }),
            None => Err(ParseError {
                message: format!("Expected {}, got EOF", description),
                position: None,
            }),
        }
    }

    /// Parse optional element
    pub fn parse_optional<T, F>(tokens: &mut ParseState<Token>, parser: F) -> Result<Option<T>, ParseError>
    where
        F: Fn(&mut ParseState<Token>) -> Result<T, ParseError>,
    {
        let start_pos = tokens.get_index();
        match parser(tokens) {
            Ok(result) => Ok(Some(result)),
            Err(_) => {
                tokens.set_index(start_pos);
                Ok(None)
            }
        }
    }

    /// Parse a sequence of items separated by a separator
    pub fn parse_separated<T, F, S>(
        tokens: &mut ParseState<Token>, 
        parser: F, 
        separator_parser: S
    ) -> Result<Vec<T>, ParseError>
    where
        F: Fn(&mut ParseState<Token>) -> Result<T, ParseError>,
        S: Fn(&mut ParseState<Token>) -> Result<(), ParseError>,
    {
        let mut items = Vec::new();
        
        // Try to parse first item
        if let Ok(first) = parser(tokens) {
            items.push(first);
            
            // Parse additional items with separator
            while separator_parser(tokens).is_ok() {
                items.push(parser(tokens)?);
            }
        }
        
        Ok(items)
    }

    /// Parse a repetition of items
    pub fn parse_repetition<T, F>(tokens: &mut ParseState<Token>, parser: F) -> Result<Vec<T>, ParseError>
    where
        F: Fn(&mut ParseState<Token>) -> Result<T, ParseError>,
    {
        let mut items = Vec::new();
        while let Ok(item) = parser(tokens) {
            items.push(item);
        }
        Ok(items)
    }

    /// Common terminal parsers
    pub fn expect_semicolon(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::Semicolon { .. }), "semicolon")
    }

    pub fn expect_lparen(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::LParen { .. }), "left parenthesis")
    }

    pub fn expect_rparen(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::RParen { .. }), "right parenthesis")
    }

    pub fn expect_lbrace(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::LBrace { .. }), "left brace")
    }

    pub fn expect_rbrace(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::RBrace { .. }), "right brace")
    }

    pub fn expect_assign(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::Assign { .. }), "assignment operator")
    }

    pub fn expect_colon(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::Colon { .. }), "colon")
    }

    pub fn expect_comma(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::Comma { .. }), "comma")
    }

    pub fn expect_let(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::Let { .. }), "let keyword")
    }

    pub fn expect_fn(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::FnKeyword { .. }), "fn keyword")
    }

    pub fn expect_minus(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::Minus { .. }), "minus operator")
    }

    pub fn expect_exclamation(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::ExclamationMark { .. }), "exclamation mark")
    }

    pub fn expect_lbracket(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::LBracket { .. }), "left bracket")
    }

    pub fn expect_rbracket(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::RBracket { .. }), "right bracket")
    }

    pub fn expect_backslash(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::Backslash { .. }), "backslash")
    }

    pub fn expect_big_right_arrow(tokens: &mut ParseState<Token>) -> Result<(), ParseError> {
        Self::parse_terminal(tokens, |t| matches!(t, Token::BigRightArrow { .. }), "big right arrow")
    }
}