pub mod declaration;
pub mod function;
pub mod initialisation;

use crate::{
    parser::ast::{LValue, Statement, TopLevelStatement},
    typechecker::{Type, ValidatedTypeInformation},
};

use super::{CodeGen, CodegenContext};

impl<'ctx> CodeGen<'ctx> for Statement<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) {
        match self {
            Statement::Function(function) => function.codegen(ctx),
            Statement::WhileLoop(while_loop) => {
                // Create basic blocks for the while loop
                let current_function = ctx
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let condition_block = ctx
                    .context
                    .append_basic_block(current_function, "while.condition");
                let loop_body_block = ctx
                    .context
                    .append_basic_block(current_function, "while.body");
                let after_loop_block = ctx
                    .context
                    .append_basic_block(current_function, "while.end");

                // Jump to condition block
                ctx.builder
                    .build_unconditional_branch(condition_block)
                    .unwrap();

                // Build condition block
                ctx.builder.position_at_end(condition_block);
                let Some(condition_value) = while_loop.condition.codegen(ctx) else {
                    unreachable!("While loop condition must produce a value")
                };
                let condition_value = condition_value.into_int_value(); // Boolean is represented as i1

                // Branch based on condition
                ctx.builder
                    .build_conditional_branch(condition_value, loop_body_block, after_loop_block)
                    .unwrap();

                // Build loop body block
                ctx.builder.position_at_end(loop_body_block);
                ctx.enter_scope();
                for statement in &while_loop.block.statements {
                    statement.codegen(ctx);
                }
                ctx.exit_scope();

                // Jump back to condition (if we haven't returned)
                if ctx
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    ctx.builder
                        .build_unconditional_branch(condition_block)
                        .unwrap();
                }

                // Position builder at after_loop_block for subsequent code
                ctx.builder.position_at_end(after_loop_block);
            }
            Statement::Initialization(initialisation) => initialisation.codegen(ctx),
            Statement::Constant(constant) => todo!(),
            Statement::Assignment(assignment) => {
                let Some(rvalue) = assignment.rvalue.codegen(ctx) else {
                    unreachable!("Assignment rvalue must produce a value")
                };

                match &assignment.lvalue {
                    LValue::Id(id) => {
                        // Simple variable assignment - store to existing variable
                        let variable_ptr = ctx.find_variable(&id.name);
                        ctx.builder
                            .build_store(variable_ptr.into_pointer_value(), rvalue)
                            .unwrap();
                    }
                    LValue::Postfix(_postfix) => {
                        // TODO: Handle array indexing and property access assignments
                        todo!("Complex lvalue assignment not yet implemented")
                    }
                }
            }
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
