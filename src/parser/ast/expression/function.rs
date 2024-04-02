use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Statement, TypeName},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

use super::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function<T> {
    pub id: Option<Id<T>>,
    pub parameters: Vec<FunctionParameter<T>>,
    pub return_type: TypeName,
    pub statements: Vec<Statement<T>>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Function<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::FN_KEYWORD
            >> !Comb::ID
            >> Comb::LPAREN
            // parameter list (optional)
            >> (Comb::PARAMETER % Comb::COMMA)
            >> Comb::RPAREN
            // return type
            >> Comb::COLON
            >> Comb::TYPE_NAME
            // body of the function
            >> Comb::BLOCK;

        let mut result = matcher.parse(tokens)?.into_iter().peekable();

        let id = match result.next_if(|item| matches!(item, AstNode::Id(_))) {
            Some(AstNode::Id(id)) => Some(id),
            _ => None,
        };

        let mut parameters = vec![];

        while let Some(AstNode::FunctionParameter(param)) =
            result.next_if(|item| matches!(item, AstNode::FunctionParameter(_)))
        {
            parameters.push(param);
        }

        let Some(AstNode::TypeName(return_type)) = result.next() else {
            unreachable!();
        };

        let Some(AstNode::Block(block)) = result.next() else {
            unreachable!();
        };

        Ok(Function {
            id,
            parameters,
            return_type,
            statements: block.statements,
            info: (),
            position,
        }
        .into())
    }
}

impl From<Function<()>> for AstNode {
    fn from(value: Function<()>) -> Self {
        AstNode::Function(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParameter<T> {
    pub name: Id<T>,
    pub type_name: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for FunctionParameter<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;
        let matcher = Comb::ID >> Comb::COLON >> Comb::TYPE_NAME;
        let result = matcher.parse(tokens)?;

        let Some(AstNode::Id(name)) = result.first() else {
            unreachable!()
        };

        let Some(AstNode::TypeName(type_name)) = result.get(1) else {
            unreachable!()
        };

        Ok(FunctionParameter {
            name: name.clone(),
            type_name: type_name.clone(),
            info: (),
            position,
        }
        .into())
    }
}

impl From<FunctionParameter<()>> for AstNode {
    fn from(value: FunctionParameter<()>) -> Self {
        AstNode::FunctionParameter(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::ast::{BinaryExpression, Expression},
    };

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
                id: None,
                parameters: vec![],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![],
                info: (),
                position: Span::default()
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
                id: None,
                parameters: vec![FunctionParameter {
                    name: Id {
                        name: "x".into(),
                        info: (),
                        position: Span::default()
                    },
                    type_name: TypeName::Literal("i32".into()),
                    info: (),
                    position: Span::default()
                }],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![],
                info: (),
                position: Span::default()
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
                id: None,
                parameters: vec![
                    FunctionParameter {
                        name: Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into()),
                        info: (),
                        position: Span::default()
                    },
                    FunctionParameter {
                        name: Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into()),
                        info: (),
                        position: Span::default()
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![],
                info: (),
                position: Span::default()
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
                id: None,
                parameters: vec![
                    FunctionParameter {
                        name: Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into()),
                        info: (),
                        position: Span::default()
                    },
                    FunctionParameter {
                        name: Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into()),
                        info: (),
                        position: Span::default()
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![Statement::Return(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Id(Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        }),
                        right: Expression::Id(Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        }),
                        info: (),
                    }
                )))],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }

    #[test]
    fn test_function_with_name() {
        let mut tokens = Lexer::new("fn main(x: i32, y: i32): i32 { return x + y; }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: Some(Id {
                    name: "main".into(),
                    info: (),
                    position: Span::default()
                }),
                parameters: vec![
                    FunctionParameter {
                        name: Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into()),
                        info: (),
                        position: Span::default()
                    },
                    FunctionParameter {
                        name: Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into()),
                        info: (),
                        position: Span::default()
                    }
                ],
                return_type: TypeName::Literal("i32".into()),
                statements: vec![Statement::Return(Expression::Binary(Box::new(
                    BinaryExpression::Addition {
                        left: Expression::Id(Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        }),
                        right: Expression::Id(Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        }),
                        info: (),
                    }
                )))],
                info: (),
                position: Span::default()
            }
            .into()),
            result
        )
    }
}
