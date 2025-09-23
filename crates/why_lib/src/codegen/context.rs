use std::{cell::RefCell, collections::HashMap};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicMetadataTypeEnum,
    values::{BasicValueEnum, FunctionValue},
};

use crate::{
    codegen::{
        build_llvm_function_type_from_own_types, convert_our_type_to_llvm_basic_metadata_type,
    },
    typechecker::Type,
};

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub types: RefCell<HashMap<Type, BasicMetadataTypeEnum<'ctx>>>,
    pub scopes: RefCell<Vec<ScopeFrame<'ctx>>>,
    pub lambdas: RefCell<HashMap<String, FunctionValue<'ctx>>>,
}

pub type ScopeFrame<'ctx> = RefCell<Scope<'ctx>>;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Scope<'ctx> {
    variables: HashMap<String, BasicValueEnum<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn get_llvm_type(&self, our_type: &Type) -> BasicMetadataTypeEnum<'ctx> {
        let mut types = self.types.borrow_mut();
        if let Some(entry) = types.get(our_type) {
            return *entry;
        }

        let new_type = convert_our_type_to_llvm_basic_metadata_type(our_type, self);
        types.insert(our_type.clone(), new_type);
        new_type
    }

    pub fn enter_scope(&self) {
        self.scopes.borrow_mut().push(ScopeFrame::default());
    }

    pub fn exit_scope(&self) {
        self.scopes.borrow_mut().pop();
    }

    pub fn try_find_variable(&self, name: impl ToString) -> Option<BasicValueEnum<'ctx>> {
        let name = name.to_string();
        let scopes = self.scopes.borrow();

        scopes
            .iter()
            .rev()
            .find(|scope| scope.borrow().variables.contains_key(&name))
            .and_then(|scope| scope.borrow().variables.get(&name).cloned())
    }

    pub fn find_variable(&self, name: impl ToString) -> BasicValueEnum<'ctx> {
        self.try_find_variable(name).unwrap()
    }

    pub fn store_variable(&self, name: impl ToString, value: BasicValueEnum<'ctx>) {
        let name = name.to_string();

        let variables = self.scopes.borrow();

        variables.last().inspect(|scope| {
            scope.borrow_mut().variables.insert(name, value);
        });
    }

    pub fn try_find_function(&self, name: impl ToString) -> Option<FunctionValue<'ctx>> {
        let name = name.to_string();
        let scopes = self.scopes.borrow();

        scopes
            .iter()
            .rev()
            .find(|scope| scope.borrow().functions.contains_key(&name))
            .and_then(|scope| scope.borrow().functions.get(&name).cloned())
    }

    pub fn find_function(&self, name: impl ToString) -> FunctionValue<'ctx> {
        self.try_find_function(name).unwrap()
    }

    pub fn store_function(&self, name: impl ToString, value: FunctionValue<'ctx>) {
        let name = name.to_string();
        let fn_pointer = value.as_global_value().as_pointer_value();

        let scopes = self.scopes.borrow();

        scopes.last().inspect(|scope| {
            let mut scope_frame = scope.borrow_mut();
            scope_frame.functions.insert(name.clone(), value);
            scope_frame.variables.insert(name, fn_pointer.into());
        });
    }

    pub fn create_lambda(&self, return_type: &Type, param_types: &[Type]) -> FunctionValue<'ctx> {
        let llvm_lambda_type =
            build_llvm_function_type_from_own_types(self, return_type, param_types);

        let mut lambdas = self.lambdas.borrow_mut();

        let lambda_count = lambdas.len();
        let lambda_name = format!("lambda_{lambda_count}");

        let llvm_lambda_value = self
            .module
            .add_function(&lambda_name, llvm_lambda_type, None);

        lambdas.insert(lambda_name, llvm_lambda_value);

        llvm_lambda_value
    }
}
