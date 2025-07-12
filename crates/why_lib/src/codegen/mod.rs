mod expressions;
mod statements;

use std::{cell::RefCell, collections::HashMap};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, BasicTypeEnum},
    values::BasicValueEnum,
};

use crate::typechecker::Type;

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub types: RefCell<HashMap<Type, BasicMetadataTypeEnum<'ctx>>>,
    pub variables: RefCell<Vec<RefCell<HashMap<String, BasicValueEnum<'ctx>>>>>,
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

    pub fn enter_scope(&self) {
        self.variables
            .borrow_mut()
            .push(RefCell::new(HashMap::default()));
    }

    pub fn exit_scope(&self) {
        self.variables.borrow_mut().pop();
    }

    pub fn find_variable(&self, name: impl ToString) -> BasicValueEnum<'ctx> {
        let name = name.to_string();
        let variables = self.variables.borrow().clone();

        variables
            .iter()
            .rev()
            .find(|scope| scope.borrow().contains_key(&name))
            .and_then(|scope| scope.borrow().get(&name).cloned())
            .unwrap()
    }

    pub fn store_variable(&self, name: impl ToString, value: BasicValueEnum<'ctx>) {
        let name = name.to_string();

        let variables = self.variables.borrow();

        variables
            .last()
            .and_then(|scope| scope.borrow_mut().insert(name, value));
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

fn convert_metadata_to_basic(ty: BasicMetadataTypeEnum) -> Option<BasicTypeEnum> {
    match ty {
        BasicMetadataTypeEnum::ArrayType(t) => Some(BasicTypeEnum::ArrayType(t)),
        BasicMetadataTypeEnum::FloatType(t) => Some(BasicTypeEnum::FloatType(t)),
        BasicMetadataTypeEnum::IntType(t) => Some(BasicTypeEnum::IntType(t)),
        BasicMetadataTypeEnum::PointerType(t) => Some(BasicTypeEnum::PointerType(t)),
        BasicMetadataTypeEnum::StructType(t) => Some(BasicTypeEnum::StructType(t)),
        BasicMetadataTypeEnum::VectorType(t) => Some(BasicTypeEnum::VectorType(t)),
        _ => None, // For metadata-only types that aren't BasicType-compatible
    }
}
