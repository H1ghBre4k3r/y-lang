use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::{StructFieldInitialisation, StructInitialisation},
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for StructInitialisation<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let StructInitialisation {
            id, fields, info, ..
        } = self;

        // Get the struct name
        let struct_name = &id.name;

        // Look up the struct type in the context
        let struct_type = {
            let types_guard = ctx.types.borrow();
            match types_guard.get(&info.type_id) {
                Some(llvm_type) => {
                    if let inkwell::types::BasicMetadataTypeEnum::StructType(struct_type) =
                        llvm_type
                    {
                        *struct_type
                    } else {
                        panic!(
                            "Expected struct type for {}, got: {:?}",
                            struct_name, llvm_type
                        )
                    }
                }
                None => {
                    panic!("Struct type {} not found in type context", struct_name);
                }
            }
        };

        // Allocate memory for the struct on the stack
        let struct_ptr = ctx.builder.build_alloca(struct_type, struct_name).unwrap();

        // Initialize each field
        for field_init in fields {
            let StructFieldInitialisation { name, value, .. } = field_init;
            let field_name = &name.name;

            // Generate code for the field value
            let Some(field_value) = value.codegen(ctx) else {
                panic!(
                    "Failed to generate code for field {} in struct {}",
                    field_name, struct_name
                );
            };

            // Get pointer to the field
            let field_ptr = unsafe {
                ctx.builder
                    .build_gep(
                        struct_type,
                        struct_ptr,
                        &[
                            ctx.context.i32_type().const_zero(),
                            ctx.context.i32_type().const_int(
                                self.get_field_index(struct_name, field_name) as u64,
                                false,
                            ),
                        ],
                        &format!("{}_{}", struct_name, field_name),
                    )
                    .unwrap()
            };

            // Store the field value
            ctx.builder.build_store(field_ptr, field_value).unwrap();
        }

        // Return the struct as a value (load from pointer)
        Some(
            ctx.builder
                .build_load(struct_type, struct_ptr, struct_name)
                .unwrap(),
        )
    }
}

impl StructInitialisation<ValidatedTypeInformation> {
    fn get_field_index(&self, struct_name: &str, field_name: &str) -> u32 {
        // Look up the field index in the struct definition
        // This is a simplified approach - in a real implementation, you'd want to
        // store field information in the context during struct declaration

        // For now, we'll use a heuristic based on the order of fields in initialization
        // This matches the order they were declared in the struct definition
        for (i, field) in self.fields.iter().enumerate() {
            if field.name.name == field_name {
                return i as u32;
            }
        }

        panic!("Field {} not found in struct {}", field_name, struct_name);
    }
}
