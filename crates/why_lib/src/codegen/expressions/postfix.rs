use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

use crate::{
    codegen::{CodeGen, CodegenContext, build_llvm_function_type_from_own_types},
    parser::ast::{Expression, Postfix},
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Postfix<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

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
    ) -> Option<BasicValueEnum<'ctx>> {
        let type_id = expr.get_info().type_id;
        let Some(expr) = expr.codegen(ctx) else {
            unreachable!()
        };

        let args = args
            .iter()
            .map(|arg| {
                let Some(arg) = arg.codegen(ctx) else {
                    unreachable!()
                };
                arg.into()
            })
            .collect::<Vec<BasicMetadataValueEnum<'ctx>>>();

        match type_id {
            Type::Function {
                params,
                return_value,
            } => Self::codegen_function_call(ctx, expr, args, params, *return_value),
            Type::Lambda {
                params,
                return_value,
                captures,
            } => Self::codegen_lambda_call(ctx, expr, args, params, *return_value, captures),
            other => unreachable!("postfix calls are not allowed for: {other:#?}"),
        }
    }

    fn codegen_function_call(
        ctx: &CodegenContext<'ctx>,
        expr: BasicValueEnum<'ctx>,
        args: Vec<BasicMetadataValueEnum<'ctx>>,
        params: Vec<Type>,
        return_type: Type,
    ) -> Option<BasicValueEnum<'ctx>> {
        let llvm_function_type =
            build_llvm_function_type_from_own_types(ctx, &return_type, &params);

        let BasicValueEnum::PointerValue(llvm_fn_pointer) = expr else {
            unreachable!("The Expression in a Call-Postfix should always return a pointer");
        };

        ctx.builder
            .build_indirect_call(llvm_function_type, llvm_fn_pointer, &args, "")
            .unwrap()
            .try_as_basic_value()
            .left()
    }

    fn codegen_lambda_call(
        ctx: &CodegenContext<'ctx>,
        expr: BasicValueEnum<'ctx>,
        args: Vec<BasicMetadataValueEnum<'ctx>>,
        params: Vec<Type>,
        return_type: Type,
        captures: Vec<String>,
    ) -> Option<BasicValueEnum<'ctx>> {
        let llvm_function_type =
            build_llvm_function_type_from_own_types(ctx, &return_type, &params);

        let BasicValueEnum::PointerValue(llvm_fn_pointer) = expr else {
            unreachable!("The Expression in a Call-Postfix should always return a pointer");
        };

        ctx.builder
            .build_indirect_call(llvm_function_type, llvm_fn_pointer, &args, "")
            .unwrap()
            .try_as_basic_value()
            .left()
    }
}
