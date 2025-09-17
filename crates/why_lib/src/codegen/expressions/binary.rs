//! # Binary Expression Code Generation
//!
//! This module implements LLVM code generation for binary expressions in Y-lang.
//! It handles arithmetic operations, comparison operations, and type-specific
//! instruction selection based on LLVM's type system.
//!
//! ## Operation Categories
//!
//! ### Arithmetic Operations
//! - **Integer**: `add`, `sub`, `mul`, `sdiv` (signed division)
//! - **Float**: `fadd`, `fsub`, `fmul`, `fdiv`
//!
//! ### Comparison Operations
//! - **Integer**: Uses `IntPredicate` for signed comparisons (EQ, NE, SLT, SLE, SGT, SGE)
//! - **Float**: Uses `FloatPredicate` with ordered semantics (OEQ, ONE, OLT, OLE, OGT, OGE)
//! - **Pointer**: Limited to equality/inequality comparisons
//!
//! ## Type-Based Dispatch
//!
//! Operations are dispatched based on the LLVM type of the operands:
//! - `IntType`: Integer arithmetic and signed integer comparisons
//! - `FloatType`: Floating-point arithmetic and ordered float comparisons
//! - `PointerType`: Pointer equality/inequality only
//!
//! ## Error Handling
//!
//! Unsupported type/operation combinations result in descriptive panic messages
//! to aid in debugging type system issues.

use inkwell::{types::BasicMetadataTypeEnum, values::BasicValueEnum, FloatPredicate, IntPredicate};

