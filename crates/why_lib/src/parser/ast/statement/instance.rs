use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Function, TypeName},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

use super::MethodDeclaration;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Instance<T> {
    pub name: TypeName,
    pub functions: Vec<Function<T>>,
    pub declarations: Vec<MethodDeclaration<T>>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Instance<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::INSTANCE_KEYWORD
            >> Comb::TYPE_NAME
            >> Comb::LBRACE
            >> ((Comb::FUNCTION | Comb::METHOD_DECLARATION) ^ Comb::RBRACE);
        let mut result = matcher.parse(tokens)?.into_iter();

        let Some(AstNode::TypeName(name)) = result.next() else {
            unreachable!();
        };

        let mut functions = vec![];
        let mut declarations = vec![];

        for next in result.by_ref() {
            match next {
                AstNode::Function(function) => functions.push(function),
                AstNode::MethodDeclaration(declaration) => declarations.push(declaration),
                _ => unreachable!(),
            }
        }

        assert!(result.next().is_none());

        let Span { end, .. } = tokens.prev_span()?;

        Ok(Instance {
            name,
            functions,
            declarations,
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
            ast::{Expression, Function, Id, MethodDeclaration, Num, Statement, TypeName},
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
                declarations: vec![],
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
                declarations: vec![],
                info: (),
                position: Span::default()
            }
            .into()
        );

        Ok(())
    }

    #[test]
    fn test_complext_instance() -> Result<()> {
        let mut tokens = Lexer::new(
            "instance Foo {
            fn bar(): i64 {
                42
            }

            declare foo(i64, (i64, f64)): i64;
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
                declarations: vec![MethodDeclaration {
                    id: Id {
                        name: "foo".into(),
                        info: (),
                        position: Span::default()
                    },
                    parameter_types: vec![
                        TypeName::Literal("i64".into(), Span::default()),
                        TypeName::Tuple(
                            vec![
                                TypeName::Literal("i64".into(), Span::default()),
                                TypeName::Literal("f64".into(), Span::default())
                            ],
                            Span::default()
                        )
                    ],
                    return_type: TypeName::Literal("i64".into(), Span::default()),
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
