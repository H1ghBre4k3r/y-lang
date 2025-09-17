//! # Character Literal Code Generation
//!
//! This module implements LLVM code generation for character literals in Y-lang.
//! Characters are represented as LLVM i8 types containing the UTF-8 byte value.
//!
//! ## Character Encoding
//!
//! - **Representation**: UTF-8 single-byte characters as i8
//! - **Conversion**: Rust char -> u8 -> i8 for LLVM compatibility
//! - **Limitation**: Currently supports ASCII/single-byte UTF-8 only
//!
//! This approach provides C-style character compatibility while maintaining
//! UTF-8 encoding for basic character sets.

use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Character,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Character<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    /// Generates LLVM IR for character literals.
    ///
    /// Converts Y-lang character literals to LLVM i8 constants representing
    /// the UTF-8 byte value of the character.
    ///
    /// # Returns
    ///
    /// LLVM i8 constant representing the character's byte value
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let ValidatedTypeInformation { type_id, .. } = &self.info;

        let llvm_type = ctx.get_llvm_type(type_id).into_int_type();
        // Convert char to its UTF-8 byte representation
        let char_value = self.character as u8;
        llvm_type.const_int(char_value as u64, false).into()
    }
}
