use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Character,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Character<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let ValidatedTypeInformation { type_id, .. } = &self.info;

        let llvm_type = ctx.get_llvm_type(type_id).into_int_type();
        // Convert char to its UTF-8 byte representation
        let char_value = self.character as u8;
        llvm_type.const_int(char_value as u64, false).into()
    }
}
