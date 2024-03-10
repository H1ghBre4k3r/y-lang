use crate::{
    lexer::{Token, Tokens},
    parser::{ast::AstNode, combinators::Comb, FromTokens, ParseError},
};

use super::{Expression, Id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructInitialisation<T> {
    pub id: Id<T>,
    pub fields: Vec<StructFieldInitialisation<T>>,
    pub info: T,
}

impl FromTokens<Token> for StructInitialisation<()> {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
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

        Ok(StructInitialisation {
            id,
            fields,
            info: (),
        }
        .into())
    }
}

impl From<StructInitialisation<()>> for AstNode {
    fn from(value: StructInitialisation<()>) -> Self {
        Self::StructInitialisation(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldInitialisation<T> {
    pub name: Id<T>,
    pub value: Expression<T>,
    pub info: T,
}

impl FromTokens<Token> for StructFieldInitialisation<()> {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
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
            info: (),
        }
        .into())
    }
}

impl From<StructFieldInitialisation<()>> for AstNode {
    fn from(value: StructFieldInitialisation<()>) -> Self {
        Self::StructFieldInitialisation(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::{
            ast::{BinaryExpression, Expression, Id, Lambda, Num, Parameter},
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
                name: Id("bar".into(), ()),
                value: Expression::Num(Num::Integer(42, ())),
                info: ()
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
                id: Id("Foo".into(), ()),
                fields: vec![],
                info: ()
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
                id: Id("Foo".into(), ()),
                fields: vec![StructFieldInitialisation {
                    name: Id("bar".into(), ()),
                    value: Expression::Num(Num::Integer(42, ())),
                    info: ()
                }],
                info: ()
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
                id: Id("Foo".into(), ()),
                fields: vec![
                    StructFieldInitialisation {
                        name: Id("bar".into(), ()),
                        value: Expression::Num(Num::Integer(42, ())),
                        info: ()
                    },
                    StructFieldInitialisation {
                        name: Id("baz".into(), ()),
                        value: Expression::Lambda(Lambda {
                            parameters: vec![Parameter {
                                name: Id("x".into(), ()),
                                type_name: None,
                                info: ()
                            }],
                            expression: Box::new(Expression::Binary(Box::new(
                                BinaryExpression::Addition(
                                    Expression::Id(Id("x".into(), ())),
                                    Expression::Id(Id("x".into(), ()))
                                )
                            ))),
                            info: ()
                        }),
                        info: ()
                    }
                ],
                info: ()
            }
            .into()),
            result
        );
    }
}
