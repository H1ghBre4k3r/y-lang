use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

use crate::{
    codegen::{
        statements::function::build_llvm_function_type_from_own_types, CodeGen, CodegenContext,
    },
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
        let Type::Function {
            params,
            return_value,
        } = expr.get_info().type_id
        else {
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

        // Check if this is a direct function call (Id expression)
        if let Expression::Id(id) = expr {
            // Try to find the function in the LLVM module by name
            let function_name = &id.name;
            if let Some(llvm_function) = ctx.module.get_function(function_name) {
                // Direct call to declared/defined function
                return ctx
                    .builder
                    .build_call(llvm_function, &args, "")
                    .unwrap()
                    .try_as_basic_value()
                    .left();
            }
        }

        // Fallback to indirect call through function pointer
        let llvm_function_type =
            build_llvm_function_type_from_own_types(ctx, &return_value, &params);
        let Some(expr_value) = expr.codegen(ctx) else {
            unreachable!()
        };

        let BasicValueEnum::PointerValue(llvm_fn_pointer) = expr_value else {
            unreachable!("The Expression in a Call-Postfix should always return a pointer");
        };

        ctx.builder
            .build_indirect_call(llvm_function_type, llvm_fn_pointer, &args, "")
            .unwrap()
            .try_as_basic_value()
            .left()
    }
}
