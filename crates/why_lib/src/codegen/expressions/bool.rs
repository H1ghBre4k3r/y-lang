use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Bool,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Bool<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let ValidatedTypeInformation { type_id, .. } = &self.info;

        let llvm_type = ctx.get_llvm_type(type_id).into_int_type();
        // true = 1, false = 0 (standard C convention)
        let value = if self.value { 1 } else { 0 };
        llvm_type.const_int(value, false).into()
    }
}
