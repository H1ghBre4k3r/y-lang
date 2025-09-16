mod expressions;
mod statements;

use std::{cell::RefCell, collections::HashMap};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    values::{BasicValueEnum, FunctionValue},
};

use crate::typechecker::{CaptureInfo, Type};

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub types: RefCell<HashMap<Type, BasicMetadataTypeEnum<'ctx>>>,
    pub scopes: RefCell<Vec<ScopeFrame<'ctx>>>,
    pub lambda_counter: RefCell<usize>,
    pub lambda_captures: RefCell<HashMap<String, CaptureInfo>>,
}

pub type ScopeFrame<'ctx> = RefCell<Scope<'ctx>>;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Scope<'ctx> {
    variables: HashMap<String, BasicValueEnum<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    constants: HashMap<String, BasicValueEnum<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn get_llvm_type(&self, our_type: &Type) -> BasicMetadataTypeEnum<'ctx> {
        {
            let types = self.types.borrow();
            if let Some(entry) = types.get(our_type) {
                return *entry;
            }
        }
        let new_type = convert_our_type_to_llvm_basic_metadata_type(our_type, self);
        {
            let mut types = self.types.borrow_mut();
            types.insert(our_type.clone(), new_type);
        }
        new_type
    }

    /// Get the canonical closure struct type {i8*, i8*}
    pub fn get_closure_struct_type(&self) -> inkwell::types::StructType<'ctx> {
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        self.context
            .struct_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false)
    }

    /// Create a closure-impl function type (i8*, params...) -> ret
    pub fn create_closure_impl_fn_type(
        &self,
        return_type: &Type,
        param_types: &[Type],
    ) -> inkwell::types::FunctionType<'ctx> {
        // Environment pointer as first parameter
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let mut llvm_param_types = vec![i8_ptr_type.into()];

        // Add user parameters
        for param_type in param_types {
            llvm_param_types.push(self.get_llvm_type(param_type));
        }

        match return_type {
            Type::Void => {
                let void_type = self.context.void_type();
                void_type.fn_type(&llvm_param_types, false)
            }
            _ => {
                let return_metadata = self.get_llvm_type(return_type);
                if let Some(basic_return_type) = convert_metadata_to_basic(return_metadata) {
                    basic_return_type.fn_type(&llvm_param_types, false)
                } else {
                    // Fallback to void
                    let void_type = self.context.void_type();
                    void_type.fn_type(&llvm_param_types, false)
                }
            }
        }
    }

    /// Construct a closure value from function pointer and environment pointer
    pub fn build_closure_value(
        &self,
        fn_ptr: inkwell::values::PointerValue<'ctx>,
        env_ptr: inkwell::values::PointerValue<'ctx>,
    ) -> inkwell::values::StructValue<'ctx> {
        let closure_type = self.get_closure_struct_type();
        let closure_undef = closure_type.get_undef();

        // Insert function pointer (cast to i8*)
        let fn_ptr_as_i8 = self
            .builder
            .build_bit_cast(
                fn_ptr,
                self.context.ptr_type(inkwell::AddressSpace::default()),
                "fn_ptr_cast",
            )
            .unwrap()
            .into_pointer_value();

        let closure_with_fn = self
            .builder
            .build_insert_value(closure_undef, fn_ptr_as_i8, 0, "closure_with_fn")
            .unwrap()
            .into_struct_value();

        // Insert environment pointer
        self.builder
            .build_insert_value(closure_with_fn, env_ptr, 1, "closure_complete")
            .unwrap()
            .into_struct_value()
    }

    /// Extract function pointer from closure value and cast to target type
    pub fn extract_closure_fn_ptr(
        &self,
        closure_value: inkwell::values::StructValue<'ctx>,
        target_fn_type: inkwell::types::FunctionType<'ctx>,
    ) -> inkwell::values::PointerValue<'ctx> {
        let fn_ptr_i8 = self
            .builder
            .build_extract_value(closure_value, 0, "extract_fn_ptr")
            .unwrap()
            .into_pointer_value();

        let target_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        self.builder
            .build_bit_cast(fn_ptr_i8, target_ptr_type, "cast_fn_ptr")
            .unwrap()
            .into_pointer_value()
    }

    /// Extract environment pointer from closure value
    pub fn extract_closure_env_ptr(
        &self,
        closure_value: inkwell::values::StructValue<'ctx>,
    ) -> inkwell::values::PointerValue<'ctx> {
        self.builder
            .build_extract_value(closure_value, 1, "extract_env_ptr")
            .unwrap()
            .into_pointer_value()
    }

    /// Store capture information for a lambda
    pub fn store_lambda_captures(&self, lambda_id: String, captures: CaptureInfo) {
        self.lambda_captures
            .borrow_mut()
            .insert(lambda_id, captures);
    }

    /// Retrieve capture information for a lambda
    pub fn get_lambda_captures(&self, lambda_id: &str) -> Option<CaptureInfo> {
        self.lambda_captures.borrow().get(lambda_id).cloned()
    }

    pub fn enter_scope(&self) {
        self.scopes.borrow_mut().push(ScopeFrame::default());
    }

    pub fn exit_scope(&self) {
        self.scopes.borrow_mut().pop();
    }

    pub fn find_variable(&self, name: impl ToString) -> BasicValueEnum<'ctx> {
        let name = name.to_string();
        let scopes = self.scopes.borrow();

        scopes
            .iter()
            .rev()
            .find(|scope| scope.borrow().variables.contains_key(&name))
            .and_then(|scope| scope.borrow().variables.get(&name).cloned())
            .unwrap_or_else(|| panic!("epected variable '{name}' to be defined"))
    }

    pub fn resolve_function(&self, name: impl ToString) -> FunctionValue<'ctx> {
        let name = name.to_string();
        let scopes = self.scopes.borrow();

        scopes
            .iter()
            .rev()
            .find(|scope| scope.borrow().functions.contains_key(&name))
            .and_then(|scope| scope.borrow().functions.get(&name).cloned())
            .unwrap_or_else(|| panic!("expected function '{name}' to be defined"))
    }

    pub fn store_variable(&self, name: impl ToString, value: BasicValueEnum<'ctx>) {
        let name = name.to_string();

        let variables = self.scopes.borrow();

        variables.last().inspect(|scope| {
            scope.borrow_mut().variables.insert(name, value);
        });
    }

    pub fn store_constant(&self, name: impl ToString, value: BasicValueEnum<'ctx>) {
        let name = name.to_string();

        let scopes = self.scopes.borrow();

        scopes.last().inspect(|scope| {
            scope.borrow_mut().constants.insert(name, value);
        });
    }

    pub fn find_constant(&self, name: impl ToString) -> Option<BasicValueEnum<'ctx>> {
        let name = name.to_string();
        let scopes = self.scopes.borrow();

        scopes
            .iter()
            .rev()
            .find(|scope| scope.borrow().constants.contains_key(&name))
            .and_then(|scope| scope.borrow().constants.get(&name).cloned())
    }

    pub fn find_function(&self, name: impl ToString) -> FunctionValue<'ctx> {
        let name = name.to_string();
        let scopes = self.scopes.borrow();

        scopes
            .iter()
            .rev()
            .find(|scope| scope.borrow().functions.contains_key(&name))
            .and_then(|scope| scope.borrow().functions.get(&name).cloned())
            .unwrap()
    }

    pub fn store_function(&self, name: impl ToString, value: FunctionValue<'ctx>) {
        let name = name.to_string();
        let fn_pointer = value.as_global_value().as_pointer_value();

        let scopes = self.scopes.borrow();

        scopes.last().inspect(|scope| {
            let mut scope_frame = scope.borrow_mut();
            scope_frame.functions.insert(name.clone(), value);
            // Store function pointer directly for now - we'll wrap it as closure when used
            scope_frame.variables.insert(name, fn_pointer.into());
        });
    }

    pub fn store_lambda(&self, name: impl ToString, value: FunctionValue<'ctx>) {
        let name = name.to_string();
        let fn_pointer = value.as_global_value().as_pointer_value();

        // Create closure struct with env = null for non-capturing lambdas
        let null_env = self
            .context
            .ptr_type(inkwell::AddressSpace::default())
            .const_null();
        let closure_struct = self.build_closure_value(fn_pointer, null_env);

        let scopes = self.scopes.borrow();

        scopes.last().inspect(|scope| {
            let mut scope_frame = scope.borrow_mut();
            scope_frame.functions.insert(name.clone(), value);
            scope_frame.variables.insert(name, closure_struct.into());
        });
    }
}

