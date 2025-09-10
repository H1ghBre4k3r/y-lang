use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::If,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for If<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        // Generate condition
        let condition_value = self.condition.codegen(ctx)?;

        // Get the current function to create basic blocks
        let current_function = ctx.builder.get_insert_block()?.get_parent()?;

        // Create basic blocks for then, else, and merge
        let then_block = ctx.context.append_basic_block(current_function, "if_then");
        let else_block = ctx.context.append_basic_block(current_function, "if_else");
        let merge_block = ctx.context.append_basic_block(current_function, "if_merge");

        // Build conditional branch
        ctx.builder
            .build_conditional_branch(condition_value.into_int_value(), then_block, else_block)
            .ok()?;

        // Generate then block
        ctx.builder.position_at_end(then_block);
        let mut then_value = None;

        // Enter new scope for then block
        ctx.enter_scope();

        for (i, statement) in self.statements.iter().enumerate() {
            if i == self.statements.len() - 1 {
                // For the last statement, if it's a yielding expression, get its value
                if let crate::parser::ast::Statement::YieldingExpression(expr) = statement {
                    then_value = expr.codegen(ctx);
                } else {
                    statement.codegen(ctx);
                }
            } else {
                statement.codegen(ctx);
            }
        }

        ctx.exit_scope();

        // Branch to merge block (if not already terminated)
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder.build_unconditional_branch(merge_block).ok()?;
        }

        // Generate else block
        ctx.builder.position_at_end(else_block);
        let mut else_value = None;

        // Enter new scope for else block
        ctx.enter_scope();

        for (i, statement) in self.else_statements.iter().enumerate() {
            if i == self.else_statements.len() - 1 {
                // For the last statement, if it's a yielding expression, get its value
                if let crate::parser::ast::Statement::YieldingExpression(expr) = statement {
                    else_value = expr.codegen(ctx);
                } else {
                    statement.codegen(ctx);
                }
            } else {
                statement.codegen(ctx);
            }
        }

        ctx.exit_scope();

        // Branch to merge block (if not already terminated)
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder.build_unconditional_branch(merge_block).ok()?;
        }

        // Position at merge block
        ctx.builder.position_at_end(merge_block);

        // If both branches produce values, create a phi node
        match (then_value, else_value) {
            (Some(then_val), Some(else_val)) => {
                let phi = ctx
                    .builder
                    .build_phi(then_val.get_type(), "if_result")
                    .ok()?;
                phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);
                Some(phi.as_basic_value())
            }
            (Some(then_val), None) if self.else_statements.is_empty() => {
                // If-without-else that produces a value - not typical, but handle it
                Some(then_val)
            }
            _ => {
                // No values produced or inconsistent values
                None
            }
        }
    }
}
