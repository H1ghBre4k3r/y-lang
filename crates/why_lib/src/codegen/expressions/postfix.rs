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
                ..
            } => {
                // Generate code for the struct expression
                let Some(struct_value) = expr.codegen(ctx) else {
                    panic!("Struct expression must produce a value for property access");
                };

                // Get the property name
                let property_name = &property.name;

                // Get struct type and field index from the expression being accessed
                // For chained access like foo.bar.value, we need the type of foo.bar (which is Bar)
                let (struct_name, field_types, field_index) = match &expr.get_info().type_id {
                    Type::Struct(struct_name, field_types) => {
                        // Find the field index by name
                        let field_index = field_types
                            .iter()
                            .position(|(name, _)| name == property_name)
                            .unwrap_or_else(|| {
                                panic!(
                                    "Field {} not found in struct {}",
                                    property_name, struct_name
                                )
                            });
                        (struct_name.clone(), field_types.clone(), field_index)
                    }
                    other_type => {
                        panic!("Property access only supported on struct types, got: {:?}", other_type);
                    }
                };

                // Get the struct type from the context
                let struct_type = {
                    let types_guard = ctx.types.borrow();
                    let struct_type_id = Type::Struct(struct_name.clone(), field_types.clone());

                    match types_guard.get(&struct_type_id) {
                        Some(llvm_type) => {
                            if let inkwell::types::BasicMetadataTypeEnum::StructType(struct_type) =
                                llvm_type
                            {
                                *struct_type
                            } else {
                                panic!(
                                    "Expected struct type for property access, got: {:?}",
                                    llvm_type
                                )
                            }
                        }
                        None => {
                            panic!(
                                "Struct type {} not found in type context for property access",
                                struct_name
                            );
                        }
                    }
                };

                // Allocate temporary storage for the struct if it's not already a pointer
                let struct_ptr = if struct_value.is_pointer_value() {
                    struct_value.into_pointer_value()
                } else {
                    // If it's not a pointer, allocate temporary storage and store the value
                    let temp_ptr = ctx
                        .builder
                        .build_alloca(struct_type, "temp_struct")
                        .unwrap();
                    ctx.builder.build_store(temp_ptr, struct_value).unwrap();
                    temp_ptr
                };

                // Get pointer to the field using GEP
                let field_ptr = unsafe {
                    ctx.builder
                        .build_gep(
                            struct_type,
                            struct_ptr,
                            &[
                                ctx.context.i32_type().const_zero(),
                                ctx.context.i32_type().const_int(field_index as u64, false),
                            ],
                            &format!(
                                "{}_{}",
                                struct_ptr.get_name().to_string_lossy(),
                                property_name
                            ),
                        )
                        .unwrap()
                };

                // Load the field value
                let field_value = ctx
                    .builder
                    .build_load(
                        struct_type
                            .get_field_type_at_index(field_index as u32)
                            .expect("Field type must exist"),
                        field_ptr,
                        property_name,
                    )
                    .unwrap();

                Some(field_value)
            }
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
