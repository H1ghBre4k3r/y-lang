use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

use crate::{
    codegen::{
        convert_metadata_to_basic, statements::function::build_llvm_function_type_from_own_types,
        CodeGen, CodegenContext,
    },
    parser::ast::{Expression, Postfix},
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Postfix<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        match self {
            Postfix::Call { expr, args, .. } => Self::codegen_call(ctx, expr, args),
            Postfix::Index { expr, index, .. } => {
                let Some(array_value) = expr.codegen(ctx) else {
                    unreachable!("Array expression must produce a value")
                };

                let Some(index_value) = index.codegen(ctx) else {
                    unreachable!("Index expression must produce a value")
                };

                let array_ptr = array_value.into_pointer_value();
                let index_int = index_value.into_int_value();

                // We need to determine the array type from the original expression type
                let expr_type = &expr.get_info().type_id;
                let Type::Array(element_type) = expr_type else {
                    unreachable!("Index expression must be on array type")
                };

                let llvm_element_type = ctx.get_llvm_type(element_type);
                let element_basic_type = convert_metadata_to_basic(llvm_element_type)
                    .expect("Array element type must be basic");

                // Build GEP to get pointer to the indexed element
                // Since we have a pointer to an array, we need to use the element type and just the index
                let element_ptr = unsafe {
                    ctx.builder
                        .build_gep(
                            element_basic_type,
                            array_ptr,
                            &[index_int], // Just the index, no need for extra i32 0
                            "array_index",
                        )
                        .unwrap()
                };

                // Load the value from the element pointer
                let element_value = ctx
                    .builder
                    .build_load(element_basic_type, element_ptr, "array_elem")
                    .unwrap();

                Some(element_value)
            }
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
