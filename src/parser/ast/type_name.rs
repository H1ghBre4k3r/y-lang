use crate::lexer::Token;
use crate::parser::combinators::Comb;
use crate::parser::FromTokens;
use crate::parser::ParseError;
use crate::parser::ParseState;

use super::AstNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeName {
    Literal(String),
    Fn {
        params: Vec<TypeName>,
        return_type: Box<TypeName>,
    },
    Tuple(Vec<TypeName>),
    Array(Box<TypeName>),
    Reference(Box<TypeName>),
}

impl From<&TypeName> for TypeName {
    fn from(value: &TypeName) -> Self {
        value.clone()
    }
}

impl FromTokens<Token> for TypeName {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        if let Ok(type_name) = Self::parse_literal(tokens) {
            return Ok(type_name);
        };

        if let Ok(function) = Self::parse_fn(tokens) {
            return Ok(function);
        };

        if let Ok(tuple) = Self::parse_tuple(tokens) {
            return Ok(tuple);
        };

        if let Ok(array) = Self::parse_array(tokens) {
            return Ok(array);
        }

        if let Ok(reference) = Self::parse_reference(tokens) {
            return Ok(reference);
        }

        Err(ParseError {
            message: "could not parse type name".into(),
            position: None,
        })
    }
}

impl TypeName {
    fn parse_literal(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let matcher = !Comb::ID;

        let result = matcher.parse(tokens).map_err(|e| {
            tokens.set_index(index);
            e
        })?;

        let Some(AstNode::Id(type_name)) = result.first() else {
            return Err(ParseError {
                message: "Could not parse type literal".into(),
                position: None,
            });
        };

        Ok(TypeName::Literal(type_name.name.clone()).into())
    }

    fn parse_tuple(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let matcher = Comb::LPAREN >> (Comb::TYPE_NAME % Comb::COMMA) >> Comb::RPAREN;

        let result = matcher.parse(tokens).map_err(|e| {
            tokens.set_index(index);
            e
        })?;

        let mut elems = vec![];

        for type_name in &result {
            let AstNode::TypeName(type_name) = type_name else {
                unreachable!()
            };
            elems.push(type_name.clone());
        }

        Ok(TypeName::Tuple(elems).into())
    }

    fn parse_fn(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let AstNode::TypeName(TypeName::Tuple(params)) = Self::parse_tuple(tokens)? else {
            unreachable!()
        };

        let matcher = Comb::SMALL_RIGHT_ARROW >> Comb::TYPE_NAME;

        let result = matcher.parse(tokens).map_err(|e| {
            tokens.set_index(index);
            e
        })?;

        let Some(AstNode::TypeName(type_name)) = result.first() else {
            unreachable!()
        };

        Ok(TypeName::Fn {
            params,
            return_type: Box::new(type_name.clone()),
        }
        .into())
    }

    fn parse_array(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let matcher = Comb::LBRACKET >> Comb::TYPE_NAME >> Comb::RBRACKET;

        let result = matcher.parse(tokens).map_err(|e| {
            tokens.set_index(index);
            e
        })?;

        let Some(AstNode::TypeName(type_name)) = result.first() else {
            unreachable!()
        };

        Ok(TypeName::Array(Box::new(type_name.clone())).into())
    }

    fn parse_reference(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let matcher = Comb::AMPERSAND >> Comb::TYPE_NAME;

        let result = matcher.parse(tokens).map_err(|e| {
            tokens.set_index(index);
            e
        })?;

        let Some(AstNode::TypeName(type_name)) = result.first() else {
            unreachable!()
        };

        Ok(TypeName::Reference(Box::new(type_name.clone())).into())
    }
}

impl From<TypeName> for AstNode {
    fn from(value: TypeName) -> Self {
        Self::TypeName(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::FromTokens};

    use super::TypeName;

    #[test]
    fn test_parse_simple_literal() {
        let mut tokens = Lexer::new("i32")
            .lex()
            .expect("something went wrong")
            .into();

        let result = TypeName::parse(&mut tokens);
        assert_eq!(Ok(TypeName::Literal("i32".into()).into()), result);
    }

    #[test]
    fn test_parse_simple_tuple() {
        let mut tokens = Lexer::new("(i32, i32)")
            .lex()
            .expect("something went wrong")
            .into();

        let result = TypeName::parse(&mut tokens);
        assert_eq!(
            Ok(TypeName::Tuple(vec![TypeName::Literal("i32".into()); 2]).into()),
            result
        );
    }

    #[test]
    fn test_parse_simple_function() {
        let mut tokens = Lexer::new("() -> i32")
            .lex()
            .expect("something went wrong")
            .into();

        let result = TypeName::parse(&mut tokens);
        assert_eq!(
            Ok(TypeName::Fn {
                params: vec![],
                return_type: Box::new(TypeName::Literal("i32".into()))
            }
            .into()),
            result
        );
    }

    #[test]
    fn test_parse_simple_reference() {
        let mut tokens = Lexer::new("&i32")
            .lex()
            .expect("something went wrong")
            .into();

        let result = TypeName::parse(&mut tokens);
        assert_eq!(
            Ok(TypeName::Reference(Box::new(TypeName::Literal("i32".into()))).into()),
            result
        );
    }

    #[test]
    fn test_parse_reference_of_tuple() {
        let mut tokens = Lexer::new("&(i32, i32)")
            .lex()
            .expect("something went wrong")
            .into();

        let result = TypeName::parse(&mut tokens);

        assert_eq!(
            Ok(TypeName::Reference(Box::new(TypeName::Tuple(vec![
                TypeName::Literal(
                    "i32".into()
                );
                2
            ])))
            .into()),
            result
        );
    }

    #[test]
    fn test_parse_tuple_of_references() {
        let mut tokens = Lexer::new("(&i32, &i32)")
            .lex()
            .expect("something went wrong")
            .into();

        let result = TypeName::parse(&mut tokens);

        assert_eq!(
            Ok(TypeName::Tuple(vec![
                TypeName::Reference(Box::new(TypeName::Literal(
                    "i32".into()
                )));
                2
            ])
            .into()),
            result
        )
    }
}
