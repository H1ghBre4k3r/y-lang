use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Id, TypeName},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDeclaration<T> {
    pub id: Id<T>,
    pub fields: Vec<StructFieldDeclaration<T>>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for StructDeclaration<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::STRUCT_KEYWORD
            >> Comb::ID
            >> Comb::LBRACE
            >> (Comb::STRUCT_FIELD_DECLARATION ^ Comb::RBRACE);

        let mut result = matcher.parse(tokens)?.into_iter();

        let Some(AstNode::Id(id)) = result.next() else {
            unreachable!()
        };

        let mut fields = vec![];

        while let Some(AstNode::StructFieldDeclaration(field)) = result.next() {
            fields.push(field);
        }

        Ok(StructDeclaration {
            id,
            fields,
            info: (),
            position,
        }
        .into())
    }
}

impl From<StructDeclaration<()>> for AstNode {
    fn from(value: StructDeclaration<()>) -> Self {
        Self::StructDeclaration(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldDeclaration<T> {
    pub name: Id<T>,
    pub type_name: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for StructFieldDeclaration<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::ID >> Comb::COLON >> Comb::TYPE_NAME >> Comb::SEMI;
        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(name)) = result.first() else {
            unreachable!()
        };

        let Some(AstNode::TypeName(type_name)) = result.get(1) else {
            unreachable!()
        };

        Ok(StructFieldDeclaration {
            name: name.clone(),
            type_name: type_name.clone(),
            info: (),
            position,
        }
        .into())
    }
}

impl From<StructFieldDeclaration<()>> for AstNode {
    fn from(value: StructFieldDeclaration<()>) -> Self {
        Self::StructFieldDeclaration(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::{
            ast::{Id, StructFieldDeclaration, TypeName},
            FromTokens,
        },
    };

    use super::StructDeclaration;

    #[test]
    fn parse_empty_struct() {
        let mut tokens = Lexer::new("struct Foo {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = StructDeclaration::parse(&mut tokens);

        assert_eq!(
            Ok(StructDeclaration {
                id: Id {
                    name: "Foo".into(),
                    info: (),
                    position: Span::default()
                },
                fields: vec![],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        );
    }

    #[test]
    fn parse_struct_with_single_field() {
        let mut tokens = Lexer::new(
            "struct Foo {
            foo: u32;
        }",
        )
        .lex()
        .expect("something is wrong")
        .into();

        let result = StructDeclaration::parse(&mut tokens);

        assert_eq!(
            Ok(StructDeclaration {
                id: Id {
                    name: "Foo".into(),
                    info: (),
                    position: Span::default()
                },
                fields: vec![StructFieldDeclaration {
                    name: Id {
                        name: "foo".into(),
                        info: (),
                        position: Span::default()
                    },
                    type_name: TypeName::Literal("u32".into()),
                    info: (),
                    position: Span::default()
                }],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        );
    }

    #[test]
    fn parse_struct_with_multiple_fields() {
        let mut tokens = Lexer::new(
            "struct Foo {
            foo: u32;
            baz: [f64];
        }",
        )
        .lex()
        .expect("something is wrong")
        .into();

        let result = StructDeclaration::parse(&mut tokens);

        assert_eq!(
            Ok(StructDeclaration {
                id: Id {
                    name: "Foo".into(),
                    info: (),
                    position: Span::default()
                },
                fields: vec![
                    StructFieldDeclaration {
                        name: Id {
                            name: "foo".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("u32".into()),
                        info: (),
                        position: Span::default()
                    },
                    StructFieldDeclaration {
                        name: Id {
                            name: "baz".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Array(Box::new(TypeName::Literal("f64".into()))),
                        info: (),
                        position: Span::default()
                    }
                ],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        );
    }
}
