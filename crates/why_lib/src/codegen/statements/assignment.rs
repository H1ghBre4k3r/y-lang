use crate::{
    codegen::CodeGen,
    parser::ast::{Assignment, Expression, LValue, Postfix},
    typechecker::ValidatedTypeInformation,
};

/// Helper function to build efficient pointer chains for nested assignments
/// Handles patterns like `a.b.c = value` by recursively analyzing postfix expressions
/// and building direct GEP pointer chains instead of temporary allocations.
fn build_nested_assignment_pointer<'ctx>(
    ctx: &crate::codegen::CodegenContext<'ctx>,
    expr: &Expression<ValidatedTypeInformation>,
    property_name: &str,
    struct_type: inkwell::types::StructType<'ctx>,
    field_index: usize,
) -> inkwell::values::PointerValue<'ctx> {
    use crate::parser::ast::Expression::*;

    match expr {
        // Simple case: direct variable access (e.g., "a.b")
        Id(id_expr) => {
            // Get the base variable pointer
            let base_ptr = ctx.find_variable(&id_expr.name).into_pointer_value();

            // Build GEP to get the field pointer directly
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

        // Nested case: another postfix expression (e.g., "b.foo.b" where "b.foo" is PropertyAccess)
        Postfix(nested_postfix) => {
            use crate::parser::ast::Postfix;
            match nested_postfix {
                Postfix::PropertyAccess {
                    expr: nested_expr,
                    property: nested_property,
                    ..
                } => {
                    // Recursively build the pointer chain
                    let nested_type_info = &nested_expr.get_info().type_id;
                    let (nested_struct_name, nested_field_types, nested_field_index) =
                        match nested_type_info {
                            crate::typechecker::Type::Struct(name, fields) => {
                                let index = fields
                                    .iter()
                                    .position(|(name, _)| name == &nested_property.name)
                                    .unwrap_or_else(|| {
                                        panic!(
                                            "Field {field_name} not found in struct {name}",
                                            field_name = nested_property.name,
                                        )
                                    });
                                (name.clone(), fields.clone(), index)
                            }
                            other => {
                                panic!("Expected struct type in nested assignment, got: {other:?}",)
                            }
                        };

                    // Get the nested struct type
                    let nested_struct_type = {
                        let types = ctx.types.borrow();
                        let nested_type_id = crate::typechecker::Type::Struct(
                            nested_struct_name.clone(),
                            nested_field_types,
                        );

                        match types.get(&nested_type_id) {
                            Some(llvm_type) => {
                                if let inkwell::types::BasicMetadataTypeEnum::StructType(
                                    struct_type,
                                ) = llvm_type
                                {
                                    *struct_type
                                } else {
                                    panic!("Expected struct type for nested assignment");
                                }
                            }
                            None => {
                                panic!(
                                    "Struct type {} not found for nested assignment",
                                    nested_struct_name
                                );
                            }
                        }
                    };

                    // Recursively build pointer to the nested field
                    let nested_ptr = build_nested_assignment_pointer(
                        ctx,
                        nested_expr,
                        &nested_property.name,
                        nested_struct_type,
                        nested_field_index,
                    );

                    // Build GEP to get the final field pointer from the nested pointer
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
                _ => {
                    // For other postfix types (Index, Call), fall back to the old method
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

        // For other expression types, fall back to the existing method
        _ => {
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

        match &self.lvalue {
            LValue::Id(id) => {
                // Simple variable assignment - store to existing variable
                let variable_ptr = ctx.find_variable(&id.name);
                ctx.builder
                    .build_store(variable_ptr.into_pointer_value(), rvalue)
                    .unwrap();
            }
            LValue::Postfix(postfix) => {
                // Handle complex lvalue assignments: obj.field = value and arr[index] = value
                match postfix {
                    Postfix::PropertyAccess { expr, property, .. } => {
                        // Get the property name
                        let property_name = &property.name;

                        // Get struct type and field index from the expression being accessed
                        let (struct_name, field_types, field_index) = match &expr.get_info().type_id
                        {
                            crate::typechecker::Type::Struct(struct_name, field_types) => {
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
                                panic!(
                                    "Property access assignment only supported on struct types, got: {:?}",
                                    other_type
                                );
                            }
                        };

                        // Get the struct type from the context
                        let struct_type = {
                            let types = ctx.types.borrow();
                            let struct_type_id = crate::typechecker::Type::Struct(
                                struct_name.clone(),
                                field_types.clone(),
                            );

                            match types.get(&struct_type_id) {
                                Some(llvm_type) => {
                                    if let inkwell::types::BasicMetadataTypeEnum::StructType(
                                        struct_type,
                                    ) = llvm_type
                                    {
                                        *struct_type
                                    } else {
                                        panic!(
                                            "Expected struct type for property access assignment, got: {:?}",
                                            llvm_type
                                        )
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

                        // Use the nested assignment pointer helper to build efficient pointer chains
                        // This handles both simple cases (a.b) and nested cases (a.b.c) efficiently
                        let field_ptr = build_nested_assignment_pointer(
                            ctx,
                            expr,
                            property_name,
                            struct_type,
                            field_index,
                        );

                        // Store the rvalue to the field pointer (instead of loading like in expression postfix)
                        ctx.builder.build_store(field_ptr, rvalue).unwrap();
                    }
                    Postfix::Index { expr, index, .. } => {
                        let Some(array_value) = expr.codegen(ctx) else {
                            unreachable!(
                                "Array expression must produce a value for index assignment"
                            )
                        };

                        let Some(index_value) = index.codegen(ctx) else {
                            unreachable!(
                                "Index expression must produce a value for index assignment"
                            )
                        };

                        let array_ptr = array_value.into_pointer_value();
                        let index_int = index_value.into_int_value();

                        // We need to determine the array type from the original expression type
                        let expr_type = &expr.get_info().type_id;
                        let crate::typechecker::Type::Array(element_type) = expr_type else {
                            unreachable!("Index assignment expression must be on array type")
                        };

                        let llvm_element_type = ctx.get_llvm_type(element_type);
                        let element_basic_type =
                            crate::codegen::convert_metadata_to_basic(llvm_element_type)
                                .expect("Array element type must be basic for index assignment");

                        // Build GEP to get pointer to the indexed element
                        // Since we have a pointer to an array, we need to use the element type and just the index
                        let element_ptr = unsafe {
                            ctx.builder
                                .build_gep(
                                    element_basic_type,
                                    array_ptr,
                                    &[index_int], // Just the index, no need for extra i32 0
                                    "array_index_ptr",
                                )
                                .unwrap()
                        };

                        // Store the rvalue to the element pointer (instead of loading like in expression postfix)
                        ctx.builder.build_store(element_ptr, rvalue).unwrap();
                    }
                    Postfix::Call { .. } => {
                        panic!("Function calls cannot be used as lvalues in assignment");
                    }
                }
            }
        }
    }
}
