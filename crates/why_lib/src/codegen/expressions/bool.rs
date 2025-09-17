//! # Boolean Literal Code Generation
//!
//! This module implements LLVM code generation for boolean literals in Y-lang.
//! Boolean values are represented as LLVM i1 (1-bit integer) types with standard
//! C-style true/false encoding.
//!
//! ## Boolean Representation
//!
//! - **true**: Encoded as integer constant 1
//! - **false**: Encoded as integer constant 0
//! - **LLVM type**: i1 (1-bit integer type)
//!
//! This follows standard C/LLVM conventions for boolean representation.

use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Bool,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Bool<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    /// Generates LLVM IR for boolean literals.
    ///
    /// Converts Y-lang boolean literals to LLVM i1 constants using standard
    /// true=1, false=0 encoding.
    ///
    /// # Returns
    ///
    /// LLVM i1 constant representing the boolean value
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let ValidatedTypeInformation { type_id, .. } = &self.info;

        let llvm_type = ctx.get_llvm_type(type_id).into_int_type();
        // true = 1, false = 0 (standard C convention)
        let value = if self.value { 1 } else { 0 };
        llvm_type.const_int(value, false).into()
    }
}
