mod expressions;
mod statements;

use std::{cell::RefCell, collections::HashMap};

use inkwell::{builder::Builder, context::Context, module::Module, types::BasicMetadataTypeEnum};

use crate::typechecker::Type;

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub types: RefCell<HashMap<Type, BasicMetadataTypeEnum<'ctx>>>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn get_llvm_type(&self, our_type: &Type) -> BasicMetadataTypeEnum<'ctx> {
        let mut types = self.types.borrow_mut();
        if let Some(entry) = types.get(our_type) {
            return *entry;
        }

        let new_type = our_type.to_llvm_type(self.context);
        types.insert(our_type.clone(), new_type);
        new_type
    }
}

pub trait CodeGen<'ctx> {
    type ReturnValue;
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue;
}

pub trait IntoLLVMType {
    fn to_llvm_type<'ctx>(&self, ctx: &'ctx Context) -> BasicMetadataTypeEnum<'ctx>;
}

impl IntoLLVMType for Type {
    fn to_llvm_type<'ctx>(&self, ctx: &'ctx Context) -> BasicMetadataTypeEnum<'ctx> {
        match self {
            Type::Integer => ctx.i64_type().into(),
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
            } => todo!(),
        }
    }
}
