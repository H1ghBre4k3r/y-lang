mod function;
mod initialisation;

use crate::{
    parser::ast::{Statement, TopLevelStatement},
    typechecker::ValidatedTypeInformation,
};

use super::{CodeGen, CodegenContext};

impl<'ctx> CodeGen<'ctx> for Statement<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) {
        match self {
            Statement::Function(function) => function.codegen(ctx),
            Statement::If(_) => todo!(),
            Statement::WhileLoop(while_loop) => todo!(),
            Statement::Initialization(initialisation) => initialisation.codegen(ctx),
            Statement::Constant(constant) => todo!(),
            Statement::Assignment(assignment) => todo!(),
            Statement::Expression(expression) => todo!(),
            Statement::YieldingExpression(expression) => todo!(),
            Statement::Return(expression) => {
                let llvm_return_value = expression.codegen(ctx);

                if let Err(e) = ctx.builder.build_return(Some(&llvm_return_value)) {
                    panic!("{e}");
                }
            }
            Statement::Comment(_) => todo!(),
            Statement::Declaration(declaration) => todo!(),
            Statement::StructDeclaration(struct_declaration) => todo!(),
        }
    }
}

impl<'ctx> CodeGen<'ctx> for TopLevelStatement<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) {
        match self {
            TopLevelStatement::Comment(_) => todo!(),
            TopLevelStatement::Function(function) => function.codegen(ctx),
            TopLevelStatement::Constant(constant) => todo!(),
            TopLevelStatement::Declaration(declaration) => todo!(),
            TopLevelStatement::StructDeclaration(struct_declaration) => todo!(),
            TopLevelStatement::Instance(instance) => todo!(),
        }
    }
}
