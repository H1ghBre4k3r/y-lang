use crate::{codegen::CodeGen, parser::ast::WhileLoop, typechecker::ValidatedTypeInformation};

impl<'ctx> CodeGen<'ctx> for WhileLoop<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
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
        let Some(condition_value) = self.condition.codegen(ctx) else {
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
        for statement in &self.block.statements {
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
}
