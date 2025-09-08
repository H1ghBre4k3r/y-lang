use std::fmt::Display;

use crate::{
    grammar::{self, FromGrammar},
    lexer::{GetPosition, Span, Token},
    parser::{combinators::Comb, FromTokens, ParseError, ParseState},
};

use super::{AstNode, Id};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TypeName {
    Literal(String, Span),
    Fn {
        params: Vec<TypeName>,
        return_type: Box<TypeName>,
        position: Span,
    },
    Tuple(Vec<TypeName>, Span),
    Array(Box<TypeName>, Span),
    Reference(Box<TypeName>, Span),
}

impl TypeName {
    pub fn position(&self) -> Span {
        match self {
            TypeName::Literal(_, position) => position.clone(),
            TypeName::Fn { position, .. } => position.clone(),
            TypeName::Tuple(_, position) => position.clone(),
            TypeName::Array(_, position) => position.clone(),
            TypeName::Reference(_, position) => position.clone(),
        }
    }
}

impl Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeName::Literal(lit, _) => f.write_str(lit.as_str()),
            TypeName::Fn {
                params,
                return_type,
                ..
            } => f.write_fmt(format_args!(
                "({}) -> {return_type}",
                params
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            TypeName::Tuple(lits, _) => f.write_fmt(format_args!(
                "({})",
                lits.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            TypeName::Array(el, _) => f.write_fmt(format_args!("[{el}]")),
            TypeName::Reference(el, _) => f.write_fmt(format_args!("&{el}")),
        }
    }
}

impl From<&TypeName> for TypeName {
    fn from(value: &TypeName) -> Self {
        value.clone()
    }
}

impl FromGrammar<grammar::TypeName> for TypeName {
    fn transform(item: rust_sitter::Spanned<grammar::TypeName>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        match value {
            grammar::TypeName::LiteralType(literal_type) => {
                let id = Id::transform(literal_type.typename, source);
                TypeName::Literal(id.name, Span::new(span, source))
            }
            grammar::TypeName::ArrayType(array_type) => TypeName::Array(
                Box::new(TypeName::transform(*array_type.inner, source)),
                Span::new(span, source),
            ),
            grammar::TypeName::ReferenceType(reference_type) => TypeName::Reference(
                Box::new(TypeName::transform(*reference_type.inner, source)),
                Span::new(span, source),
            ),
            grammar::TypeName::FunctionType(function_type) => TypeName::Fn {
                params: function_type
                    .params
                    .types
                    .into_iter()
                    .map(|param| TypeName::transform(param, source))
                    .collect(),
                return_type: Box::new(TypeName::transform(*function_type.return_type, source)),
                position: Span::new(span, source),
            },
            grammar::TypeName::TupleType(tuple_type) => TypeName::Tuple(
                tuple_type
                    .types
                    .into_iter()
                    .map(|t| TypeName::transform(t, source))
                    .collect(),
                Span::new(span, source),
            ),
        }
    }
}

impl FromTokens<Token> for TypeName {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.peek().map(|token| token.position());
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
            position,
        })
    }
}

impl TypeName {
    fn parse_literal(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let index = tokens.get_index();

        let matcher = !Comb::ID;

        let result = matcher.parse(tokens).inspect_err(|_| {
            tokens.set_index(index);
        })?;

        let Some(AstNode::Id(type_name)) = result.first() else {
            return Err(ParseError {
                message: "Could not parse type literal".into(),
                position: None,
            });
        };

        Ok(TypeName::Literal(type_name.name.clone(), position).into())
    }

    fn parse_tuple(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let index = tokens.get_index();

        let matcher = Comb::LPAREN >> (Comb::TYPE_NAME % Comb::COMMA) >> Comb::RPAREN;

        let result = matcher.parse(tokens).inspect_err(|_| {
            tokens.set_index(index);
        })?;

        let mut elems = vec![];

        for type_name in &result {
            let AstNode::TypeName(type_name) = type_name else {
                unreachable!()
            };
            elems.push(type_name.clone());
        }

        let Span { end, .. } = tokens.prev_span()?;

        Ok(TypeName::Tuple(
            elems,
            Span {
                start: position.start,
                end,
                source: position.source,
            },
        )
        .into())
    }

