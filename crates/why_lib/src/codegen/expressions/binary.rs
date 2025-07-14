use inkwell::{
    types::BasicMetadataTypeEnum,
    values::{BasicValueEnum, IntValue},
};

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

        let left = left.codegen(ctx);
        let right = right.codegen(ctx);

        match (ctx.get_llvm_type(type_id), operator) {
            (BasicMetadataTypeEnum::IntType(int_type), BinaryOperator::Add) => {
                let left = match left {
                    BasicValueEnum::IntValue(int_value) => int_value,
                    BasicValueEnum::PointerValue(pointer_value) => ctx
                        .builder
                        .build_ptr_to_int(pointer_value, int_type, "")
                        .unwrap(),
                    _ => unreachable!(),
                };
                let right = match right {
                    BasicValueEnum::IntValue(int_value) => int_value,
                    BasicValueEnum::PointerValue(pointer_value) => ctx
                        .builder
                        .build_ptr_to_int(pointer_value, int_type, "")
                        .unwrap(),
                    _ => unreachable!(),
                };
                ctx.builder.build_int_add(left, right, "").unwrap().into()
            }
            (BasicMetadataTypeEnum::IntType(int_type), BinaryOperator::Substract) => {
                let left = match left {
                    BasicValueEnum::IntValue(int_value) => int_value,
                    BasicValueEnum::PointerValue(pointer_value) => ctx
                        .builder
                        .build_ptr_to_int(pointer_value, int_type, "")
                        .unwrap(),
                    _ => unreachable!(),
                };
                let right = match right {
                    BasicValueEnum::IntValue(int_value) => int_value,
                    BasicValueEnum::PointerValue(pointer_value) => ctx
                        .builder
                        .build_ptr_to_int(pointer_value, int_type, "")
                        .unwrap(),
                    _ => unreachable!(),
                };
                ctx.builder.build_int_sub(left, right, "").unwrap().into()
            }
            (BasicMetadataTypeEnum::IntType(int_type), BinaryOperator::Multiply) => {
                let left = match left {
                    BasicValueEnum::IntValue(int_value) => int_value,
                    BasicValueEnum::PointerValue(pointer_value) => ctx
                        .builder
                        .build_ptr_to_int(pointer_value, int_type, "")
                        .unwrap(),
                    _ => unreachable!(),
                };
                let right = match right {
                    BasicValueEnum::IntValue(int_value) => int_value,
                    BasicValueEnum::PointerValue(pointer_value) => ctx
                        .builder
                        .build_ptr_to_int(pointer_value, int_type, "")
                        .unwrap(),
                    _ => unreachable!(),
                };
                ctx.builder.build_int_mul(left, right, "").unwrap().into()
            }
            (BasicMetadataTypeEnum::IntType(int_type), BinaryOperator::Divide) => {
                let left = match left {
                    BasicValueEnum::IntValue(int_value) => int_value,
                    BasicValueEnum::PointerValue(pointer_value) => ctx
                        .builder
                        .build_ptr_to_int(pointer_value, int_type, "")
                        .unwrap(),
                    _ => unreachable!(),
                };
                let right = match right {
                    BasicValueEnum::IntValue(int_value) => int_value,
                    BasicValueEnum::PointerValue(pointer_value) => ctx
                        .builder
                        .build_ptr_to_int(pointer_value, int_type, "")
                        .unwrap(),
                    _ => unreachable!(),
                };
                ctx.builder
                    .build_int_signed_div(left, right, "")
                    .unwrap()
                    .into()
            }
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
            _ => todo!(),
        }
    }
}
