use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{convert_metadata_to_basic, CodeGen},
    parser::ast::Id,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Id<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let Id {
            name,
            info: ValidatedTypeInformation { type_id, .. },
            ..
        } = self;

        let variable = ctx.find_variable(name);

        match variable {
            BasicValueEnum::PointerValue(pointer_value) => {
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
