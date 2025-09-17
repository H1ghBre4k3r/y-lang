//! # Identifier Expression Code Generation
//!
//! This module implements LLVM code generation for identifier expressions in Y-lang.
//! Identifiers can reference constants, variables, or functions with different
//! resolution and loading strategies.
//!
//! ## Resolution Order
//!
//! 1. **Constants**: Compile-time known values stored globally
//! 2. **Variables**: Runtime values stored in stack or as parameters
//! 3. **Functions**: Function declarations stored in the symbol table
//!
//! ## Type-Specific Handling
//!
//! - **Strings**: Returned as pointers (pass-by-reference semantics)
//! - **Functions**: May resolve to raw function pointers (direct calls) or closure structs when used as values; higher‑order positions should expect a closure struct
//! - **Other types**: Loaded from memory addresses as values
//!
//! ## Memory Access Patterns
//!
//! Variables and constants may be stored as pointers requiring load operations
//! to retrieve their actual values, except for types that are naturally
//! represented as pointers (strings, closures). In higher‑order positions (e.g. passing
//! as an argument expecting a function), identifiers must yield a closure struct; named
//! functions are wrapped lazily if needed.

use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{convert_metadata_to_basic, CodeGen},
    parser::ast::Id,
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Id<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    /// Generates LLVM IR for identifier expressions.
    ///
    /// Resolves identifiers by searching through constants and variables,
    /// handling type-specific loading and access patterns.
    ///
    /// # Returns
    ///
    /// The LLVM value corresponding to the identifier, with appropriate
    /// loading and type conversion applied
    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let Id {
            name,
            info: ValidatedTypeInformation { type_id, .. },
            ..
        } = self;

        // First try to find as a constant
        if let Some(constant) = ctx.find_constant(name) {
            // Constants are stored as global variable pointers, so we need to load their values
            return match constant {
                BasicValueEnum::PointerValue(pointer_value) => {
                    let Some(llvm_type) = convert_metadata_to_basic(ctx.get_llvm_type(type_id))
                    else {
                        return constant;
                    };

                    let val = ctx
                        .builder
                        .build_load(llvm_type, pointer_value, &format!("const_{}", name))
                        .unwrap();
                    val
                }
                _ => constant,
            };
        }

        // If not found as constant, try as a variable
        let variable = ctx.find_variable(name);

        let result = match variable {
            BasicValueEnum::PointerValue(pointer_value) => {
                // For string types, return the pointer directly (strings are passed by reference)
                if matches!(type_id, Type::String) {
                    variable
                } else if matches!(type_id, Type::Function { .. }) {
                    // For function types, load the closure struct from the pointer
                    let closure_struct_type = ctx.get_closure_struct_type();
                    let closure_value = ctx
                        .builder
                        .build_load(closure_struct_type, pointer_value, "")
                        .unwrap();
                    closure_value
                } else {
                    let Some(llvm_type) = convert_metadata_to_basic(ctx.get_llvm_type(type_id))
                    else {
                        return variable;
                    };

                    let val = ctx
                        .builder
                        .build_load(llvm_type, pointer_value, "")
                        .unwrap();
                    val
                }
            }
            variable => variable,
        };

        result
    }
}
