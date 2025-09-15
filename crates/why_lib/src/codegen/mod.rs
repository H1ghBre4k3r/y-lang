mod expressions;
mod statements;

use std::{cell::RefCell, collections::HashMap};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType},
    values::{BasicValueEnum, FunctionValue},
};

use crate::typechecker::Type;

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub types: RefCell<HashMap<Type, BasicMetadataTypeEnum<'ctx>>>,
    pub scopes: RefCell<Vec<ScopeFrame<'ctx>>>,
    pub lambda_counter: RefCell<usize>,
    /// Cache of environment struct types, keyed by capture signature
    pub environment_types: RefCell<HashMap<Vec<(String, Type)>, StructType<'ctx>>>,
    /// Counter for generating unique environment type names
    pub environment_counter: RefCell<usize>,
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
            scope_frame.variables.insert(name, fn_pointer.into());
        });
    }

    pub fn store_lambda(&self, name: impl ToString, value: FunctionValue<'ctx>) {
        let name = name.to_string();
        let fn_pointer = value.as_global_value().as_pointer_value();

        let scopes = self.scopes.borrow();

        scopes.last().inspect(|scope| {
            let mut scope_frame = scope.borrow_mut();
            scope_frame.functions.insert(name.clone(), value);
            scope_frame.variables.insert(name, fn_pointer.into());
        });
    }

    /// Generate or retrieve an environment struct type for the given captures
    pub fn get_environment_type(&self, captures: &[(String, Type)]) -> StructType<'ctx> {
        // Check if we already have this environment type cached
        {
            let environment_types = self.environment_types.borrow();
            if let Some(existing_type) = environment_types.get(captures) {
                return *existing_type;
            }
        }

        // Generate a new environment type
        let field_types: Vec<BasicTypeEnum> = captures
            .iter()
            .map(|(_, capture_type)| {
                let llvm_type = self.get_llvm_type(capture_type);
                convert_metadata_to_basic(llvm_type)
                    .unwrap_or_else(|| panic!("Cannot convert capture type {:?} to basic type", capture_type))
            })
            .collect();

        // Generate unique name for the environment type
        let env_counter = {
            let mut counter = self.environment_counter.borrow_mut();
            let current = *counter;
            *counter += 1;
            current
        };

        let _env_type_name = format!("closure_env_{}", env_counter);
        let struct_type = self.context.struct_type(&field_types, false);

        // Cache the generated type
        {
            let mut environment_types = self.environment_types.borrow_mut();
            environment_types.insert(captures.to_vec(), struct_type);
        }

        struct_type
    }

    /// Create a closure struct type that pairs a function pointer with an environment pointer
    pub fn get_closure_struct_type(&self) -> StructType<'ctx> {
        // Closure struct: { function_ptr, environment_ptr }
        let function_ptr_type = self.context.ptr_type(Default::default());
        let environment_ptr_type = self.context.ptr_type(Default::default());

        self.context.struct_type(
            &[
                function_ptr_type.into(),
                environment_ptr_type.into(),
            ],
            false,
        )
    }

    /// Declare malloc function in LLVM module if not already declared
    pub fn declare_malloc(&self) -> FunctionValue<'ctx> {
        // Check if malloc is already declared
        if let Some(malloc_fn) = self.module.get_function("malloc") {
            return malloc_fn;
        }

        // Declare malloc: void* malloc(size_t size)
        let size_t_type = self.context.i64_type(); // size_t as i64
        let void_ptr_type = self.context.ptr_type(Default::default());
        let malloc_type = void_ptr_type.fn_type(&[size_t_type.into()], false);

        self.module.add_function("malloc", malloc_type, None)
    }

    /// Declare free function in LLVM module if not already declared
    pub fn declare_free(&self) -> FunctionValue<'ctx> {
        // Check if free is already declared
        if let Some(free_fn) = self.module.get_function("free") {
            return free_fn;
        }

        // Declare free: void free(void* ptr)
        let void_ptr_type = self.context.ptr_type(Default::default());
        let void_type = self.context.void_type();
        let free_type = void_type.fn_type(&[void_ptr_type.into()], false);

        self.module.add_function("free", free_type, None)
    }

    /// Allocate memory on the heap for a closure environment
    pub fn heap_allocate_environment(&self, env_type: StructType<'ctx>) -> BasicValueEnum<'ctx> {
        let malloc_fn = self.declare_malloc();

        // Calculate size of environment struct
        let env_size = env_type.size_of().unwrap();

        // Call malloc with the environment size
        let malloc_call = self.builder
            .build_call(malloc_fn, &[env_size.into()], "closure_env_malloc")
            .unwrap();

        let malloc_result = malloc_call.try_as_basic_value().left().unwrap();

        // Cast the void* result to our environment struct pointer type
        let env_ptr_type = env_type.ptr_type(Default::default());
        self.builder
            .build_pointer_cast(
                malloc_result.into_pointer_value(),
                env_ptr_type,
                "closure_env_ptr"
            )
            .unwrap()
            .into()
    }

    /// Free heap-allocated memory (for future memory management)
    pub fn heap_free(&self, ptr: BasicValueEnum<'ctx>) {
        let free_fn = self.declare_free();

        // Cast pointer to void* if needed
        let void_ptr = if ptr.is_pointer_value() {
            let void_ptr_type = self.context.ptr_type(Default::default());
            self.builder
                .build_pointer_cast(
                    ptr.into_pointer_value(),
                    void_ptr_type,
                    "void_ptr_cast"
                )
                .unwrap()
        } else {
            panic!("Expected pointer value for heap_free");
        };

        self.builder
            .build_call(free_fn, &[void_ptr.into()], "")
            .unwrap();
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
        // Function types are represented as function pointers
        Type::Function {
            params,
            return_value,
        } => {
            // Create function type and return pointer to it
            let llvm_param_types: Vec<_> = params
                .iter()
                .map(|param_type| ctx.get_llvm_type(param_type))
                .collect();

            match return_value.as_ref() {
                Type::Void => {
                    // TODO: is this correct? Why dont we use them?
                    let llvm_void_type = ctx.context.void_type();
                    let fn_type = llvm_void_type.fn_type(&llvm_param_types, false);
                    ctx.context.ptr_type(Default::default()).into()
                }
                return_type => {
                    let llvm_return_metadata_type = ctx.get_llvm_type(return_type);
                    if let Some(basic_return_type) =
                        convert_metadata_to_basic(llvm_return_metadata_type)
                    {
                        let _fn_type = basic_return_type.fn_type(&llvm_param_types, false);
                        ctx.context.ptr_type(Default::default()).into()
                    } else {
                        // Fallback to void function pointer if conversion fails
                        let llvm_void_type = ctx.context.void_type();
                        let _fn_type = llvm_void_type.fn_type(&llvm_param_types, false);
                        ctx.context.ptr_type(Default::default()).into()
                    }
                }
            }
        }
        Type::Closure {
            params,
            return_value,
            ..
        } => {
            // For closures, treat them like function pointers for now
            // TODO: Properly handle capture environments
            let llvm_param_types: Vec<_> = params
                .iter()
                .map(|param_type| ctx.get_llvm_type(param_type))
                .collect();
            match return_value.as_ref() {
                Type::Void => {
                    let llvm_void_type = ctx.context.void_type();
                    let fn_type = llvm_void_type.fn_type(&llvm_param_types, false);
                    ctx.context.ptr_type(Default::default()).into()
                }
                return_type => {
                    let llvm_return_metadata_type = ctx.get_llvm_type(return_type);
                    if let Some(basic_return_type) =
                        convert_metadata_to_basic(llvm_return_metadata_type)
                    {
                        let _fn_type = basic_return_type.fn_type(&llvm_param_types, false);
                        ctx.context.ptr_type(Default::default()).into()
                    } else {
                        // Fallback to void function pointer if conversion fails
                        let llvm_void_type = ctx.context.void_type();
                        let _fn_type = llvm_void_type.fn_type(&llvm_param_types, false);
                        ctx.context.ptr_type(Default::default()).into()
                    }
                }
            }
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
