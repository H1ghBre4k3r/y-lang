mod assignment;
mod constant;
mod declaration;
mod initialisation;
mod instance;
mod method_declaration;
mod struct_declaration;
mod while_loop;

use rust_sitter::Spanned;

pub use self::assignment::*;
pub use self::constant::*;
pub use self::declaration::*;
pub use self::initialisation::*;
pub use self::instance::*;
pub use self::method_declaration::*;
pub use self::struct_declaration::*;
pub use self::while_loop::*;

use crate::{
    grammar::{self, FromGrammar},
    lexer::{GetPosition, Span, Token},
    parser::{combinators::Comb, FromTokens, ParseError, ParseState},
};

use super::{AstNode, Expression, Function, If};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Statement<T> {
    Function(Function<T>),
    If(If<T>),
    WhileLoop(WhileLoop<T>),
    Initialization(Initialisation<T>),
    Constant(Constant<T>),
    Assignment(Assignment<T>),
    Expression(Expression<T>),
    YieldingExpression(Expression<T>),
    Return(Expression<T>),
    Comment(String),
    Declaration(Declaration<T>),
    StructDeclaration(StructDeclaration<T>),
}

impl FromGrammar<grammar::Statement> for Statement<()> {
    fn transform(item: rust_sitter::Spanned<grammar::Statement>, source: &str) -> Self {
        let rust_sitter::Spanned { value, span } = item;

        match value {
            grammar::Statement::FunctionDeclaration(function_decl) => {
                Statement::Function(Function::transform(function_decl, source))
            }
            grammar::Statement::VariableDeclaration(var_decl) => {
                Statement::Initialization(Initialisation::transform(var_decl, source))
            }
            grammar::Statement::Assignment(assignment) => {
                Statement::Assignment(Assignment::transform(assignment, source))
            }
            grammar::Statement::WhileStatement(while_stmt) => {
                Statement::WhileLoop(WhileLoop::transform(while_stmt, source))
            }
            grammar::Statement::Constant(constant) => {
                Statement::Constant(Constant::transform(constant, source))
            }
            grammar::Statement::Expression { inner, .. } => {
                Statement::Expression(Expression::transform(inner, source))
            }
            grammar::Statement::YieldingExpression(expr) => {
                Statement::YieldingExpression(Expression::transform(expr, source))
            }
            grammar::Statement::Return { inner, .. } => {
                Statement::Return(Expression::transform(inner, source))
            }
            grammar::Statement::Declaration(declaration) => {
                Statement::Declaration(Declaration::transform(declaration, source))
            }
            grammar::Statement::StructDeclaration(struct_decl) => {
                Statement::StructDeclaration(StructDeclaration::transform(struct_decl, source))
            }
            grammar::Statement::Comment(comment) => {
                // For now, use placeholder until we find the right access pattern
                Statement::Comment(comment.value.content)
            }
        }
    }
}

/// Everything that is allowed at toplevel
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TopLevelStatement<T> {
    Comment(String),
    Function(Function<T>),
    Constant(Constant<T>),
    Declaration(Declaration<T>),
    StructDeclaration(StructDeclaration<T>),
    Instance(Instance<T>),
}

impl FromGrammar<grammar::ToplevelStatement> for TopLevelStatement<()> {
    fn transform(item: rust_sitter::Spanned<grammar::ToplevelStatement>, source: &str) -> Self {
        let Spanned { value, span } = item;

        match value {
            grammar::ToplevelStatement::FunctionDeclaration(function) => {
                TopLevelStatement::Function(Function::transform(function, source))
            }
            grammar::ToplevelStatement::Constant(constant) => {
                TopLevelStatement::Constant(Constant::transform(constant, source))
            }
            grammar::ToplevelStatement::Declaration(declaration) => {
                TopLevelStatement::Declaration(Declaration::transform(declaration, source))
            }
            grammar::ToplevelStatement::StructDeclaration(struct_declaration) => {
                TopLevelStatement::StructDeclaration(StructDeclaration::transform(
                    struct_declaration,
                    source,
                ))
            }
            grammar::ToplevelStatement::Instance(instance) => {
                TopLevelStatement::Instance(Instance::transform(instance, source))
            }
            grammar::ToplevelStatement::Comment(spanned) => {
                TopLevelStatement::Comment(spanned.value.content)
            }
        }
    }
}

