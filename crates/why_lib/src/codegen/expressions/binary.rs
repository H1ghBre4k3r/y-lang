use inkwell::{types::BasicMetadataTypeEnum, values::BasicValueEnum, FloatPredicate, IntPredicate};

use crate::{
    codegen::CodeGen,
    parser::ast::{BinaryExpression, BinaryOperator},
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for BinaryExpression<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

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

        match (ctx.get_llvm_type(type_id), operator) {
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
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::Divide) => ctx
                .builder
                .build_int_signed_div(left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
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
            // Comparison operations for integers - return boolean
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::Equals) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::EQ,
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::NotEquals) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::NE,
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::LessThan) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::SLT,
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::LessOrEqual) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::SLE,
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::GreaterThan) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::SGT,
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::GreaterOrEqual) => ctx
                .builder
                .build_int_compare(
                    IntPredicate::SGE,
                    left.into_int_value(),
                    right.into_int_value(),
                    "",
                )
                .unwrap()
                .into(),

            // Comparison operations for floats - return boolean
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::Equals) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OEQ,
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::NotEquals) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::ONE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::LessThan) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OLT,
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::LessOrEqual) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OLE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::GreaterThan) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OGT,
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::GreaterOrEqual) => ctx
                .builder
                .build_float_compare(
                    FloatPredicate::OGE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "",
                )
                .unwrap()
                .into(),

            // String operations - only comparison makes sense at LLVM level (pointer comparison)
            (BasicMetadataTypeEnum::PointerType(_), BinaryOperator::Equals) => {
                // Pointer equality comparison
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
                // Pointer inequality comparison
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
