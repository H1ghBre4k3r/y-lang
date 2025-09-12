use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{convert_metadata_to_basic, CodeGen},
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

        // First try to find as a constant
        if let Some(constant) = ctx.find_constant(name) {
            // Constants are stored as global variable pointers, so we need to load their values
            return match constant {
                BasicValueEnum::PointerValue(pointer_value) => {
                    let Some(llvm_type) = convert_metadata_to_basic(ctx.get_llvm_type(type_id))
                    else {
                        return constant;
                    };

                    let val = ctx
                        .builder
                        .build_load(llvm_type, pointer_value, &format!("const_{}", name))
                        .unwrap();
                    val
                }
                _ => constant,
            };
        }

        // If not found as constant, try as a variable
        let variable = ctx.find_variable(name);

        match variable {
            BasicValueEnum::PointerValue(pointer_value) => {
                // Check if this is a function type - if so, return the pointer directly
                if matches!(type_id, Type::Function { .. }) {
                    return variable;
                }

                let Some(llvm_type) = convert_metadata_to_basic(ctx.get_llvm_type(type_id)) else {
                    return variable;
                };

                let val = ctx
                    .builder
                    .build_load(llvm_type, pointer_value, "")
                    .unwrap();
                val
            }
            variable => variable,
        }
    }
}
