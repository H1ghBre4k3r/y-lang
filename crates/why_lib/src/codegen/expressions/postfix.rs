use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

use crate::{
    codegen::{
        statements::function::build_llvm_function_type_from_own_types, CodeGen, CodegenContext,
    },
    parser::ast::{Expression, Postfix},
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Postfix<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        match self {
            Postfix::Call { expr, args, .. } => Self::codegen_call(ctx, expr, args),
            Postfix::Index {
                expr,
                index,
                info,
                position,
            } => todo!(),
            Postfix::PropertyAccess {
                expr,
                property,
                info,
                position,
            } => todo!(),
        }
    }
}

impl<'ctx> Postfix<ValidatedTypeInformation> {
    fn codegen_call(
        ctx: &CodegenContext<'ctx>,
        expr: &Expression<ValidatedTypeInformation>,
        args: &[Expression<ValidatedTypeInformation>],
    ) -> BasicValueEnum<'ctx> {
        let Type::Function {
            params,
            return_value,
        } = expr.get_info().type_id
        else {
            unreachable!()
        };

        let llvm_function_type =
            build_llvm_function_type_from_own_types(ctx, &return_value, &params);
        let expr = expr.codegen(ctx);

        let BasicValueEnum::PointerValue(llvm_fn_pointer) = expr else {
            unreachable!("The Expression in a Call-Postfix should always return a pointer");
        };

        let args = args
            .iter()
            .map(|arg| arg.codegen(ctx).into())
            .collect::<Vec<BasicMetadataValueEnum<'ctx>>>();

        ctx.builder
            .build_indirect_call(llvm_function_type, llvm_fn_pointer, &args, "")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
    }
}
