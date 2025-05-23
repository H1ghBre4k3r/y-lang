use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Expression, Id, TypeName},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Constant<T> {
    pub id: Id<T>,
    pub type_name: TypeName,
    pub value: Expression<T>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Constant<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let position = tokens.span()?;

        Comb::CONST_KEYWORD.parse(tokens)?;

        let matcher = Comb::ID >> Comb::COLON >> Comb::TYPE_NAME >> Comb::ASSIGN >> Comb::EXPR;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(id)) = result.first() else {
            unreachable!()
        };

        let Some(AstNode::TypeName(type_name)) = result.get(1).cloned() else {
            unreachable!()
        };

        let Some(AstNode::Expression(value)) = result.get(2).cloned() else {
            unreachable!()
        };

        Ok(Constant {
            id: id.clone(),
            value: value.clone(),
            type_name,
            info: (),
            position,
        }
        .into())
    }
}

impl From<Constant<()>> for AstNode {
    fn from(value: Constant<()>) -> Self {
        AstNode::Constant(value)
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
    fn test_simple_constant() {
        let mut tokens = Lexer::new("const foo: i32 = 42")
            .lex()
            .expect("should work")
            .into();

        let result = Constant::parse(&mut tokens);

        assert_eq!(
            Ok(Constant {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                type_name: TypeName::Literal("i32".into(), Span::default()),
                value: Expression::Num(Num::Integer(42, (), Span::default())),
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }
}
