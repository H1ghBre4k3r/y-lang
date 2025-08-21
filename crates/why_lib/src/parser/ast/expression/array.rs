use crate::{
    lexer::{Span, Token},
    parser::{ast::AstNode, direct_parsing::DirectParser, FromTokens, ParseError, ParseState},
};

use super::{Expression, Num};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Array<T> {
    Literal {
        values: Vec<Expression<T>>,
        info: T,
        position: Span,
    },
    Default {
        initial_value: Box<Expression<T>>,
        length: Num<T>,
        info: T,
        position: Span,
    },
}

impl<T> Array<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Array::Literal { info, .. } => info.clone(),
            Array::Default { info, .. } => info.clone(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Array::Literal { position, .. } => position.clone(),
            Array::Default { position, .. } => position.clone(),
        }
    }
}

impl FromTokens<Token> for Array<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;
        let start = tokens.get_index();
        
        // Parse opening bracket
        DirectParser::expect_lbracket(tokens)?;
        
        // Try to parse first expression to see if this is an empty array or not
        let saved_pos = tokens.get_index();
        
        // Check for empty array first
        if DirectParser::expect_rbracket(tokens).is_ok() {
            return Ok(Array::Literal {
                values: vec![],
                info: (),
                position,
            }.into());
        }
        
        // Reset and try to parse expressions
        tokens.set_index(saved_pos);
        
        // Try to parse first expression
        let first_expr = match Expression::parse(tokens) {
            Ok(AstNode::Expression(expr)) => expr,
            Ok(_) => unreachable!("Expression::parse should return Expression"),
            Err(_) => {
                tokens.set_index(start);
                return Err(ParseError {
                    message: "Expected expression in array".to_string(),
                    position: Some(position),
                });
            }
        };
        
        // Check if this is a default array [expr; num]
        if DirectParser::parse_terminal(tokens, |t| matches!(t, Token::Semicolon { .. }), "semicolon").is_ok() {
            let length = match Num::parse(tokens)? {
                AstNode::Num(num) => num,
                _ => unreachable!("Num::parse should return Num"),
            };
            
            DirectParser::expect_rbracket(tokens)?;
            
            return Ok(Array::Default {
                initial_value: Box::new(first_expr),
                length,
                info: (),
                position,
            }.into());
        }
        
        // Otherwise, this is a literal array [expr, expr, ...]
        let mut expressions = vec![first_expr];
        
        // Parse additional expressions separated by commas  
        loop {
            let before_comma = tokens.get_index();
            if DirectParser::expect_comma(tokens).is_err() {
                // No more commas, we're done
                break;
            }
            
            match Expression::parse(tokens) {
                Ok(AstNode::Expression(expr)) => expressions.push(expr),
                Ok(_) => unreachable!("Expression::parse should return Expression"),
                Err(_) => {
                    // Failed to parse expression after comma, backtrack
                    tokens.set_index(before_comma);
                    break;
                }
            }
        }
        
        DirectParser::expect_rbracket(tokens)?;
        
        Ok(Array::Literal {
            values: expressions,
            info: (),
            position,
        }.into())
    }
}

impl From<Array<()>> for AstNode {
    fn from(value: Array<()>) -> Self {
        Self::Array(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, Span};

    use super::*;

    #[test]
    fn test_empty_array() {
        let mut tokens = Lexer::new("[]").lex().expect("something is wrong").into();

        let result = Array::parse(&mut tokens);
        assert_eq!(
            Ok(Array::Literal {
                values: vec![],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        );
    }

    #[test]
    fn test_simple_literal() {
        let mut tokens = Lexer::new("[42, 1337]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Array::parse(&mut tokens);
        assert_eq!(
            Ok(Array::Literal {
                values: vec![
                    Expression::Num(Num::Integer(42, (), Span::default())),
                    Expression::Num(Num::Integer(1337, (), Span::default())),
                ],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        );
    }

    #[test]
    fn test_simple_default() {
        let mut tokens = Lexer::new("[42; 5]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Array::parse(&mut tokens);
        assert_eq!(
            Ok(Array::Default {
                initial_value: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
                length: Num::Integer(5, (), Span::default()),
                info: (),
                position: Span::default()
            }
            .into()),
            result
        );
    }

    #[test]
    fn test_faulty_default() {
        let mut tokens = Lexer::new("[42; ]")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Array::parse(&mut tokens);
        assert!(result.is_err());
    }
}