impl TopLevelStatement<()> {
    pub fn parse(tokens: &mut ParseState<Token>) -> Result<TopLevelStatement<()>, ParseError> {
        let Some(next) = tokens.peek() else {
            return Err(ParseError {
                message: "Unexpected EOF!".into(),
                position: tokens.last_token().map(|token| token.position()),
            });
        };

        match next {
            Token::FnKeyword { .. } => {
                let matcher = Comb::FUNCTION;
                let result = matcher.parse(tokens)?;

                let [AstNode::Function(function)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(TopLevelStatement::Function(function.clone()))
            }
            Token::Const { .. } => {
                let matcher = Comb::CONSTANT >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::Constant(constant)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(TopLevelStatement::Constant(constant.clone()))
            }
            Token::DeclareKeyword { .. } => {
                let matcher = Comb::DECLARATION >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let Some(AstNode::Declaration(declaration)) = result.first().cloned() else {
                    unreachable!()
                };
                Ok(TopLevelStatement::Declaration(declaration))
            }
            Token::Comment { value, .. } => {
                tokens.next();
                Ok(TopLevelStatement::Comment(value))
            }
            Token::StructKeyword { .. } => {
                let matcher = Comb::STRUCT_DECLARATION;
                let result = matcher.parse(tokens).inspect_err(|e| {
                    tokens.add_error(e.clone());
                })?;

                let Some(AstNode::StructDeclaration(declaration)) = result.first().cloned() else {
                    unreachable!()
                };
                Ok(TopLevelStatement::StructDeclaration(declaration))
            }
            Token::InstanceKeyword { .. } => {
                let matcher = Comb::INSTANCE;
                let result = matcher.parse(tokens).inspect_err(|e| {
                    tokens.add_error(e.clone());
                })?;

                let Some(AstNode::Instance(instance)) = result.first().cloned() else {
                    unreachable!()
                };
                Ok(TopLevelStatement::Instance(instance))
            }
            token => Err(ParseError {
                message: format!("unexpected {token:?} at toplevel!"),
                position: Some(token.position()),
            }),
        }
    }
}

impl FromTokens<Token> for Statement<()> {
    fn parse(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError>
    where
        Self: Sized,
    {
        let Some(next) = tokens.peek() else {
            return Err(ParseError {
                message: "Unexpected EOF!".into(),
                position: tokens.last_token().map(|token| token.position()),
            });
        };

        match next {
            Token::IfKeyword { .. } => {
                let matcher = Comb::IF;
                let result = matcher.parse(tokens)?;

                let [AstNode::If(if_statement)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::If(if_statement.clone()).into())
            }
            Token::FnKeyword { .. } => {
                let matcher = Comb::FUNCTION;
                let result = matcher.parse(tokens)?;

                let [AstNode::Function(function)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Function(function.clone()).into())
            }
            Token::WhileKeyword { .. } => {
                let matcher = Comb::WHILE_LOOP;
                let result = matcher.parse(tokens).inspect_err(|e| {
                    tokens.add_error(e.clone());
                })?;

                let [AstNode::WhileLoop(while_loop_statement)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::WhileLoop(while_loop_statement.clone()).into())
            }
            Token::Let { .. } => {
                let matcher = Comb::INITIALISATION >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::Initialization(init)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Initialization(init.clone()).into())
            }
            Token::Const { .. } => {
                let matcher = Comb::CONSTANT >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::Constant(constant)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Constant(constant.clone()).into())
            }
            Token::ReturnKeyword { .. } => {
                let matcher = Comb::RETURN_KEYWORD >> Comb::EXPR >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let [AstNode::Expression(expr)] = result.as_slice() else {
                    unreachable!()
                };
                Ok(Statement::Return(expr.clone()).into())
            }
            Token::DeclareKeyword { .. } => {
                let matcher = Comb::DECLARATION >> Comb::SEMI;
                let result = matcher.parse(tokens)?;

                let Some(AstNode::Declaration(declaration)) = result.first().cloned() else {
                    unreachable!()
                };
                Ok(Statement::Declaration(declaration).into())
            }
            Token::Comment { value, .. } => {
                tokens.next();
                Ok(Statement::Comment(value).into())
            }
            Token::StructKeyword { .. } => {
                let matcher = Comb::STRUCT_DECLARATION;
                let result = matcher.parse(tokens).inspect_err(|e| {
                    tokens.add_error(e.clone());
                })?;

                let Some(AstNode::StructDeclaration(declaration)) = result.first().cloned() else {
                    unreachable!()
                };
                Ok(Statement::StructDeclaration(declaration).into())
            }
            token => {
                if let Ok(assignment) = Self::parse_assignment(tokens) {
                    return Ok(assignment);
                };

                if let Ok(expr) = Self::parse_expression(tokens) {
                    return Ok(expr);
                };

                Err(ParseError {
                    message: format!("unexpected {token:?} while trying to parse statement",),
                    position: Some(token.position()),
                })
            }
        }
    }
}

