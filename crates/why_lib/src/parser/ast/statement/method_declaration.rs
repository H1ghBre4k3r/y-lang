use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Id, TypeName},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MethodDeclaration<T> {
    pub id: Id<T>,
    pub parameter_types: Vec<TypeName>,
    pub return_type: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for MethodDeclaration<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::DECLARE_KEYWORD
            >> Comb::ID
            >> Comb::LPAREN
            >> (Comb::TYPE_NAME % Comb::COMMA)
            >> Comb::RPAREN;

        let mut result = matcher.parse(tokens)?.into_iter();

        let Some(AstNode::Id(id)) = result.next() else {
            unreachable!()
        };

        let mut parameter_types = vec![];

        while let Some(AstNode::TypeName(parameter)) = result.next() {
            parameter_types.push(parameter);
        }
        assert!(result.next().is_none());

        let matcher = Comb::COLON >> Comb::TYPE_NAME >> Comb::SEMI;

        let mut result = matcher.parse(tokens)?.into_iter();

        let Some(AstNode::TypeName(return_type)) = result.next() else {
            unreachable!()
        };

        let end = tokens.prev_span()?;

        Ok(MethodDeclaration {
            id,
            parameter_types,
            return_type,
            info: (),
            position: position.merge(&end),
        }
        .into())
    }
}

impl From<MethodDeclaration<()>> for AstNode {
    fn from(value: MethodDeclaration<()>) -> Self {
        AstNode::MethodDeclaration(value)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{
        lexer::{Lexer, Span},
        parser::{
            ast::{AstNode, Id, TypeName},
            FromTokens,
        },
    };

    use super::MethodDeclaration;

    #[test]
    fn test_simple_method_declaration() -> Result<()> {
        let mut tokens = Lexer::new("declare foo(): void;").lex()?.into();

        let result = MethodDeclaration::parse(&mut tokens)?;

        assert_eq!(
            result,
            AstNode::MethodDeclaration(MethodDeclaration {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                parameter_types: vec![],
                return_type: TypeName::Literal("void".into(), Span::default()),
                info: (),
                position: Span::default()
            })
        );
        Ok(())
    }

    #[test]
    fn test_complex_method_declaration() -> Result<()> {
        let mut tokens = Lexer::new("declare foo(i64, (i64, f64)): i64;")
            .lex()?
            .into();

        let result = MethodDeclaration::parse(&mut tokens)?;

        assert_eq!(
            result,
            AstNode::MethodDeclaration(MethodDeclaration {
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
            })
        );
        Ok(())
    }
}
