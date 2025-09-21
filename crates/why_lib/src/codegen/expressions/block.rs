use inkwell::values::BasicValueEnum;

use crate::{codegen::CodeGen, parser::ast::Block, typechecker::ValidatedTypeInformation};

impl<'ctx> CodeGen<'ctx> for Block<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        ctx.enter_scope();

        let mut last_value = None;
        // Process all statements in the block with special handling for the last one
        let statements_len = self.statements.len();
        for (i, statement) in self.statements.iter().enumerate() {
            if i == statements_len - 1 {
                // Last statement: check if it's a yielding expression for value production
                if let crate::parser::ast::Statement::YieldingExpression(expr) = statement {
                    // Yielding expression becomes the block's value
                    last_value = expr.codegen(ctx);
                } else {
                    // Regular statement: execute for side effects only
                    statement.codegen(ctx);
                }
            } else {
                // Non-last statements: execute purely for side effects
                statement.codegen(ctx);
            }
        }

        // Clean up block scope - this discards all block-local variables
        // Must happen after all processing to ensure proper variable lifetimes
        ctx.exit_scope();

        // Return the value produced by the last statement (if any)
        last_value
    }
}