impl Statement<()> {
    fn parse_assignment(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let matcher = Comb::ASSIGNMENT;
        let result = matcher.parse(tokens).inspect_err(|_| {
            tokens.set_index(index);
        })?;

        let [AstNode::Assignment(assignment)] = result.as_slice() else {
            unreachable!()
        };

        let index = tokens.get_index();
        let matcher = Comb::SEMI;

        matcher.parse(tokens).inspect_err(|e| {
            tokens.set_index(index);
            tokens.add_error(e.clone());
        })?;

        Ok(Statement::Assignment(assignment.clone()).into())
    }

    fn parse_expression(tokens: &mut ParseState<Token>) -> Result<AstNode, ParseError> {
        let index = tokens.get_index();

        let matcher = Comb::EXPR;
        let result = matcher.parse(tokens).inspect_err(|_| {
            tokens.set_index(index);
        })?;

        let [AstNode::Expression(expr)] = result.as_slice() else {
            unreachable!()
        };
        match tokens.peek() {
            Some(Token::Semicolon { .. }) => {
                tokens.next();
                Ok(Statement::Expression(expr.clone()).into())
            }
            _ => Ok(Statement::YieldingExpression(expr.clone()).into()),
        }
    }
}

impl From<Statement<()>> for AstNode {
    fn from(value: Statement<()>) -> Self {
        AstNode::Statement(value)
    }
}

