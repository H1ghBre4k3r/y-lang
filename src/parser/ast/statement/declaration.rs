use crate::{
    lexer::{TokenKind, Tokens},
    parser::{
        ast::{AstNode, Id, TypeName},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub name: Id,
    pub type_name: TypeName,
}

impl FromTokens<TokenKind> for Declaration {
    fn parse(tokens: &mut Tokens<TokenKind>) -> Result<AstNode, ParseError> {
        let matcher = Comb::DECLARE_KEYWORD >> Comb::ID >> Comb::COLON >> Comb::TYPE_NAME;

        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(name)) = result.first().cloned() else {
            unreachable!()
        };

        let Some(AstNode::TypeName(type_name)) = result.get(1).cloned() else {
            unreachable!()
        };

        Ok(Declaration { name, type_name }.into())
    }
}

impl From<Declaration> for AstNode {
    fn from(value: Declaration) -> Self {
        AstNode::Declaration(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::Lexer,
        parser::{
            ast::{Id, TypeName},
            FromTokens,
        },
    };

    use super::Declaration;

    #[test]
    fn test_parse_simple_declaration() {
        let mut tokens = Lexer::new("declare foo: i32")
            .lex()
            .expect("something went wrong")
            .into();

        let result = Declaration::parse(&mut tokens);

        assert_eq!(
            Ok(Declaration {
                name: Id("foo".into()),
                type_name: TypeName::Literal("i32".into())
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_tuple_declaration() {
        let mut tokens = Lexer::new("declare foo: (i32, i32)")
            .lex()
            .expect("something went wrong")
            .into();

        let result = Declaration::parse(&mut tokens);
        assert_eq!(
            Ok(Declaration {
                name: Id("foo".into()),
                type_name: TypeName::Tuple(vec![TypeName::Literal("i32".into()); 2])
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_parse_function_declaration() {
        let mut tokens = Lexer::new("declare foo: (i32, i32) -> i32")
            .lex()
            .expect("something went wrong")
            .into();

        let result = Declaration::parse(&mut tokens);
        assert_eq!(
            Ok(Declaration {
                name: Id("foo".into()),
                type_name: TypeName::Fn {
                    params: vec![TypeName::Literal("i32".into()); 2],
                    return_type: Box::new(TypeName::Literal("i32".into()))
                }
            }
            .into()),
            result
        )
    }
}
