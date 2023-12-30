use crate::{
    lexer::{TokenKind, Tokens},
    parser::{
        ast::{AstNode, Id, TypeName},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDeclaration {
    id: Id,
    fields: Vec<StructFieldDeclaration>,
}

impl FromTokens<TokenKind> for StructDeclaration {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError> {
        let matcher = Comb::STRUCT_KEYWORD
            >> Comb::ID
            >> Comb::LBRACE
            >> (Comb::STRUCT_FIELD_DECLARATION ^ ())
            >> Comb::RBRACE;

        let mut result = matcher.parse(tokens)?.into_iter();

        let Some(AstNode::Id(id)) = result.next() else {
            unreachable!()
        };

        let mut fields = vec![];

        while let Some(AstNode::StructFieldDeclaration(field)) = result.next() {
            fields.push(field);
        }

        Ok(StructDeclaration { id, fields }.into())
    }
}

impl From<StructDeclaration> for AstNode {
    fn from(value: StructDeclaration) -> Self {
        Self::StructDeclaration(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldDeclaration {
    name: Id,
    type_name: TypeName,
}

impl FromTokens<TokenKind> for StructFieldDeclaration {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError> {
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
        }
        .into())
    }
}

impl From<StructFieldDeclaration> for AstNode {
    fn from(value: StructFieldDeclaration) -> Self {
        Self::StructFieldDeclaration(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
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
                id: Id("Foo".into()),
                fields: vec![]
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
                id: Id("Foo".into()),
                fields: vec![StructFieldDeclaration {
                    name: Id("foo".into()),
                    type_name: TypeName::Literal("u32".into())
                }]
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
                id: Id("Foo".into()),
                fields: vec![
                    StructFieldDeclaration {
                        name: Id("foo".into()),
                        type_name: TypeName::Literal("u32".into())
                    },
                    StructFieldDeclaration {
                        name: Id("baz".into()),
                        type_name: TypeName::Array(Box::new(TypeName::Literal("f64".into())))
                    }
                ]
            }
            .into()),
            result
        );
    }
}
