pub mod declaration;
pub mod function;
pub mod initialisation;

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
            Statement::WhileLoop(while_loop) => todo!(),
            Statement::Initialization(initialisation) => initialisation.codegen(ctx),
            Statement::Constant(constant) => todo!(),
            Statement::Assignment(assignment) => todo!(),
            Statement::Expression(expression) => {
                expression.codegen(ctx);
            }
            Statement::YieldingExpression(expression) => todo!(),
            Statement::Return(expression) => {
                let Some(llvm_return_value) = expression.codegen(ctx) else {
                    unreachable!()
                };

                if let Err(e) = ctx.builder.build_return(Some(&llvm_return_value)) {
                    panic!("{e}");
                }
            }
            Statement::Comment(_) => todo!(),
            Statement::Declaration(declaration) => declaration.codegen(ctx),
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
            TopLevelStatement::Declaration(declaration) => declaration.codegen(ctx),
            TopLevelStatement::StructDeclaration(struct_declaration) => todo!(),
            TopLevelStatement::Instance(instance) => todo!(),
        }
    }
}
