//! # Numeric Literal Code Generation
//!
//! This module implements LLVM code generation for numeric literals in Y-lang.
//! It handles both integer and floating-point literals with appropriate LLVM constant generation.
//!
//! ## Constant Generation
//!
//! Numeric literals are converted to LLVM constants at compile time:
//! - **Integer literals**: Use `const_int` with the appropriate bit width
//! - **Floating-point literals**: Use `const_float` with double precision
//!
//! ## Type-Driven Generation
//!
//! The LLVM type is determined from Y-lang's type system:
//! - Uses validated type information from the type checker
//! - Ensures numeric constants match their declared/inferred types
//! - Maintains type safety through the compilation pipeline

use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Num,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Num<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    /// Generates LLVM IR for numeric literals.
    ///
    /// This method converts Y-lang numeric literals to LLVM constant values.
    /// The type information from the type checker determines the appropriate
    /// LLVM type for the constant.
    ///
    /// ## Implementation Strategy
    ///
    /// 1. **Extract type information**: Get the validated Y-lang type
    /// 2. **Convert to LLVM type**: Use type cache for consistent types
    /// 3. **Create constant**: Generate appropriate LLVM constant value
    ///
    /// ## Integer Constants
    ///
    /// Integer literals are created using `const_int` with:
    /// - The literal value as a u64
    /// - `false` for the sign_extend parameter (handles signedness correctly)
    /// - Appropriate bit width from the LLVM integer type
    ///
    /// ## Floating-Point Constants
    ///
    /// Float literals are created using `const_float` with:
    /// - The literal value as an f64
    /// - LLVM handles precision conversion automatically
    ///
    /// # Returns
    ///
    /// LLVM constant value representing the numeric literal
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        match self {
            // Generate LLVM constant for integer literals
            Num::Integer(val, info, ..) => {
                let ValidatedTypeInformation { type_id, .. } = info;

                // Convert Y-lang integer type to LLVM integer type
                let llvm_type = ctx.get_llvm_type(type_id).into_int_type();

                // Create LLVM integer constant
                // false = don't sign extend (LLVM handles signedness based on type)
                llvm_type.const_int(*val, false).into()
            }
            // Generate LLVM constant for floating-point literals
            Num::FloatingPoint(val, info, _) => {
                let ValidatedTypeInformation { type_id, .. } = info;

                // Convert Y-lang float type to LLVM float type
                let llvm_type = ctx.get_llvm_type(type_id).into_float_type();

                // Create LLVM floating-point constant
                llvm_type.const_float(*val).into()
            }
        }
    }
}
