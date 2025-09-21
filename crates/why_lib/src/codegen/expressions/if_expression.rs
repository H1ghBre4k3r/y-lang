use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::If,
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for If<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let If {
            condition,
            then_block,
            else_block,
            info,
            ..
        } = self;

        // Generate the condition expression first - this must be evaluated before branching
        let condition_llvm_value = condition.codegen(ctx)?;

        let function_llvm_value = ctx.builder.get_insert_block()?.get_parent()?;

        // Create three basic blocks for the if expression control flow:
        // - then_label: executes when condition is true
        // - else_label: executes when condition is false
        // - merge_label: where both branches converge after execution
        let then_label = ctx
            .context
            .append_basic_block(function_llvm_value, "if.then");
        let else_label = ctx
            .context
            .append_basic_block(function_llvm_value, "if.else");
        let merge_label = ctx
            .context
            .append_basic_block(function_llvm_value, "if.merge");

        // Generate the conditional branch instruction that directs control flow
        // based on the boolean condition value
        ctx.builder
            .build_conditional_branch(
                condition_llvm_value.into_int_value(),
                then_label,
                else_label,
            )
            .ok()?;

        ctx.builder.position_at_end(then_label);

        // Generate code for the then branch within its own scope to isolate variables
        ctx.enter_scope();
        let then_value = then_block.codegen(ctx);
        ctx.exit_scope();

        // Only add a branch to merge if the then block doesn't already have a terminator
        // (e.g., a return statement). This prevents invalid IR with multiple terminators.
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder.build_unconditional_branch(merge_label).ok()?;
        }

        ctx.builder.position_at_end(else_label);

        // Generate code for the else branch within its own scope to isolate variables
        ctx.enter_scope();
        let else_value = else_block.codegen(ctx);
        ctx.exit_scope();

        // Only add a branch to merge if the else block doesn't already have a terminator
        // (e.g., a return statement). This prevents invalid IR with multiple terminators.

        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder.build_unconditional_branch(merge_label).ok()?;
        }

        ctx.builder.position_at_end(merge_label);

        // If the if expression yields a value (non-void), we need to merge the values
        // from both branches using a phi node
        let yields_value = info.type_id != Type::Void;
        match (then_value, else_value) {
            (Some(then_value), Some(else_value)) if yields_value => {
                // Create a phi node to merge values from different control flow paths.
                // This is necessary because at the merge point, we don't know statically
                // which branch was taken - the phi node selects the appropriate value
                // based on which predecessor block we came from.
                let phi = ctx
                    .builder
                    .build_phi(then_value.get_type(), "if_result")
                    .ok()?;

                // Register the incoming values with their source basic blocks
                phi.add_incoming(&[(&then_value, then_label), (&else_value, else_label)]);
                Some(phi.as_basic_value())
            }
            other if yields_value => panic!("Unexpected {other:?} for yielding if expression"),
            // For void if expressions, we don't need to return a value
            _ => None,
        }
    }
}