impl<T> Statement<T>
where
    T: Clone,
{
    pub fn get_info(&self) -> T {
        match self {
            Statement::Function(Function { info, .. }) => info.clone(),
            Statement::If(If { info, .. }) => info.clone(),
            Statement::WhileLoop(WhileLoop { info, .. }) => info.clone(),
            Statement::Initialization(Initialisation { info, .. }) => info.clone(),
            Statement::Constant(Constant { info, .. }) => info.clone(),
            Statement::Assignment(Assignment { info, .. }) => info.clone(),
            Statement::Expression(exp) => exp.get_info(),
            Statement::YieldingExpression(exp) => exp.get_info(),
            Statement::Return(exp) => exp.get_info(),
            Statement::Comment(_) => unimplemented!("Comments to not have type information"),
            Statement::Declaration(Declaration { info, .. }) => info.clone(),
            Statement::StructDeclaration(StructDeclaration { info, .. }) => info.clone(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Statement::Function(Function { position, .. }) => position.clone(),
            Statement::If(If { position, .. }) => position.clone(),
            Statement::WhileLoop(WhileLoop { position, .. }) => position.clone(),
            Statement::Initialization(Initialisation { position, .. }) => position.clone(),
            Statement::Constant(Constant { position, .. }) => position.clone(),
            Statement::Assignment(Assignment { position, .. }) => position.clone(),
            Statement::Expression(exp) => exp.position(),
            Statement::YieldingExpression(exp) => exp.position(),
            Statement::Return(exp) => exp.position(),
            Statement::Comment(_) => todo!(),
            Statement::Declaration(Declaration { position, .. }) => position.clone(),
            Statement::StructDeclaration(StructDeclaration { position, .. }) => position.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, Span},
        parser::ast::{BinaryExpression, BinaryOperator, Id, Num, TypeName},
    };

    use super::*;

    #[test]
    fn test_basic_constant() {
        let mut tokens = Lexer::new("const foo: i32 = 42;")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::Constant(Constant {
                id: Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default()
                },
                type_name: TypeName::Literal("i32".into(), Span::default()),
                value: Expression::Num(Num::Integer(42, (), Span::default())),
                info: (),
                position: Span::default()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_basic_return() {
        let mut tokens = Lexer::new("return 42;").lex().expect("should work").into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::Return(Expression::Num(Num::Integer(42, (), Span::default()))).into()),
            result
        );
    }

    #[test]
    fn test_if_else_without_semicolon() {
        let mut tokens = Lexer::new("if (x) { 3 + 4 } else { 42 + 1337 }")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::If(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                })),
                statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression {
                        left: Expression::Num(Num::Integer(3, (), Span::default())),
                        right: Expression::Num(Num::Integer(4, (), Span::default())),
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
                    }
                )))],
                else_statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression {
                        left: Expression::Num(Num::Integer(42, (), Span::default())),
                        right: Expression::Num(Num::Integer(1337, (), Span::default())),
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
                    }
                )))],
                info: (),
                position: Span::default()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_if_else_with_semicolon() {
        let mut tokens = Lexer::new("if (x) { 3 + 4 } else { 42 + 1337 };")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::If(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                })),
                statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression {
                        left: Expression::Num(Num::Integer(3, (), Span::default())),
                        right: Expression::Num(Num::Integer(4, (), Span::default())),
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
                    }
                )))],
                else_statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression {
                        left: Expression::Num(Num::Integer(42, (), Span::default())),
                        right: Expression::Num(Num::Integer(1337, (), Span::default())),
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
                    }
                )))],
                info: (),
                position: Span::default()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_if_else_ignores_call() {
        let mut tokens = Lexer::new("if (x) { 3 + 4 } else { 42 + 1337 }()")
            .lex()
            .expect("should work")
            .into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::If(If {
                condition: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                })),
                statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression {
                        left: Expression::Num(Num::Integer(3, (), Span::default())),
                        right: Expression::Num(Num::Integer(4, (), Span::default())),
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
                    }
                )))],
                else_statements: vec![Statement::YieldingExpression(Expression::Binary(Box::new(
                    BinaryExpression {
                        left: Expression::Num(Num::Integer(42, (), Span::default())),
                        right: Expression::Num(Num::Integer(1337, (), Span::default())),
                        operator: BinaryOperator::Add,
                        info: (),
                        position: Span::default()
                    }
                )))],
                info: (),
                position: Span::default()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_simple_assignment() {
        let mut tokens = Lexer::new("x = 42;").lex().expect("should work").into();

        let result = Statement::parse(&mut tokens);

        assert_eq!(
            Ok(Statement::Assignment(Assignment {
                lvalue: LValue::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default()
                }),
                rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
                info: (),
                position: Span::default()
            })
            .into()),
            result
        )
    }

    #[test]
    fn test_assignment_needs_semicolon() {
        let mut tokens = Lexer::new("x = 42").lex().expect("should work").into();

        let result = Statement::parse_assignment(&mut tokens);

        assert!(result.is_err())
    }
}
