mod assignment;
mod constant;
mod declaration;
mod initialisation;
mod instance;
mod struct_declaration;
mod while_loop;

use crate::{
    parser::ast::{Statement, TopLevelStatement},
    typechecker::{
        context::Context, error::TypeCheckError, types::Type, ShallowCheck, TypeCheckable,
        TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for TopLevelStatement<()> {
    type Output = TopLevelStatement<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        match self {
            TopLevelStatement::Function(func) => Ok(TopLevelStatement::Function(func.check(ctx)?)),
            TopLevelStatement::Constant(constant) => {
                Ok(TopLevelStatement::Constant(constant.check(ctx)?))
            }
            TopLevelStatement::Comment(c) => Ok(TopLevelStatement::Comment(c)),
            TopLevelStatement::Declaration(dec) => {
                Ok(TopLevelStatement::Declaration(dec.check(ctx)?))
            }
            TopLevelStatement::StructDeclaration(dec) => {
                Ok(TopLevelStatement::StructDeclaration(dec.check(ctx)?))
            }
            TopLevelStatement::Instance(inst) => Ok(TopLevelStatement::Instance(inst.check(ctx)?)),
        }
    }

    fn revert(this: &Self::Output) -> Self {
        match this {
            TopLevelStatement::Function(func) => {
                TopLevelStatement::Function(TypeCheckable::revert(func))
            }
            TopLevelStatement::Constant(_) => {
                unimplemented!("TypeCheckable::revert is not implemented for Constants")
            }
            TopLevelStatement::Comment(c) => TopLevelStatement::Comment(c.to_owned()),
            TopLevelStatement::Declaration(dec) => {
                TopLevelStatement::Declaration(TypeCheckable::revert(dec))
            }
            TopLevelStatement::StructDeclaration(dec) => {
                TopLevelStatement::StructDeclaration(TypeCheckable::revert(dec))
            }
            TopLevelStatement::Instance(inst) => {
                TopLevelStatement::Instance(TypeCheckable::revert(inst))
            }
        }
    }
}

impl ShallowCheck for TopLevelStatement<()> {
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        match self {
            TopLevelStatement::Comment(_) => Ok(()),
            TopLevelStatement::Function(inner) => inner.shallow_check(ctx),
            TopLevelStatement::Constant(inner) => inner.shallow_check(ctx),
            TopLevelStatement::Declaration(inner) => inner.shallow_check(ctx),
            TopLevelStatement::StructDeclaration(inner) => inner.shallow_check(ctx),
            TopLevelStatement::Instance(inner) => inner.shallow_check(ctx),
        }
    }
}

impl TypeCheckable for Statement<()> {
    type Output = Statement<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        match self {
            Statement::Function(func) => Ok(Statement::Function(func.check(ctx)?)),
            Statement::If(if_exp) => Ok(Statement::If(if_exp.check(ctx)?)),
            Statement::WhileLoop(while_l) => Ok(Statement::WhileLoop(while_l.check(ctx)?)),
            Statement::Initialization(init) => Ok(Statement::Initialization(init.check(ctx)?)),
            Statement::Constant(constant) => Ok(Statement::Constant(constant.check(ctx)?)),
            Statement::Assignment(assign) => Ok(Statement::Assignment(assign.check(ctx)?)),
            Statement::Expression(exp) => Ok(Statement::Expression(exp.check(ctx)?)),
            Statement::YieldingExpression(exp) => {
                Ok(Statement::YieldingExpression(exp.check(ctx)?))
            }
            Statement::Return(exp) => Ok(Statement::Return(exp.check(ctx)?)),
            Statement::Comment(c) => Ok(Statement::Comment(c)),
            Statement::Declaration(dec) => Ok(Statement::Declaration(dec.check(ctx)?)),
            Statement::StructDeclaration(dec) => Ok(Statement::StructDeclaration(dec.check(ctx)?)),
        }
    }

    fn revert(this: &Self::Output) -> Self {
        match this {
            Statement::Function(func) => Statement::Function(TypeCheckable::revert(func)),
            Statement::If(if_exp) => Statement::If(TypeCheckable::revert(if_exp)),
            Statement::WhileLoop(while_l) => Statement::WhileLoop(TypeCheckable::revert(while_l)),
            Statement::Initialization(init) => {
                Statement::Initialization(TypeCheckable::revert(init))
            }
            Statement::Constant(_) => todo!(),
            Statement::Assignment(assign) => Statement::Assignment(TypeCheckable::revert(assign)),
            Statement::Expression(expr) => Statement::Expression(TypeCheckable::revert(expr)),
            Statement::YieldingExpression(expr) => {
                Statement::YieldingExpression(TypeCheckable::revert(expr))
            }
            Statement::Return(expr) => Statement::Return(TypeCheckable::revert(expr)),
            Statement::Comment(c) => Statement::Comment(c.to_owned()),
            Statement::Declaration(dec) => Statement::Declaration(TypeCheckable::revert(dec)),
            Statement::StructDeclaration(dec) => {
                Statement::StructDeclaration(TypeCheckable::revert(dec))
            }
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
            Statement::Constant(constant) => constant.update_type(type_id),
            Statement::Assignment(_) => todo!(),
            Statement::Expression(expr) => expr.update_type(type_id),
            Statement::YieldingExpression(expr) => expr.update_type(type_id),
            Statement::Return(expr) => expr.update_type(type_id),
            Statement::Comment(_) => Ok(()),
            Statement::Declaration(dec) => dec.update_type(type_id),
            Statement::StructDeclaration(dec) => dec.update_type(type_id),
        }
    }
}
