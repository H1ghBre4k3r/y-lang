//! # If Expression Code Generation
//!
//! This module implements LLVM code generation for if expressions in Y-lang.
//! If expressions are control flow constructs that can optionally produce values,
//! requiring careful management of basic blocks and phi nodes.
//!
//! ## Control Flow Architecture
//!
//! If expressions are implemented using LLVM's basic block system:
//! - **Entry block**: Contains the condition evaluation and conditional branch
//! - **Then block**: Executed when condition is true
//! - **Else block**: Executed when condition is false
//! - **Merge block**: Convergence point after both branches
//!
//! ## Phi Node Generation
//!
//! When if expressions produce values, phi nodes are used to merge values from different branches:
//! - **Value-producing branches**: Create phi node with incoming values from both branches
//! - **Inconsistent branches**: Handle cases where only one branch produces a value
//! - **Void expressions**: No phi node needed, just control flow merging
//!
//! ## Scope Management
//!
//! Each branch (then/else) executes in its own lexical scope:
//! - Variables declared in branches don't leak to outer scope
//! - Inner scopes can shadow outer scope variables
//! - Scopes are properly cleaned up when branches exit
//!
//! ## Terminator Management
//!
//! The code carefully manages LLVM basic block terminators:
//! - Checks for existing terminators before adding branches
//! - Handles early returns and other control flow within branches
//! - Ensures all blocks are properly terminated

use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::If,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for If<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    /// Generates LLVM IR for if expressions with proper control flow and value merging.
    ///
    /// This method implements the complete control flow pattern for if expressions,
    /// including basic block creation, conditional branching, scope management,
    /// and phi node generation for value merging when needed.
    ///
    /// ## Implementation Steps
    ///
    /// 1. **Condition evaluation**: Generate IR for the condition expression
    /// 2. **Basic block creation**: Create then, else, and merge blocks
    /// 3. **Conditional branch**: Branch to then/else based on condition
    /// 4. **Branch execution**: Generate IR for both branches with proper scoping
    /// 5. **Value merging**: Use phi nodes to merge values from both branches
    /// 6. **Control flow merging**: Ensure all paths converge at merge block
    ///
    /// ## Basic Block Management
    ///
    /// The implementation creates three basic blocks:
    /// - **then_block**: Executed when condition is true (i1 true)
    /// - **else_block**: Executed when condition is false (i1 false)
    /// - **merge_block**: Convergence point for both execution paths
    ///
    /// ## Terminator Handling
    ///
    /// Before adding unconditional branches to merge_block, the code checks
    /// if the current block already has a terminator (like return statements).
    /// This prevents LLVM IR validation errors from double-terminated blocks.
    ///
    /// ## Phi Node Strategy
    ///
    /// Phi nodes are created when:
    /// - Both branches produce values of the same type
    /// - The if expression is used in a value context
    /// - Special handling for if-without-else that produces values
    ///
    /// ## Scope Isolation
    ///
    /// Each branch executes in its own scope to prevent variable leakage:
    /// - `enter_scope()` before branch execution
    /// - `exit_scope()` after branch completion
    /// - Variables declared in branches don't affect outer scope
    ///
    /// # Returns
    ///
    /// - `Some(BasicValueEnum)`: When if expression produces a value
    /// - `None`: When if expression is used for control flow only
    ///
    /// # LLVM Operations Used
    ///
    /// - **`build_conditional_branch`**: For condition-based control flow
    /// - **`build_unconditional_branch`**: For merging control flow
    /// - **`build_phi`**: For value merging from different branches
    /// - **`append_basic_block`**: For creating control flow blocks
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        // Generate IR for the condition expression (must produce i1 boolean value)
        let condition_value = self.condition.codegen(ctx)?;

        // Get the current function context to create new basic blocks
        // All basic blocks must belong to a function in LLVM
        let current_function = ctx.builder.get_insert_block()?.get_parent()?;

        // Create the three basic blocks needed for if-expression control flow
        // Block names are used for debugging and IR readability
        let then_block = ctx.context.append_basic_block(current_function, "if_then");
        let else_block = ctx.context.append_basic_block(current_function, "if_else");
        let merge_block = ctx.context.append_basic_block(current_function, "if_merge");

        // Generate conditional branch instruction: br i1 %cond, label %then, label %else
        // This terminates the current basic block and transfers control based on condition
        ctx.builder
            .build_conditional_branch(condition_value.into_int_value(), then_block, else_block)
            .ok()?;

        // === THEN BLOCK GENERATION ===
        // Move IR builder to then block and generate its code
        ctx.builder.position_at_end(then_block);
        let mut then_value = None;

        // Create isolated scope for then block to prevent variable leakage
        ctx.enter_scope();

        // Generate code for the then block (may or may not produce a value)
        then_value = self.then_block.codegen(ctx);

        // Clean up scope - variables declared in then block are discarded
        ctx.exit_scope();

        // Add unconditional branch to merge block if block isn't already terminated
        // Termination can occur from return statements or other control flow
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder.build_unconditional_branch(merge_block).ok()?;
        }

        // === ELSE BLOCK GENERATION ===
        // Move IR builder to else block and generate its code
        ctx.builder.position_at_end(else_block);
        let mut else_value = None;

        // Create isolated scope for else block
        ctx.enter_scope();

        // Generate code for the else block (may or may not produce a value)
        else_value = self.else_block.codegen(ctx);

        // Clean up else block scope
        ctx.exit_scope();

        // Add unconditional branch to merge block if not already terminated
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            ctx.builder.build_unconditional_branch(merge_block).ok()?;
        }

        // === MERGE BLOCK AND VALUE MERGING ===
        // Position IR builder at the merge block where control flow converges
        ctx.builder.position_at_end(merge_block);

        // Determine if and how to merge values from both branches
        match (then_value, else_value) {
            // Both branches produced values - create phi node to merge them
            (Some(then_val), Some(else_val)) => {
                // Create phi node with the type of the values (both should have same type)
                // Phi nodes implement the concept: "this value comes from different predecessors"
                let phi = ctx
                    .builder
                    .build_phi(then_val.get_type(), "if_result")
                    .ok()?;

                // Add incoming edges: associate each value with its originating basic block
                // This tells LLVM which value to use based on which branch was taken
                phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);

                // Return the phi node as the result of the if expression
                Some(phi.as_basic_value())
            }
            // Only then branch produced a value (if-without-else case)
            (Some(then_val), None) if self.else_block.statements.is_empty() => {
                // This is an unusual case in Y-lang but we handle it gracefully
                // The if expression evaluates to the then value when condition is true,
                // and undefined behavior when condition is false
                Some(then_val)
            }
            // No values produced or inconsistent value production
            _ => {
                // This occurs when:
                // - Both branches are used for side effects only (no return values)
                // - Only one branch produces a value but else block is non-empty
                // - Expression is used in statement context rather than value context
                None
            }
        }
    }
}
