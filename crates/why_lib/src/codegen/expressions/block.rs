//! # Block Expression Code Generation
//!
//! This module implements LLVM code generation for block expressions in Y-lang.
//! Blocks create lexical scopes and can optionally yield values from their last expression.
//!
//! ## Block Semantics
//!
//! Blocks in Y-lang have expression semantics:
//! - **Scoping**: Create new lexical scope for contained statements
//! - **Value production**: Can produce a value from the last statement if it's a yielding expression
//! - **Side effects**: Execute all statements for their side effects
//! - **Scope cleanup**: Automatically clean up scope when block exits
//!
//! ## Statement Execution Order
//!
//! 1. **Enter scope**: Create new lexical scope for block variables
//! 2. **Execute statements**: Process all statements except the last one for side effects
//! 3. **Handle last statement**: Special handling for yielding expressions vs. regular statements
//! 4. **Exit scope**: Clean up block scope and restore previous scope
//!
//! ## Value Yielding
//!
//! - **YieldingExpression**: Last statement that produces a value becomes block's value
//! - **Other statements**: Block doesn't produce a value (returns `None`)
//! - **Empty blocks**: Return `None` (no value produced)

use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Block,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Block<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    /// Generates LLVM IR for block expressions.
    ///
    /// Executes all statements in the block within a new lexical scope,
    /// optionally producing a value from the last statement if it's a yielding expression.
    ///
    /// ## Implementation Strategy
    ///
    /// 1. **Scope management**: Enter new scope for block-local variables
    /// 2. **Statement processing**: Execute statements sequentially for side effects
    /// 3. **Last statement handling**: Special processing for potential value production
    /// 4. **Scope cleanup**: Ensure scope is cleaned up even on early returns
    ///
    /// ## Memory Management
    ///
    /// Variables declared within the block are automatically cleaned up when
    /// the scope is exited, preventing memory leaks and maintaining proper
    /// variable lifetime semantics.
    ///
    /// # Returns
    ///
    /// - `Some(BasicValueEnum)`: When last statement is a yielding expression
    /// - `None`: When block is used for side effects only or is empty
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        // Create isolated lexical scope for block-local variables and declarations
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
