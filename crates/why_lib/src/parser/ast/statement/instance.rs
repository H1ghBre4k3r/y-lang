use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Function, TypeName},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance<T> {
    pub name: TypeName,
    pub functions: Vec<Function<T>>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Instance<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::INSTANCE_KEYWORD
            >> Comb::TYPE_NAME
            >> Comb::LBRACE
            >> (Comb::FUNCTION ^ Comb::RBRACE);
        let mut result = matcher.parse(tokens)?.into_iter();

        let Some(AstNode::TypeName(name)) = result.next() else {
            unreachable!();
        };

        let mut functions = vec![];

        while let Some(AstNode::Function(function)) = result.next() {
            functions.push(function);
        }

        assert!(result.next().is_none());

        let Span { end, .. } = tokens.prev_span()?;

        Ok(Instance {
            name,
            functions,
            info: (),
            position: Span {
                start: position.start,
                end,
                source: position.source,
            },
        }
        .into())
    }
}

impl From<Instance<()>> for AstNode {
    fn from(value: Instance<()>) -> Self {
        AstNode::Instance(value)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{
        lexer::{Lexer, Span},
        parser::{
            ast::{Expression, Function, Id, Num, Statement, TypeName},
            FromTokens,
        },
    };

    use super::Instance;

    #[test]
    fn test_empty_instance() -> Result<()> {
        let mut tokens = Lexer::new("instance Foo {}").lex()?.into();

        let result = Instance::parse(&mut tokens)?;

        assert_eq!(
            result,
            Instance {
                name: TypeName::Literal("Foo".into(), Span::default()),
                functions: vec![],
                info: (),
                position: Span::default()
            }
            .into()
        );

        Ok(())
    }

    #[test]
    fn test_simple_instace() -> Result<()> {
        let mut tokens = Lexer::new(
            "instance Foo {
            fn bar(): i64 {
                42
            }
        }",
        )
        .lex()?
        .into();

        let result = Instance::parse(&mut tokens)?;

        assert_eq!(
            result,
            Instance {
                name: TypeName::Literal("Foo".into(), Span::default()),
                functions: vec![Function {
                    id: Id {
                        name: "bar".into(),
                        info: (),
                        position: Span::default()
                    },
                    parameters: vec![],
                    return_type: TypeName::Literal("i64".into(), Span::default()),
                    statements: vec![Statement::YieldingExpression(Expression::Num(
                        Num::Integer(42, (), Span::default())
                    ))],
                    info: (),
                    position: Span::default()
                }],
                info: (),
                position: Span::default()
            }
            .into()
        );

        Ok(())
    }
}
