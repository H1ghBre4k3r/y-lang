//! # Prefix Expression Code Generation
//!
//! This module implements LLVM code generation for prefix (unary) expressions in Y-lang.
//! It handles logical negation and arithmetic negation with type-specific operations.
//!
//! ## Supported Operations
//!
//! ### Logical Negation (`!expr`)
//! - **Boolean negation**: Uses LLVM's `build_not` for bitwise NOT on i1 values
//! - **Type restriction**: Only valid for boolean types
//!
//! ### Arithmetic Negation (`-expr`)
//! - **Integer negation**: Uses LLVM's `build_int_neg` for two's complement negation
//! - **Float negation**: Uses LLVM's `build_float_neg` for IEEE 754 sign bit flip
//! - **Character negation**: Treats characters as integers for ASCII arithmetic
//!
//! ## Type Safety
//!
//! The implementation includes comprehensive type checking with descriptive error
//! messages for invalid type/operator combinations, helping catch type system issues.

use inkwell::values::BasicValueEnum;

use crate::{
    codegen::CodeGen,
    parser::ast::Prefix,
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Prefix<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    /// Generates LLVM IR for prefix (unary) expressions.
    ///
    /// Dispatches to appropriate LLVM unary operations based on the operator
    /// and operand type, with comprehensive type validation.
    ///
    /// # Returns
    ///
    /// LLVM value representing the result of the unary operation
    ///
    /// # Panics
    ///
    /// Panics with descriptive messages for invalid type/operator combinations
    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        match self {
            Prefix::Negation { expr, .. } => {
                let expr_type = expr.get_info().type_id;
                let Some(expr_value) = expr.codegen(ctx) else {
                    unreachable!()
                };

                match expr_type {
                    Type::Boolean => {
                        let expr = expr_value.into_int_value();
                        // Boolean negation: !true = false (0), !false = true (1)
                        ctx.builder.build_not(expr, "").unwrap().into()
                    }
                    _ => unreachable!("Negation operator only valid for boolean types"),
                }
            }
            Prefix::Minus { expr, .. } => {
                let expr_type = expr.get_info().type_id;
                let Some(expr) = expr.codegen(ctx) else {
                    unreachable!()
                };

                match expr_type {
                    Type::Integer => {
                        let expr = expr.into_int_value();

                        ctx.builder.build_int_neg(expr, "").unwrap().into()
                    }
                    Type::FloatingPoint => {
                        let expr = expr.into_float_value();

                        ctx.builder.build_float_neg(expr, "").unwrap().into()
                    }

                    Type::Boolean => {
                        // Boolean negation using unary minus doesn't make semantic sense
                        panic!("Unary minus operator (-) is not valid for boolean types. Use logical negation (!) instead.");
                    }
                    Type::Character => {
                        // Character negation could be valid (negate ASCII value)
                        let expr = expr.into_int_value();
                        ctx.builder.build_int_neg(expr, "").unwrap().into()
                    }
                    Type::String => {
                        panic!("Unary minus operator (-) is not valid for string types.");
                    }
                    Type::Void => {
                        panic!("Unary minus operator (-) is not valid for void types.");
                    }
                    Type::Unknown => {
                        panic!("Cannot apply unary minus to unknown type.");
                    }
                    Type::Reference(_) => {
                        panic!("Unary minus operator (-) is not valid for reference types.");
                    }
                    Type::Tuple(_items) => {
                        panic!("Unary minus operator (-) is not valid for tuple types.");
                    }
                    Type::Array(_) => {
                        panic!("Unary minus operator (-) is not valid for array types.");
                    }
                    Type::Struct(_, _items) => {
                        panic!("Unary minus operator (-) is not valid for struct types.");
                    }
                    Type::Function { .. } => {
                        panic!("Unary minus operator (-) is not valid for function types.");
                    }
                }
            }
        }
    }
}
