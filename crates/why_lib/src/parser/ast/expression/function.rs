use crate::{
    grammar::{self, FromGrammar},
    lexer::{Span, Token},
    parser::{
        ast::{AstNode, Statement, TypeName},
        combinators::Comb,
        FromTokens, ParseError, ParseState,
    },
};

use super::{Block, Id};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Function<T> {
    pub id: Id<T>,
    pub parameters: Vec<FunctionParameter<T>>,
    pub return_type: TypeName,
    pub statements: Vec<Statement<T>>,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::FunctionDeklaration> for Function<()> {
    fn transform(item: rust_sitter::Spanned<grammar::FunctionDeklaration>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        // Extract block statements
        let block = Block::transform(value.block, source);

        Function {
            id: Id::transform(value.ident, source),
            parameters: value
                .parameters
                .into_iter()
                .map(|param| FunctionParameter::transform(param, source))
                .collect(),
            return_type: TypeName::transform(value.type_annotation.type_name, source),
            statements: block.statements,
            info: (),
            position: Span::new(span, source),
        }
    }
}

impl FromTokens<Token> for Function<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let position = tokens.span()?;

        let matcher = Comb::FN_KEYWORD
            >> Comb::ID
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

        let Some(AstNode::Id(id)) = result.next() else {
            unreachable!()
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FunctionParameter<T> {
    pub name: Id<T>,
    pub type_name: TypeName,
    pub info: T,
    pub position: Span,
}

impl FromGrammar<grammar::FunctionParameter> for FunctionParameter<()> {
    fn transform(item: rust_sitter::Spanned<grammar::FunctionParameter>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        FunctionParameter {
            name: Id::transform(value.ident, source),
            type_name: TypeName::transform(value.type_annotation.type_name, source),
            info: (),
            position: Span::new(span, source),
        }
    }
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
