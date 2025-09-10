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
                .build_int_compare(IntPredicate::EQ, left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::NotEquals) => ctx
                .builder
                .build_int_compare(IntPredicate::NE, left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::LessThan) => ctx
                .builder
                .build_int_compare(IntPredicate::SLT, left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::LessOrEqual) => ctx
                .builder
                .build_int_compare(IntPredicate::SLE, left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::GreaterThan) => ctx
                .builder
                .build_int_compare(IntPredicate::SGT, left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::IntType(_), BinaryOperator::GreaterOrEqual) => ctx
                .builder
                .build_int_compare(IntPredicate::SGE, left.into_int_value(), right.into_int_value(), "")
                .unwrap()
                .into(),
            
            // Comparison operations for floats - return boolean
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::Equals) => ctx
                .builder
                .build_float_compare(FloatPredicate::OEQ, left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::NotEquals) => ctx
                .builder
                .build_float_compare(FloatPredicate::ONE, left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::LessThan) => ctx
                .builder
                .build_float_compare(FloatPredicate::OLT, left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::LessOrEqual) => ctx
                .builder
                .build_float_compare(FloatPredicate::OLE, left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::GreaterThan) => ctx
                .builder
                .build_float_compare(FloatPredicate::OGT, left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),
            (BasicMetadataTypeEnum::FloatType(_), BinaryOperator::GreaterOrEqual) => ctx
                .builder
                .build_float_compare(FloatPredicate::OGE, left.into_float_value(), right.into_float_value(), "")
                .unwrap()
                .into(),
                
            _ => todo!(),
        }
    }
}
