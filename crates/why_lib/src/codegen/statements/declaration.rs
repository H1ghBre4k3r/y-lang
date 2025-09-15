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
            Type::Integer => {
                let llvm_type = ctx.context.i64_type();
                let llvm_alloca = ctx
                    .builder
                    .build_alloca(llvm_type, &name.name)
                    .expect("Failed to build alloca for integer");
                ctx.store_variable(&name.name, llvm_alloca.into());
            }
            Type::FloatingPoint => {
                let llvm_type = ctx.context.f64_type();
                let llvm_alloca = ctx
                    .builder
                    .build_alloca(llvm_type, &name.name)
                    .expect("Failed to build alloca for float");
                ctx.store_variable(&name.name, llvm_alloca.into());
            }
            Type::Boolean => {
                let llvm_type = ctx.context.bool_type();
                let llvm_alloca = ctx
                    .builder
                    .build_alloca(llvm_type, &name.name)
                    .expect("Failed to build alloca for bool");
                ctx.store_variable(&name.name, llvm_alloca.into());
            }
            Type::Character => {
                let llvm_type = ctx.context.i8_type();
                let llvm_alloca = ctx
                    .builder
                    .build_alloca(llvm_type, &name.name)
                    .expect("Failed to build alloca for char");
                ctx.store_variable(&name.name, llvm_alloca.into());
            }
            Type::String => {
                let llvm_type = ctx.context.ptr_type(Default::default());
                let llvm_alloca = ctx
                    .builder
                    .build_alloca(llvm_type, &name.name)
                    .expect("Failed to build alloca for string");
                ctx.store_variable(&name.name, llvm_alloca.into());
            }
            Type::Void => {
                // Void variables don't make sense - this is likely an error in type checking
                panic!("Cannot declare variable of void type: {}", name.name);
            }
            Type::Unknown => {
                panic!("Cannot declare variable of unknown type: {}", name.name);
            }
            Type::Reference(_inner_type) => {
                let llvm_type = ctx.context.ptr_type(Default::default());
                let llvm_alloca = ctx
                    .builder
                    .build_alloca(llvm_type, &name.name)
                    .expect("Failed to build alloca for reference");
                ctx.store_variable(&name.name, llvm_alloca.into());
            }
            Type::Tuple(_items) => {
                // Use the general type conversion for tuples
                let llvm_type =
                    crate::codegen::convert_metadata_to_basic(ctx.get_llvm_type(type_id))
                        .expect("Failed to convert tuple type");
                let llvm_alloca = ctx
                    .builder
                    .build_alloca(llvm_type, &name.name)
                    .expect("Failed to build alloca for tuple");
                ctx.store_variable(&name.name, llvm_alloca.into());
            }
            Type::Array(_element_type) => {
                let llvm_type = ctx.context.ptr_type(Default::default());
                let llvm_alloca = ctx
                    .builder
                    .build_alloca(llvm_type, &name.name)
                    .expect("Failed to build alloca for array");
                ctx.store_variable(&name.name, llvm_alloca.into());
            }
            Type::Struct(_name, _items) => {
                // Use the general type conversion for structs
                let llvm_type =
                    crate::codegen::convert_metadata_to_basic(ctx.get_llvm_type(type_id))
                        .expect("Failed to convert struct type");
                let llvm_alloca = ctx
                    .builder
                    .build_alloca(llvm_type, &name.name)
                    .expect("Failed to build alloca for struct");
                ctx.store_variable(&name.name, llvm_alloca.into());
            }
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
