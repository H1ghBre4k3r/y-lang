use crate::{
    codegen::CodeGen,
    parser::ast::{Assignment, Expression, LValue, Postfix},
    typechecker::ValidatedTypeInformation,
};

/// Unified helper function to build pointers for any lvalue expression
/// Handles all assignment targets: variables, array elements, struct fields, and combinations
fn build_lvalue_pointer<'ctx>(
    ctx: &crate::codegen::CodegenContext<'ctx>,
    lvalue: &LValue<ValidatedTypeInformation>,
) -> inkwell::values::PointerValue<'ctx> {
    match lvalue {
        LValue::Id(id) => {
            // Simple variable access
            ctx.find_variable(&id.name).into_pointer_value()
        }
        LValue::Postfix(postfix) => match postfix {
            Postfix::Index { expr, index, .. } => {
                // Array element access: arr[index]
                let Some(array_value) = expr.codegen(ctx) else {
                    unreachable!("Array expression must produce a value")
                };

                let Some(index_value) = index.codegen(ctx) else {
                    unreachable!("Index expression must produce a value")
                };

                let array_ptr = array_value.into_pointer_value();
                let index_int = index_value.into_int_value();

                // Get array element type
                let expr_type = &expr.get_info().type_id;
                let crate::typechecker::Type::Array(element_type) = expr_type else {
                    unreachable!("Index expression must be on array type")
                };

                let llvm_element_type = ctx.get_llvm_type(element_type);
                let element_basic_type =
                    crate::codegen::convert_metadata_to_basic(llvm_element_type)
                        .expect("Array element type must be basic");

                // Build GEP to get pointer to the indexed element
                unsafe {
                    ctx.builder
                        .build_gep(
                            element_basic_type,
                            array_ptr,
                            &[index_int],
                            "array_index_ptr",
                        )
                        .unwrap()
                }
            }
            Postfix::PropertyAccess { expr, property, .. } => {
                // Struct field access: expr.field
                build_struct_field_pointer(ctx, expr, &property.name)
            }
            Postfix::Call { .. } => {
                panic!("Function calls cannot be used as lvalues in assignment");
            }
        },
    }
}

