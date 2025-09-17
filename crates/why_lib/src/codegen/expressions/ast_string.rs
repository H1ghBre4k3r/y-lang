//! # String Literal Code Generation
//!
//! This module implements LLVM code generation for string literals in Y-lang.
//! Strings are represented as global string constants with pointer access.
//!
//! ## String Representation
//!
//! - **Storage**: Global string constants in the LLVM module
//! - **Access**: Via pointer to the global string data
//! - **Encoding**: UTF-8 with null termination (C-style strings)
//!
//! This approach provides efficient string storage and C interoperability.

use inkwell::values::BasicValueEnum;

use crate::{codegen::CodeGen, parser::ast::AstString, typechecker::ValidatedTypeInformation};

impl<'ctx> CodeGen<'ctx> for AstString<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    /// Generates LLVM IR for string literals.
    ///
    /// Creates a global string constant in the LLVM module and returns
    /// a pointer to it. This provides efficient string storage and sharing.
    ///
    /// # Returns
    ///
    /// Pointer to the global string constant
    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let AstString { value, .. } = self;
        let hello_str = ctx.builder.build_global_string_ptr(value, "").unwrap();
        hello_str.as_pointer_value().into()
    }
}
