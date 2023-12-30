use crate::{
    lexer::{TokenKind, Tokens},
    parser::{ast::AstNode, combinators::Comb, FromTokens, ParseError},
};

use super::{Expression, Id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructInitialisation {
    pub id: Id,
    pub fields: Vec<StructFieldInitialisation>,
}

impl FromTokens<TokenKind> for StructInitialisation {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError> {
        let matcher = Comb::ID
            >> Comb::LBRACE
            >> (Comb::STRUCT_FIELD_INITIALISATION % Comb::COMMA)
            >> Comb::RBRACE;

        let mut result = matcher.parse(tokens)?.into_iter();

        let Some(AstNode::Id(id)) = result.next() else {
            unreachable!();
        };

        let mut fields = vec![];

        while let Some(AstNode::StructFieldInitialisation(field)) = result.next() {
            fields.push(field);
        }

        Ok(StructInitialisation { id, fields }.into())
    }
}

impl From<StructInitialisation> for AstNode {
    fn from(value: StructInitialisation) -> Self {
        Self::StructInitialisation(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldInitialisation {
    pub name: Id,
    pub value: Expression,
}

impl FromTokens<TokenKind> for StructFieldInitialisation {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError> {
        let matcher = Comb::ID >> Comb::COLON >> Comb::EXPR;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(name)) = result.first() else {
            unreachable!();
        };

        let Some(AstNode::Expression(value)) = result.get(1) else {
            unreachable!();
        };

        Ok(StructFieldInitialisation {
            name: name.clone(),
            value: value.clone(),
        }
        .into())
    }
}

impl From<StructFieldInitialisation> for AstNode {
    fn from(value: StructFieldInitialisation) -> Self {
        Self::StructFieldInitialisation(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::{
            ast::{Expression, Id, Lambda, Num, Parameter},
            FromTokens,
        },
    };

    use super::{StructFieldInitialisation, StructInitialisation};

    #[test]
    fn parse_simple_struct_field_initialisation() {
        let mut tokens = Lexer::new("bar: 42")
            .lex()
            .expect("something is wrong")
            .into();

        let result = StructFieldInitialisation::parse(&mut tokens);

        assert_eq!(
            Ok(StructFieldInitialisation {
                name: Id("bar".into()),
                value: Expression::Num(Num(42))
            }
            .into()),
            result
        )
    }

    #[test]
    fn parse_simple_struct_initialisation() {
        let mut tokens = Lexer::new("Foo {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = StructInitialisation::parse(&mut tokens);

        assert_eq!(
            Ok(StructInitialisation {
                id: Id("Foo".into()),
                fields: vec![]
            }
            .into()),
            result
        );
    }

    #[test]
    fn parse_struct_initialisation_with_one_field() {
        let mut tokens = Lexer::new("Foo { bar: 42 }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = StructInitialisation::parse(&mut tokens);

        assert_eq!(
            Ok(StructInitialisation {
                id: Id("Foo".into()),
                fields: vec![StructFieldInitialisation {
                    name: Id("bar".into()),
                    value: Expression::Num(Num(42))
                }]
            }
            .into()),
            result
        );
    }

    #[test]
    fn parse_struct_initialisation_with_multiple_fields() {
        let mut tokens = Lexer::new("Foo { bar: 42, baz: \\(x) => x + x }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = StructInitialisation::parse(&mut tokens);

        assert_eq!(
            Ok(StructInitialisation {
                id: Id("Foo".into()),
                fields: vec![
                    StructFieldInitialisation {
                        name: Id("bar".into()),
                        value: Expression::Num(Num(42))
                    },
                    StructFieldInitialisation {
                        name: Id("baz".into()),
                        value: Expression::Lambda(Lambda {
                            parameters: vec![Parameter {
                                name: Id("x".into()),
                                type_name: None
                            }],
                            expression: Box::new(Expression::Addition(
                                Box::new(Expression::Id(Id("x".into()))),
                                Box::new(Expression::Id(Id("x".into())))
                            ))
                        })
                    }
                ]
            }
            .into()),
            result
        );
    }
}
