use crate::{
    lexer::{Token, Tokens},
    parser::{
        ast::{AstNode, Statement, TypeName},
        combinators::Comb,
        FromTokens, ParseError,
    },
};

use super::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub parameters: Vec<Parameter>,
    pub return_type: TypeName,
    pub statements: Vec<Statement>,
}

impl FromTokens<Token> for Function {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::FN_KEYWORD
            >> Comb::LPAREN
            // parameter list (optional)
            >> (Comb::PARAMETER % Comb::COMMA)
            >> Comb::RPAREN
            // return type
            >> Comb::COLON
            >> Comb::TYPE_NAME
            // body of the function
            >> Comb::LBRACE
            >> (Comb::STATEMENT ^ ())
            >> Comb::RBRACE;

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let mut parameters = vec![];

        while let Some(AstNode::Parameter(param)) =
            result.next_if(|item| matches!(item, AstNode::Parameter(_)))
        {
            parameters.push(param);
        }

        let Some(AstNode::TypeName(return_type)) = result.next() else {
            unreachable!();
        };

        let mut statements = vec![];

        while let Some(AstNode::Statement(param)) =
            result.next_if(|item| matches!(item, AstNode::Statement(_)))
        {
            statements.push(param);
        }

        Ok(Function {
            parameters,
            return_type,
            statements,
        }
        .into())
    }
}

impl From<Function> for AstNode {
    fn from(value: Function) -> Self {
        AstNode::Function(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: Id,
    pub type_name: Option<TypeName>,
}

impl FromTokens<Token> for Parameter {
    fn parse(tokens: &mut Tokens<Token>) -> Result<AstNode, ParseError> {
        let matcher = Comb::ID >> !(Comb::COLON >> Comb::TYPE_NAME);
        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(name)) = result.get(0) else {
            unreachable!()
        };

        let type_name = result.get(1).map(|type_name| {
            let AstNode::TypeName(type_name) = type_name else {
                unreachable!()
            };
            type_name.clone()
        });

        Ok(Parameter {
            name: name.clone(),
            type_name,
        }
        .into())
    }
}

impl From<Parameter> for AstNode {
    fn from(value: Parameter) -> Self {
        AstNode::Parameter(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::ast::Expression};

    use super::*;

    #[test]
    fn test_simple_function() {
        let mut tokens = Lexer::new("fn (): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                parameters: vec![],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![]
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_function_with_single_param() {
        let mut tokens = Lexer::new("fn (x: i32): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                parameters: vec![Parameter {
                    name: Id("x".into()),
                    type_name: Some(TypeName::Literal("i32".into()))
                }],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![]
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_function_with_multiple_params() {
        let mut tokens = Lexer::new("fn (x: i32, y: i32): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                parameters: vec![
                    Parameter {
                        name: Id("x".into()),
                        type_name: Some(TypeName::Literal("i32".into()))
                    },
                    Parameter {
                        name: Id("y".into()),
                        type_name: Some(TypeName::Literal("i32".into()))
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![]
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_function_with_statements() {
        let mut tokens = Lexer::new("fn (x: i32, y: i32): i32 { return x + y; }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                parameters: vec![
                    Parameter {
                        name: Id("x".into()),
                        type_name: Some(TypeName::Literal("i32".into()))
                    },
                    Parameter {
                        name: Id("y".into()),
                        type_name: Some(TypeName::Literal("i32".into()))
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![Statement::Return(Expression::Addition(
                    Box::new(Expression::Id(Id("x".into()))),
                    Box::new(Expression::Id(Id("y".into()))),
                ))]
            }
            .into()),
            result
        )
    }
}
