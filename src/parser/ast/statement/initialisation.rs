use crate::{
    lexer::{TokenKind, Tokens},
    parser::{
        ast::{AstNode, Expression, Id, TypeName},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Initialisation {
    pub id: Id,
    pub mutable: bool,
    pub type_name: Option<TypeName>,
    pub value: Expression,
}

impl FromTokens<TokenKind> for Initialisation {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        Comb::LET.parse(tokens)?;

        let mutable = matches!(tokens.peek(), Some(TokenKind::Mut { .. }));

        let matcher =
            !Comb::MUT >> Comb::ID >> !(Comb::COLON >> Comb::TYPE_NAME) >> Comb::EQ >> Comb::EXPR;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(id)) = result.get(0) else {
            unreachable!()
        };

        let mut type_name = None;

        let value: Expression;

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
        }
        .into())
    }
}

impl From<Initialisation> for AstNode {
    fn from(value: Initialisation) -> Self {
        AstNode::Initialization(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::ast::Num};

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
                id: Id("foo".into()),
                mutable: false,
                type_name: None,
                value: Expression::Num(Num(42))
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
                id: Id("foo".into()),
                mutable: false,
                type_name: Some(TypeName::Literal("i32".into())),
                value: Expression::Num(Num(42))
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
                id: Id("foo".into()),
                mutable: true,
                type_name: None,
                value: Expression::Num(Num(42))
            }
            .into()),
            result
        )
    }
}
