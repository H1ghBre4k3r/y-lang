mod declaration;
mod initialisation;

use crate::{
    parser::ast::Statement,
    typechecker::{
        context::Context, error::TypeCheckError, types::Type, TypeCheckable, TypeInformation,
        TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Statement<()> {
    type Output = Statement<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        match self {
            Statement::Function(func) => Ok(Statement::Function(func.check(ctx)?)),
            Statement::If(_) => todo!(),
            Statement::WhileLoop(_) => todo!(),
            Statement::Initialization(init) => Ok(Statement::Initialization(init.check(ctx)?)),
            Statement::Constant(_) => todo!(),
            Statement::Assignment(_) => todo!(),
            Statement::Expression(exp) => Ok(Statement::Expression(exp.check(ctx)?)),
            Statement::YieldingExpression(exp) => {
                Ok(Statement::YieldingExpression(exp.check(ctx)?))
            }
            Statement::Return(exp) => Ok(Statement::Return(exp.check(ctx)?)),
            Statement::Comment(c) => Ok(Statement::Comment(c)),
            Statement::Declaration(dec) => Ok(Statement::Declaration(dec.check(ctx)?)),
            Statement::StructDeclaration(_) => todo!(),
        }
    }

    fn revert(this: &Self::Output) -> Self {
        match this {
            Statement::Function(func) => Statement::Function(TypeCheckable::revert(func)),
            Statement::If(_) => todo!(),
            Statement::WhileLoop(_) => todo!(),
            Statement::Initialization(init) => {
                Statement::Initialization(TypeCheckable::revert(init))
            }
            Statement::Constant(_) => todo!(),
            Statement::Assignment(_) => todo!(),
            Statement::Expression(expr) => Statement::Expression(TypeCheckable::revert(expr)),
            Statement::YieldingExpression(expr) => {
                Statement::YieldingExpression(TypeCheckable::revert(expr))
            }
            Statement::Return(expr) => Statement::Return(TypeCheckable::revert(expr)),
            Statement::Comment(c) => Statement::Comment(c.to_owned()),
            Statement::Declaration(dec) => Statement::Declaration(TypeCheckable::revert(dec)),
            Statement::StructDeclaration(_) => todo!(),
        }
    }
}

impl TypedConstruct for Statement<TypeInformation> {
    fn update_type(&mut self, type_id: Type) -> std::result::Result<(), TypeCheckError> {
        match self {
            Statement::Function(_) => todo!(),
            Statement::If(_) => todo!(),
            Statement::WhileLoop(_) => todo!(),
            Statement::Initialization(init) => init.update_type(type_id),
            Statement::Constant(_) => todo!(),
            Statement::Assignment(_) => todo!(),
            Statement::Expression(expr) => expr.update_type(type_id),
            Statement::YieldingExpression(expr) => expr.update_type(type_id),
            Statement::Return(expr) => expr.update_type(type_id),
            Statement::Comment(_) => Ok(()),
            Statement::Declaration(dec) => dec.update_type(type_id),
            Statement::StructDeclaration(_) => todo!(),
        }
    }
}
