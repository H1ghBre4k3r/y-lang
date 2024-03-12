use crate::{
    parser::ast::Statement,
    typechecker::{context::Context, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for Statement<()> {
    type Output = Statement<TypeInformation>;

    fn check(self, scope: &mut Context) -> TypeResult<Self::Output> {
        match self {
            Statement::Function(_) => todo!(),
            Statement::If(_) => todo!(),
            Statement::WhileLoop(_) => todo!(),
            Statement::Initialization(_) => todo!(),
            Statement::Constant(_) => todo!(),
            Statement::Assignment(_) => todo!(),
            Statement::Expression(_) => todo!(),
            Statement::YieldingExpression(_) => todo!(),
            Statement::Return(_) => todo!(),
            Statement::Comment(_) => todo!(),
            Statement::Declaration(_) => todo!(),
            Statement::StructDeclaration(_) => todo!(),
        }
    }
}