use crate::{
    codegen::CodeGen,
    parser::ast::{BinaryExpression, BinaryOperator},
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for BinaryExpression<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    /// Generates LLVM IR for binary expressions.
    ///
    /// This method handles the generation of LLVM instructions for binary operations
    /// by first generating code for both operands, then dispatching to the appropriate
    /// LLVM instruction based on the combination of operation type and operand type.
    ///
    /// ## Code Generation Process
    ///
    /// 1. **Generate operands**: Create LLVM values for left and right expressions
    /// 2. **Type dispatch**: Match on the LLVM type to determine instruction family
    /// 3. **Operation dispatch**: Select specific instruction based on the operation
    /// 4. **Instruction generation**: Build the appropriate LLVM instruction
    ///
    /// ## LLVM Instruction Selection
    ///
    /// ### Integer Operations
    /// - **Arithmetic**: Uses `build_int_add`, `build_int_sub`, `build_int_mul`, `build_int_signed_div`
    /// - **Comparison**: Uses `build_int_compare` with signed predicates (SLT, SGT, etc.)
    /// - **Rationale**: Y-lang integers are signed, so we use signed operations
    ///
    /// ### Float Operations
    /// - **Arithmetic**: Uses `build_float_add`, `build_float_sub`, `build_float_mul`, `build_float_div`
    /// - **Comparison**: Uses `build_float_compare` with ordered predicates (OEQ, OLT, etc.)
    /// - **Rationale**: Ordered comparisons handle NaN properly by returning false
    ///
    /// ### Pointer Operations
    /// - **Equality**: Uses `build_int_compare` treating pointers as integers
    /// - **Limitation**: Only equality/inequality supported (no ordering)
    /// - **Rationale**: Pointer arithmetic is not part of Y-lang's type system
    ///
    /// ## Error Handling
    ///
    /// Unsupported combinations of types and operations result in panic with
    /// descriptive error messages that indicate:
    /// - The specific type that caused the issue
    /// - Why the operation is not supported
    /// - Suggested alternatives where applicable
    ///
    /// # Parameters
    ///
    /// * `ctx` - Code generation context containing LLVM state
    ///
    /// # Returns
    ///
    /// LLVM value representing the result of the binary operation
    ///
    /// # Panics
    ///
    /// - Unsupported operation for the given type combination
    /// - LLVM instruction building failures (wrapped and re-panicked)
    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let BinaryExpression {
            left,
            right,
            operator,
            info: ValidatedTypeInformation { type_id, .. },
            ..
        } = self;

        let Some(left) = left.codegen(ctx) else {
            unreachable!()
        };
        let Some(right) = right.codegen(ctx) else {
            unreachable!()
        };

        // Dispatch based on type and operation combination
        match (ctx.get_llvm_type(type_id), operator) {
            // Integer arithmetic operations
            // Using LLVM's integer arithmetic instructions for Y-lang integer type
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::Add) => ctx
                .builder
                .build_int_add(left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::Substract) => ctx
                .builder
                .build_int_sub(left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::Multiply) => ctx
                .builder
                .build_int_mul(left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            // Using signed division since Y-lang integers are signed
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::Divide) => ctx
                .builder
                .build_int_signed_div(left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            // Float arithmetic operations
            // Using LLVM's floating-point arithmetic instructions
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::Add) => ctx
                .builder
                .build_float_add(left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::Substract) => ctx
                .builder
                .build_float_sub(left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::Multiply) => ctx
                .builder
                .build_float_mul(left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::Divide) => ctx
                .builder
                .build_float_div(left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),

            // Integer comparison operations - return boolean (i1) type
            // Using signed predicates since Y-lang integers are signed
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::Equals) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::EQ, // Equal
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::NotEquals) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::NE, // Not Equal
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::LessThan) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::SLT, // Signed Less Than
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::LessOrEqual) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::SLE, // Signed Less than or Equal
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::GreaterThan) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::SGT, // Signed Greater Than
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::GreaterOrEqual) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::SGE, // Signed Greater than or Equal
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),

            // Float comparison operations - return boolean (i1) type
            // Using ordered predicates to handle NaN correctly (NaN comparisons return false)
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::Equals) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OEQ, // Ordered Equal (false if either operand is NaN)
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::NotEquals) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::ONE, // Ordered Not Equal (false if either operand is NaN)
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::LessThan) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OLT, // Ordered Less Than (false if either operand is NaN)
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::LessOrEqual) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OLE, // Ordered Less than or Equal
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::GreaterThan) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OGT, // Ordered Greater Than
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::GreaterOrEqual) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OGE, // Ordered Greater than or Equal
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),

            // Pointer/String operations - only equality comparison supported
            // Treating pointers as integers for comparison (address comparison)
            (BasicMetadataTypeEnum::PointerType(_), BinaryOperator::Equals) => {
                // Pointer equality: compares memory addresses
                ctx.builder
                    .build_int_compare(
                        IntPredicate::EQ,
                        left.into_pointer_value(),
                        right.into_pointer_value(),
                        "",
                    )
                    .unwrap()
                    .into()
            }
            (BasicMetadataTypeEnum::PointerType(_), BinaryOperator::NotEquals) => {
                // Pointer inequality: compares memory addresses
                ctx.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        left.into_pointer_value(),
                        right.into_pointer_value(),
                        "",
                    )
                    .unwrap()
                    .into()
            }

            // Unsupported operations - provide meaningful error messages
            (BasicMetadataTypeEnum::StructType(_), _) => {
                panic!("Binary operations on struct types are not supported. Structs cannot be used with arithmetic or comparison operators directly.");
            }
            (BasicMetadataTypeEnum::ArrayType(_), _) => {
                panic!("Binary operations on array types are not supported. Use array element access or iteration instead.");
            }
            (BasicMetadataTypeEnum::PointerType(_), op) => {
                panic!("Binary operation {:?} is not supported for pointer/string types. Only equality (==) and inequality (!=) are supported for pointers.", op);
            }

            // Catch-all for any remaining unsupported combinations
            (llvm_type, op) => {
                panic!(
                    "Binary operation {:?} is not supported for LLVM type {:?}",
                    op, llvm_type
                );
            }
        }
    }
}