pub trait CodeGen<'ctx> {
    type ReturnValue;
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue;
}

fn convert_our_type_to_llvm_basic_metadata_type<'ctx>(
    our_type: &Type,
    ctx: &CodegenContext<'ctx>,
) -> BasicMetadataTypeEnum<'ctx> {
    match our_type {
        Type::Integer => ctx.context.i64_type().into(),
        Type::FloatingPoint => ctx.context.f64_type().into(),
        Type::Boolean => ctx.context.bool_type().into(),
        Type::Character => ctx.context.i8_type().into(), // UTF-8 char representation
        Type::String => {
            // Represent strings as pointer to i8 (C-style strings or slices)
            ctx.context.ptr_type(Default::default()).into()
        }
        Type::Void => {
            // Void isn't a valid BasicMetadataTypeEnum — can return pointer or dummy
            panic!("Void cannot be used as a BasicMetadataTypeEnum")
        }
        Type::Unknown => {
            panic!("Cannot convert unknown type to LLVM")
        }
        Type::Reference(_) => ctx.context.ptr_type(Default::default()).into(),
        Type::Tuple(items) => {
            let types: Vec<_> = items
                .iter()
                .map(|item_type| {
                    // TODO: what about functions?
                    convert_metadata_to_basic(ctx.get_llvm_type(item_type)).unwrap_or_else(|| {
                        panic!("{item_type:?} can not be converted to a tuple item")
                    })
                })
                .collect();
            let struct_type = ctx.context.struct_type(&types, false);
            struct_type.into()
        }
        Type::Array(element_type) => {
            // TODO: do we actually need this?
            let element_llvm_type = ctx.get_llvm_type(element_type);
            let element_basic_type = convert_metadata_to_basic(element_llvm_type)
                .expect("Array element type must be basic");

            // For now, we'll represent arrays as pointers to their element type
            // This matches how we handle them in codegen (stack-allocated arrays)
            ctx.context.ptr_type(Default::default()).into()
        }
        Type::Struct(_, fields) => {
            let llvm_fields: Vec<_> = fields
                .iter()
                .map(|(_, field_type)| {
                    // TODO: what about functions?
                    convert_metadata_to_basic(ctx.get_llvm_type(field_type)).unwrap_or_else(|| {
                        panic!("{field_type:?} can not be converted to a struct field")
                    })
                })
                .collect();
            let struct_type = ctx.context.struct_type(&llvm_fields, false);
            struct_type.into()
        }
        // Function types are now represented as closure structs {i8*, i8*}
        Type::Function {
            params: _,
            return_value: _,
        } => {
            // All function types use the same closure struct representation
            let closure_struct_type = ctx.get_closure_struct_type();
            closure_struct_type.into()
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
