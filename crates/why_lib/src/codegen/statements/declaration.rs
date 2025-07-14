use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Declaration,
    typechecker::{Type, ValidatedTypeInformation},
};

use super::function::build_llvm_function_type_from_own_types;

impl<'ctx> CodeGen<'ctx> for Declaration<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let Declaration { name, .. } = self;
        let ValidatedTypeInformation { type_id, .. } = &name.info;

        match type_id {
            Type::Integer => todo!(),
            Type::FloatingPoint => todo!(),
            Type::Boolean => todo!(),
            Type::Character => todo!(),
            Type::String => todo!(),
            Type::Void => todo!(),
            Type::Unknown => todo!(),
            Type::Reference(_) => todo!(),
            Type::Tuple(items) => todo!(),
            Type::Array(_) => todo!(),
            Type::Struct(_, items) => todo!(),
            Type::Function {
                params,
                return_value,
            } => {
                let llvm_fn_type =
                    build_llvm_function_type_from_own_types(ctx, return_value, params);

                let llvm_fn_value = ctx.module.add_function(&name.name, llvm_fn_type, None);
                ctx.store_function(&name.name, llvm_fn_value);
            }
        }
    }
}
