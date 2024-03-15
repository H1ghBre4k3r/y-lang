mod declaration;
mod initialisation;

use crate::{
    parser::ast::Statement,
    typechecker::{context::Context, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for Statement<()> {
    type Output = Statement<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        match self {
            Statement::Function(_) => todo!(),
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
}
