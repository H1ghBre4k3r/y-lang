use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Num,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Num<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        match self {
            Num::Integer(val, info, ..) => {
                let ValidatedTypeInformation { type_id, .. } = info;

                let llvm_type = ctx.get_llvm_type(type_id).into_int_type();

                llvm_type.const_int(*val, false).into()
            }
            Num::FloatingPoint(_, _, span) => todo!(),
        }
    }
}
