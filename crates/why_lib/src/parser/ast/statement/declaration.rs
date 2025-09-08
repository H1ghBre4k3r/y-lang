use crate::{
    grammar::{self, FromGrammar},
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Id, TypeName},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Declaration<T> {
    pub name: Id<T>,
    pub type_name: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::Declaration> for Declaration<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Declaration>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        Declaration {
            name: Id::transform(value.name, source),
            type_name: TypeName::transform(value.type_annotation.type_name, source),
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl FromTokens<Token> for Declaration<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::DECLARE_KEYWORD >> Comb::ID >> Comb::COLON >> Comb::TYPE_NAME;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(name)) = result.first().cloned() else {
            unreachable!()
        };

        let Some(AstNode::TypeName(type_name)) = result.get(1).cloned() else {
            unreachable!()
        };

        Ok(Declaration {
            name,
            type_name,
            info: (),
            position,
        }
        .into())
    }
}

impl From<Declaration<()>> for AstNode {
    fn from(value: Declaration<()>) -> Self {
        AstNode::Declaration(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::{
            ast::{Id, TypeName},
            FromTokens,
        },
    };

    use super::Declaration;

    #[test]
    fn test_parse_simple_declaration() {
        let mut tokens = Lexer::new("declare foo: i32")
            .lex()
            .expect("something went wrong")
            .into();

        let result = Declaration::parse(&mut tokens);

        assert_eq!(
            Ok(Declaration {
                name: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                type_name: TypeName::Literal("i32".into(), Span::default()),
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_tuple_declaration() {
        let mut tokens = Lexer::new("declare foo: (i32, i32)")
            .lex()
            .expect("something went wrong")
            .into();

        let result = Declaration::parse(&mut tokens);
        assert_eq!(
            Ok(Declaration {
                name: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                type_name: TypeName::Tuple(
                    vec![TypeName::Literal("i32".into(), Span::default()); 2],
                    Span::default()
                ),
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_function_declaration() {
        let mut tokens = Lexer::new("declare foo: (i32, i32) -> i32")
            .lex()
            .expect("something went wrong")
            .into();

        let result = Declaration::parse(&mut tokens);
        assert_eq!(
            Ok(Declaration {
                name: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                type_name: TypeName::Fn {
                    params: vec![TypeName::Literal("i32".into(), Span::default()); 2],
                    return_type: Box::new(TypeName::Literal("i32".into(), Span::default())),
                    position: Span::default()
                },
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }
}