    fn parse_fn(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let index = tokens.get_index();

        let AstNode::TypeName(TypeName::Tuple(params, _)) = Self::parse_tuple(tokens)? else {
            unreachable!()
        };

        let matcher = Comb::SMALL_RIGHT_ARROW >> Comb::TYPE_NAME;

        let result = matcher.parse(tokens).inspect_err(|_| {
            tokens.set_index(index);
        })?;

        let Some(AstNode::TypeName(type_name)) = result.first() else {
            unreachable!()
        };

        let Span { end, .. } = tokens.prev_span()?;
        Ok(TypeName::Fn {
            params,
            return_type: Box::new(type_name.clone()),
            position: Span {
                start: position.start,
                end,
                source: position.source,
            },
        }
        .into())
    }

    fn parse_array(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let index = tokens.get_index();

        let matcher = Comb::LBRACKET >> Comb::TYPE_NAME >> Comb::RBRACKET;

        let result = matcher.parse(tokens).inspect_err(|_| {
            tokens.set_index(index);
        })?;

        let Some(AstNode::TypeName(type_name)) = result.first() else {
            unreachable!()
        };

        let Span { end, .. } = tokens.prev_span()?;
        Ok(TypeName::Array(
            Box::new(type_name.clone()),
            Span {
                start: position.start,
                end,
                source: position.source,
            },
        )
        .into())
    }

    fn parse_reference(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let index = tokens.get_index();

        let matcher = Comb::AMPERSAND >> Comb::TYPE_NAME;

        let result = matcher.parse(tokens).inspect_err(|_| {
            tokens.set_index(index);
        })?;

        let Some(AstNode::TypeName(type_name)) = result.first() else {
            unreachable!()
        };

        let Span { end, .. } = tokens.prev_span()?;
        Ok(TypeName::Reference(
            Box::new(type_name.clone()),
            Span {
                start: position.start,
                end,
                source: position.source,
            },
        )
        .into())
    }
}

impl From<TypeName> for AstNode {
    fn from(value: TypeName) -> Self {
        Self::TypeName(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::FromTokens,
    };

    use super::TypeName;

    #[test]
    fn test_parse_simple_literal() {
        let mut tokens = Lexer::new("i32")
            .lex()
            .expect("something went wrong")
            .into();

        let result = TypeName::parse(&mut tokens);
        assert_eq!(
            Ok(TypeName::Literal("i32".into(), Span::default()).into()),
            result
        );
    }

    #[test]
    fn test_parse_simple_tuple() {
        let mut tokens = Lexer::new("(i32, i32)")
            .lex()
            .expect("something went wrong")
            .into();

        let result = TypeName::parse(&mut tokens);
        assert_eq!(
            Ok(TypeName::Tuple(
                vec![TypeName::Literal("i32".into(), Span::default()); 2],
                Span::default()
            )
            .into()),
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
                return_type: Box::new(TypeName::Literal("i32".into(), Span::default())),
                position: Span::default()
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
            Ok(TypeName::Reference(
                Box::new(TypeName::Literal("i32".into(), Span::default())),
                Span::default()
            )
            .into()),
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
            Ok(TypeName::Reference(
                Box::new(TypeName::Tuple(
                    vec![TypeName::Literal("i32".into(), Span::default()); 2],
                    Span::default()
                )),
                Span::default()
            )
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
            Ok(TypeName::Tuple(
                vec![
                    TypeName::Reference(
                        Box::new(TypeName::Literal("i32".into(), Span::default())),
                        Span::default()
                    );
                    2
                ],
                Span::default()
            )
            .into()),
            result
        )
    }
}
