use inkwell::values::BasicValueEnum;

use crate::{codegen::CodeGen, parser::ast::Id, typechecker::ValidatedTypeInformation};

impl<'ctx> CodeGen<'ctx> for Id<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let Id { name, .. } = self;

        ctx.find_variable(name)
    }
}
