pub mod assignment;
pub mod constant;
pub mod declaration;
pub mod function;
pub mod initialisation;
pub mod instance;
pub mod struct_declaration;
pub mod while_loop;

use crate::{
    parser::ast::{Statement, TopLevelStatement},
    typechecker::{Type, ValidatedTypeInformation},
};

use super::{CodeGen, CodegenContext};

impl<'ctx> CodeGen<'ctx> for Statement<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) {
        match self {
            Statement::Function(function) => function.codegen(ctx),
            Statement::WhileLoop(while_loop) => while_loop.codegen(ctx),
            Statement::Initialization(initialisation) => initialisation.codegen(ctx),
            Statement::Constant(constant) => constant.codegen(ctx),
            Statement::Assignment(assignment) => assignment.codegen(ctx),
            Statement::Expression(expression) => {
                expression.codegen(ctx);
            }
            Statement::YieldingExpression(expression) => {
                let llvm_return_value = expression.codegen(ctx);

                if expression.get_info().type_id == Type::Void {
                    if let Err(e) = ctx.builder.build_return(None) {
                        panic!("{e}");
                    }
                } else {
                    let Some(llvm_return_value) = llvm_return_value else {
                        unreachable!("YieldingExpression should always produce a value")
                    };
                    if let Err(e) = ctx.builder.build_return(Some(&llvm_return_value)) {
                        panic!("{e}");
                    }
                }
            }
            Statement::Return(expression) => {
                let Some(llvm_return_value) = expression.codegen(ctx) else {
                    unreachable!()
                };

                if let Err(e) = ctx.builder.build_return(Some(&llvm_return_value)) {
                    panic!("{e}");
                }
            }
            Statement::Comment(_) => {} // Comments are no-ops in codegen
            Statement::Declaration(declaration) => declaration.codegen(ctx),
            Statement::StructDeclaration(struct_declaration) => struct_declaration.codegen(ctx),
        }
    }
}

impl<'ctx> CodeGen<'ctx> for TopLevelStatement<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) {
        match self {
            TopLevelStatement::Comment(_) => {} // Comments are no-ops in codegen
            TopLevelStatement::Function(function) => function.codegen(ctx),
            TopLevelStatement::Constant(constant) => constant.codegen(ctx),
            TopLevelStatement::Declaration(declaration) => declaration.codegen(ctx),
            TopLevelStatement::StructDeclaration(struct_declaration) => {
                struct_declaration.codegen(ctx)
            }
            TopLevelStatement::Instance(instance) => instance.codegen(ctx),
        }
    }
}

impl TopLevelStatement<ValidatedTypeInformation> {
    /// First pass: Register function declarations without generating bodies
    /// This allows forward references to functions defined later in the file
    pub fn register_function_declaration<'ctx>(&self, ctx: &CodegenContext<'ctx>) {
        match self {
            TopLevelStatement::Function(function) => {
                function.register_declaration(ctx);
            }
            TopLevelStatement::Instance(instance) => {
                instance.register_declarations(ctx);
            }
            // Other statements don't need declaration registration
            _ => {}
        }
    }
}
