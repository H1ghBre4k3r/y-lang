use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Block,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Block<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        // Enter a new scope for the block
        ctx.enter_scope();
        
        let mut last_value = None;
        
        // Execute all statements in the block except the last one
        let statements_len = self.statements.len();
        for (i, statement) in self.statements.iter().enumerate() {
            if i == statements_len - 1 {
                // For the last statement, if it's a yielding expression, get its value
                if let crate::parser::ast::Statement::YieldingExpression(expr) = statement {
                    last_value = expr.codegen(ctx);
                } else {
                    // For other types of last statements, just execute them normally
                    statement.codegen(ctx);
                }
            } else {
                // For non-last statements, just execute them
                statement.codegen(ctx);
            }
        }
        
        // Exit the scope when leaving the block
        ctx.exit_scope();
        
        last_value
    }
}