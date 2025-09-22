use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, convert_metadata_to_basic},
    parser::ast::Id,
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Id<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let Id {
            name,
            info: ValidatedTypeInformation { type_id, .. },
            ..
        } = self;

        // Check if this identifier refers to a function
        match type_id {
            Type::Function { .. } => {
                // For function references, we need to handle both direct function references
                // and variables that contain function pointers
                let function_value = ctx.try_find_function(name);

                match function_value {
                    Some(function) => {
                        // Direct function reference - return function pointer
                        function.as_global_value().as_pointer_value().into()
                    }
                    None => {
                        // Function stored as variable - load the function pointer
                        let variable = ctx.find_variable(name);
                        match variable {
                            BasicValueEnum::PointerValue(ptr) => {
                                // Load the function pointer from the variable
                                let function_ptr_type = ctx.context.ptr_type(Default::default());
                                ctx.builder.build_load(function_ptr_type, ptr, "").unwrap()
                            }
                            _ => variable, // Direct function pointer
                        }
                    }
                }
            }
            _ => {
                // Handle regular variables
                let variable = ctx.find_variable(name);

                match variable {
                    BasicValueEnum::PointerValue(pointer_value) => {
                        let Some(llvm_type) = convert_metadata_to_basic(ctx.get_llvm_type(type_id))
                        else {
                            return variable;
                        };

                        ctx.builder
                            .build_load(llvm_type, pointer_value, "")
                            .unwrap()
                    }
                    variable => variable,
                }
            }
        }
    }
}
