use crate::{
    lexer::Token,
    parser::{
        ast::{AstNode, Expression, Id, TypeName},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Initialisation<T> {
    pub id: Id<T>,
    pub mutable: bool,
    pub type_name: Option<TypeName>,
    pub value: Expression<T>,
    pub info: T,
}

impl FromTokens<Token> for Initialisation<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        Comb::LET.parse(tokens)?;

        let mutable = matches!(tokens.peek(), Some(Token::Mut { .. }));

        let matcher = !Comb::MUT
            >> Comb::ID
            >> !(Comb::COLON >> Comb::TYPE_NAME)
            >> Comb::ASSIGN
            >> Comb::EXPR;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(id)) = result.first() else {
            unreachable!()
        };

        let mut type_name = None;

        let value: Expression<()>;

        match result.get(1) {
            Some(AstNode::TypeName(type_)) => {
                type_name = Some(type_.clone());

                let Some(AstNode::Expression(expr)) = result.get(2) else {
                    unreachable!()
                };
                value = expr.clone();
            }
            Some(AstNode::Expression(expr)) => {
                value = expr.clone();
            }
            _ => unreachable!(),
        }

        Ok(Initialisation {
            id: id.clone(),
            mutable,
            value: value.clone(),
            type_name,
            info: (),
        }
        .into())
    }
}

impl From<Initialisation<()>> for AstNode {
    fn from(value: Initialisation<()>) -> Self {
        AstNode::Initialization(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::ast::Num,
    };

    use super::*;

    #[test]
    fn test_simple_initialisation() {
        let mut tokens = Lexer::new("let foo = 42;")
            .lex()
            .expect("should work")
            .into();

        let result = Initialisation::parse(&mut tokens);

        assert_eq!(
            Ok(Initialisation {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                mutable: false,
                type_name: None,
                value: Expression::Num(Num::Integer(42, (), Span::default())),
                info: ()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_initialisation_with_typename() {
        let mut tokens = Lexer::new("let foo: i32 = 42;")
            .lex()
            .expect("should work")
            .into();

        let result = Initialisation::parse(&mut tokens);

        assert_eq!(
            Ok(Initialisation {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                mutable: false,
                type_name: Some(TypeName::Literal("i32".into())),
                value: Expression::Num(Num::Integer(42, (), Span::default())),
                info: ()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_mutable_initialisation() {
        let mut tokens = Lexer::new("let mut foo = 42;")
            .lex()
            .expect("should work")
            .into();

        let result = Initialisation::parse(&mut tokens);

        assert_eq!(
            Ok(Initialisation {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                mutable: true,
                type_name: None,
                value: Expression::Num(Num::Integer(42, (), Span::default())),
                info: ()
            }
            .into()),
            result
        )
    }
}
