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
    lexer::Span,
};

use super::{AstNode, Expression, Function};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Statement<T> {
    Function(Function<T>),
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
        let rust_sitter::Spanned { value, span: _ } = item;

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
        let Spanned { value, span: _ } = item;

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
            Statement::WhileLoop(WhileLoop { info, .. }) => info.clone(),
            Statement::Initialization(Initialisation { info, .. }) => info.clone(),
            Statement::Constant(Constant { info, .. }) => info.clone(),
            Statement::Assignment(Assignment { info, .. }) => info.clone(),
            Statement::Expression(expression) => expression.get_info(),
            Statement::YieldingExpression(expression) => expression.get_info(),
            Statement::Return(expression) => expression.get_info(),
            Statement::Comment(_) => unreachable!("comments don't have info"),
            Statement::Declaration(Declaration { info, .. }) => info.clone(),
            Statement::StructDeclaration(StructDeclaration { info, .. }) => info.clone(),
        }
    }

    pub fn position(&self) -> Span {
        match self {
            Statement::Function(Function { position, .. }) => position.clone(),
            Statement::WhileLoop(WhileLoop { position, .. }) => position.clone(),
            Statement::Initialization(Initialisation { position, .. }) => position.clone(),
            Statement::Constant(Constant { position, .. }) => position.clone(),
            Statement::Assignment(Assignment { position, .. }) => position.clone(),
            Statement::Expression(expression) => expression.position(),
            Statement::YieldingExpression(expression) => expression.position(),
            Statement::Return(expression) => expression.position(),
            Statement::Comment(_) => Span::default(),
            Statement::Declaration(Declaration { position, .. }) => position.clone(),
            Statement::StructDeclaration(StructDeclaration { position, .. }) => position.clone(),
        }
    }
}
