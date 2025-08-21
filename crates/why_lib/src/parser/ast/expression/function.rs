use crate::{
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Statement, TypeName, Block},
        direct_parsing::DirectParser,
        FromTokens, ParseError, ParseState,
    },
};

use super::Id;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Function<T> {
    pub id: Id<T>,
    pub parameters: Vec<FunctionParameter<T>>,
    pub return_type: TypeName,
    pub statements: Vec<Statement<T>>,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for Function<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        // Parse function syntax: fn name(params): return_type { body }
        DirectParser::expect_fn(tokens)?;
        
        let id = match Id::parse(tokens)? {
            AstNode::Id(id) => id,
            _ => unreachable!("Id::parse should return Id"),
        };
        
        DirectParser::expect_lparen(tokens)?;
        
        let mut parameters = vec![];
        
        // Parse optional parameters separated by commas
        if !matches!(tokens.peek(), Some(Token::RParen { .. })) {
            // Parse first parameter
            match FunctionParameter::parse(tokens)? {
                AstNode::FunctionParameter(param) => parameters.push(param),
                _ => unreachable!("FunctionParameter::parse should return FunctionParameter"),
            }
            
            // Parse additional parameters
            while DirectParser::expect_comma(tokens).is_ok() {
                match FunctionParameter::parse(tokens)? {
                    AstNode::FunctionParameter(param) => parameters.push(param),
                    _ => unreachable!("FunctionParameter::parse should return FunctionParameter"),
                }
            }
        }
        
        DirectParser::expect_rparen(tokens)?;
        DirectParser::expect_colon(tokens)?;
        
        let return_type = match TypeName::parse(tokens)? {
            AstNode::TypeName(type_name) => type_name,
            _ => unreachable!("TypeName::parse should return TypeName"),
        };
        
        let block = match Block::parse(tokens)? {
            AstNode::Block(block) => block,
            _ => unreachable!("Block::parse should return Block"),
        };

        Ok(Function {
            id,
            parameters,
            return_type,
            statements: block.statements,
            info: (),
            position,
        }.into())
    }
}

impl From<Function<()>> for AstNode {
    fn from(value: Function<()>) -> Self {
        AstNode::Function(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FunctionParameter<T> {
    pub name: Id<T>,
    pub type_name: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromTokens<Token> for FunctionParameter<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;
        
        let name = match Id::parse(tokens)? {
            AstNode::Id(id) => id,
            _ => unreachable!("Id::parse should return Id"),
        };
        
        DirectParser::expect_colon(tokens)?;
        
        let type_name = match TypeName::parse(tokens)? {
            AstNode::TypeName(type_name) => type_name,
            _ => unreachable!("TypeName::parse should return TypeName"),
        };

        Ok(FunctionParameter {
            name,
            type_name,
            info: (),
            position,
        }.into())
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
        parser::ast::{BinaryExpression, BinaryOperator, Expression},
    };

    use super::*;

    #[test]
    fn test_simple_function() {
        let mut tokens = Lexer::new("fn foo(): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                parameters: vec![],
                return_type: TypeName::Literal("i32".into(), Span::default()),
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
        let mut tokens = Lexer::new("fn foo(x: i32): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                parameters: vec![FunctionParameter {
                    name: Id {
                        name: "x".into(),
                        info: (),
                        position: Span::default()
                    },
                    type_name: TypeName::Literal("i32".into(), Span::default()),
                    info: (),
                    position: Span::default()
                }],
                return_type: TypeName::Literal("i32".into(), Span::default()),
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
        let mut tokens = Lexer::new("fn foo(x: i32, y: i32): i32 {}")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                parameters: vec![
                    FunctionParameter {
                        name: Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into(), Span::default()),
                        info: (),
                        position: Span::default()
                    },
                    FunctionParameter {
                        name: Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into(), Span::default()),
                        info: (),
                        position: Span::default()
                    }
                ],
                return_type: TypeName::Literal("i32".into(), Span::default()),
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
        let mut tokens = Lexer::new("fn foo(x: i32, y: i32): i32 { return x + y; }")
            .lex()
            .expect("something is wrong")
            .into();

        let result = Function::parse(&mut tokens);

        assert_eq!(
            Ok(Function {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                parameters: vec![
                    FunctionParameter {
                        name: Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into(), Span::default()),
                        info: (),
                        position: Span::default()
                    },
                    FunctionParameter {
                        name: Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into(), Span::default()),
                        info: (),
                        position: Span::default()
                    }
                ],
                return_type: TypeName::Literal("i32".into(), Span::default()),
                statements: vec![Statement::Return(Expression::Binary(Box::new(
                    BinaryExpression {
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
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
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
                id: Id {
                    name: "main".into(),
                    info: (),
                    position: Span::default()
                },
                parameters: vec![
                    FunctionParameter {
                        name: Id {
                            name: "x".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into(), Span::default()),
                        info: (),
                        position: Span::default()
                    },
                    FunctionParameter {
                        name: Id {
                            name: "y".into(),
                            info: (),
                            position: Span::default()
                        },
                        type_name: TypeName::Literal("i32".into(), Span::default()),
                        info: (),
                        position: Span::default()
                    }
                ],
                return_type: TypeName::Literal("i32".into(), Span::default()),
                statements: vec![Statement::Return(Expression::Binary(Box::new(
                    BinaryExpression {
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
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
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
