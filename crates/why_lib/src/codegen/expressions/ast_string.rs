use inkwell::values::BasicValueEnum;

use crate::{codegen::CodeGen, parser::ast::AstString, typechecker::ValidatedTypeInformation};

impl<'ctx> CodeGen<'ctx> for AstString<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let AstString { value, .. } = self;
        let hello_str = ctx.builder.build_global_string_ptr(value, "").unwrap();
        hello_str.as_pointer_value().into()
    }
}