/// Helper function to build pointers to struct fields, handling nested expressions
fn build_struct_field_pointer<'ctx>(
    ctx: &crate::codegen::CodegenContext<'ctx>,
    expr: &Expression<ValidatedTypeInformation>,
    property_name: &str,
) -> inkwell::values::PointerValue<'ctx> {
    use crate::parser::ast::Expression::*;

    // Get the struct type and field index
    let (struct_name, field_types, field_index) = match &expr.get_info().type_id {
        crate::typechecker::Type::Struct(struct_name, field_types) => {
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
            panic!(
                "Property access assignment only supported on struct types, got: {:?}",
                other_type
            );
        }
    };

    // Get the LLVM struct type
    let struct_type = {
        let types = ctx.types.borrow();
        let struct_type_id =
            crate::typechecker::Type::Struct(struct_name.clone(), field_types.clone());

        match types.get(&struct_type_id) {
            Some(llvm_type) => {
                if let inkwell::types::BasicMetadataTypeEnum::StructType(struct_type) = llvm_type {
                    *struct_type
                } else {
                    panic!("Expected struct type for property access assignment");
                }
            }
            None => {
                panic!(
                    "Struct type {} not found in type context for property access assignment",
                    struct_name
                );
            }
        }
    };

    // Build pointer based on the expression type
    match expr {
        Id(id_expr) => {
            // Direct variable access: var.field
            let base_ptr = ctx.find_variable(&id_expr.name).into_pointer_value();
            unsafe {
                ctx.builder
                    .build_gep(
                        struct_type,
                        base_ptr,
                        &[
                            ctx.context.i32_type().const_zero(),
                            ctx.context.i32_type().const_int(field_index as u64, false),
                        ],
                        &format!("{}_{}_ptr", id_expr.name, property_name),
                    )
                    .unwrap()
            }
        }
        Postfix(nested_postfix) => {
            use crate::parser::ast::Postfix;
            match nested_postfix {
                Postfix::PropertyAccess {
                    expr: nested_expr,
                    property: nested_property,
                    ..
                } => {
                    // Nested property access: expr.nested.field
                    let nested_ptr =
                        build_struct_field_pointer(ctx, nested_expr, &nested_property.name);
                    unsafe {
                        ctx.builder
                            .build_gep(
                                struct_type,
                                nested_ptr,
                                &[
                                    ctx.context.i32_type().const_zero(),
                                    ctx.context.i32_type().const_int(field_index as u64, false),
                                ],
                                &format!(
                                    "{}_{}_{}_ptr",
                                    nested_ptr.get_name().to_string_lossy(),
                                    nested_property.name,
                                    property_name
                                ),
                            )
                            .unwrap()
                    }
                }
                Postfix::Index {
                    expr: array_expr,
                    index,
                    ..
                } => {
                    // Indexed struct access: arr[index].field
                    let Some(array_value) = array_expr.codegen(ctx) else {
                        panic!("Array expression must produce a value for indexed assignment");
                    };

                    let Some(index_value) = index.codegen(ctx) else {
                        panic!("Index expression must produce a value for indexed assignment");
                    };

                    let array_ptr = array_value.into_pointer_value();
                    let index_int = index_value.into_int_value();

                    // Get the array element type
                    let array_expr_type = &array_expr.get_info().type_id;
                    let crate::typechecker::Type::Array(element_type) = array_expr_type else {
                        panic!("Expected array type for indexed assignment");
                    };

                    // The element should be a struct type
                    let (element_struct_name, element_field_types) = match &**element_type {
                        crate::typechecker::Type::Struct(name, fields) => {
                            (name.clone(), fields.clone())
                        }
                        other => {
                            panic!("Expected struct element type for indexed struct assignment, got: {other:?}");
                        }
                    };

                    // Get the LLVM struct type for the element
                    let element_struct_type = {
                        let types = ctx.types.borrow();
                        let element_type_id = crate::typechecker::Type::Struct(
                            element_struct_name,
                            element_field_types,
                        );

                        match types.get(&element_type_id) {
                            Some(llvm_type) => {
                                if let inkwell::types::BasicMetadataTypeEnum::StructType(
                                    struct_type,
                                ) = llvm_type
                                {
                                    *struct_type
                                } else {
                                    panic!("Expected struct type for array element");
                                }
                            }
                            None => {
                                panic!("Struct type for array element not found");
                            }
                        }
                    };

                    // Build GEP to get pointer to the indexed struct element
                    let element_ptr = unsafe {
                        ctx.builder
                            .build_gep(
                                element_struct_type,
                                array_ptr,
                                &[index_int],
                                "indexed_struct_ptr",
                            )
                            .unwrap()
                    };

                    // Now build GEP to get the field pointer within the struct
                    unsafe {
                        ctx.builder
                            .build_gep(
                                struct_type,
                                element_ptr,
                                &[
                                    ctx.context.i32_type().const_zero(),
                                    ctx.context.i32_type().const_int(field_index as u64, false),
                                ],
                                &format!("indexed_{}_ptr", property_name),
                            )
                            .unwrap()
                    }
                }
                _ => {
                    // For other postfix types, fall back to loading (shouldn't happen for valid lvalues)
                    let Some(struct_value) = expr.codegen(ctx) else {
                        panic!(
                            "Struct expression must produce a value for property access assignment"
                        );
                    };

                    if struct_value.is_pointer_value() {
                        struct_value.into_pointer_value()
                    } else {
                        let temp_ptr = ctx
                            .builder
                            .build_alloca(struct_type, "temp_struct_assign")
                            .unwrap();
                        ctx.builder.build_store(temp_ptr, struct_value).unwrap();
                        temp_ptr
                    }
                }
            }
        }
        _ => {
            // For other expression types, fall back to loading
            let Some(struct_value) = expr.codegen(ctx) else {
                panic!("Struct expression must produce a value for property access assignment");
            };

            if struct_value.is_pointer_value() {
                struct_value.into_pointer_value()
            } else {
                let temp_ptr = ctx
                    .builder
                    .build_alloca(struct_type, "temp_struct_assign")
                    .unwrap();
                ctx.builder.build_store(temp_ptr, struct_value).unwrap();
                temp_ptr
            }
        }
    }
}

impl<'ctx> CodeGen<'ctx> for Assignment<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let Some(rvalue) = self.rvalue.codegen(ctx) else {
            unreachable!("Assignment rvalue must produce a value")
        };

        // Unified approach: build pointer for any lvalue, then store
        let lvalue_ptr = build_lvalue_pointer(ctx, &self.lvalue);
        ctx.builder.build_store(lvalue_ptr, rvalue).unwrap();
    }
}
