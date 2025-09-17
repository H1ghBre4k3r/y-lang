//! # Code Generation Module
//!
//! This module implements LLVM-based code generation for the Y programming language.
//! It transforms the validated AST into executable LLVM IR that can be compiled to native code.
//!
//! ## Architecture Overview
//!
//! The code generation follows a visitor pattern where each AST node implements the `CodeGen` trait.
//! The central coordination is handled by the `CodegenContext` which maintains:
//!
//! - **LLVM Integration**: Direct interface to LLVM's context, module, and IR builder
//! - **Type Management**: Conversion between Y-lang types and LLVM types with caching
//! - **Scope Management**: Lexical scoping for variables, functions, and constants
//! - **Closure Support**: First-class support for lambda expressions with environment capture
//!
//! ## Closure System (Comprehensive)
//!
//! The closure subsystem provides a uniform, type–erased representation for all callable
//! entities (named functions and lambdas) enabling first‑class higher‑order usage without
//! proliferating distinct IR shapes.
//!
//! ### Lifecycle Overview
//!
//! ```text
//!  Source            Type Checking                 Codegen (Lambda)                      Storage                    Invocation
//!  ------            -------------                 ------------------                    -------                    ----------
//!  \(x) => x + y  →  capture analysis ----┐        build impl fn (env?, params...)       wrap as {fn*, env*}         extract {fn*, env*}
//!                     (y captured?)       │        allocate & populate env (if needed)   store in variable slot      bitcast fn*, pass env*, params
//!                                         └────►   construct closure struct              (functions lazily wrapped)  indirect call
//! ```
//!
//! ### Uniform Representation
//!
//! All closure-capable values use an LLVM struct `{ i8*, i8* }` (named functions may be kept as
//! raw pointers until wrapped at use sites):
//! - Field 0: Erased function pointer (bitcast from concrete `fn` pointer)
//! - Field 1: Environment pointer (heap block for captures, null for non‑capturing)
//!
//! This erasure avoids generating distinct nominal types per function signature and keeps
//! call dispatch logic simple (single extraction + generic pointer path).
//!
//! ### Environment Allocation Strategy
//!
//! For capturing lambdas we synthesise a dedicated heap struct whose field order matches the
//! capture list order provided by the type checker. Allocation currently uses a raw `malloc`
//! (through an LLVM intrinsic / external symbol expectation) and is never freed (basic leak
//! semantics acceptable for now while lifetime analysis is absent).
//!
//! Steps (capturing):
//! 1. Derive struct type of captured variable LLVM types
//! 2. Compute size via `size_of` (LLVM layout query) or implicit by GEP pattern
//! 3. Allocate heap block
//! 4. Bitcast to struct pointer
//! 5. Populate each field using `build_struct_gep` + `store`
//! 6. Cast environment pointer to `i8*` for storage in closure struct
//!
//! Non‑capturing: environment pointer is a constant null of type `i8*`.
//!
//! ### Invocation Protocol
//!
//! When a closure value is used in a call position:
//! 1. Extract function pointer (field 0) as a generic pointer
//! 2. Extract environment pointer (field 1)
//! 3. Assemble final argument list: `[env_ptr, user_args...]`
//! 4. Emit indirect call typed with the target `FunctionType` (see `expressions/postfix.rs`)
//!
//! For non‑capturing closures the `env_ptr` is null; implementation functions must tolerate
//! (and usually ignore) a null environment.
//!
//! ### Invariants
//! - Closure struct size and layout are fixed: exactly two pointer fields
//! - Function pointer field is always non‑null
//! - Environment pointer may be null (signals non‑capturing)
//! - Captured values are stored by value (no aliasing back to original stack slots)
//! - Environment memory is immutable post‑population (no mutation helpers yet)
//! - Every capturing lambda's implementation function expects its first parameter to be the environment pointer
//!
//! ### Edge Cases & Current Limitations
//! - No deallocation / lifetime tracking for environments (memory leak potential)
//! - Only by‑value capture (no by‑reference / by‑move / by‑mut semantics)
//! - No partial application support (call‑site arity mismatch not lowered to new closure)
//! - No small closure optimisation (SCO) for empty env; still stores null env pointer
//! - No environment mutation primitives (future feature)
//! - No recursion through captured self without explicit naming (standard lexical rule)
//! - No tail call optimisation marking for closure indirect calls yet
//! - Environment layout not deduplicated (identical shapes allocate distinct structs)
//! - Captured large aggregates always copied (no move elision)
//! - Indirect call sites lack inlining heuristics metadata (could add !prof in future)
//!
//! ### Future Evolution Hooks
//! - Reference counting / arena / epoch based reclamation for environments
//! - Small closure optimisation: encode empty env as tagged pointer or pointer+null sentinel elision
//! - Partial application: synthesise layered environments for fixed prefix arguments
//! - By‑reference / move / mutable capture kinds (introduce capture descriptor metadata table)
//! - Environment layout hashing + deduplication / interning
//! - Inline caching + speculative devirtualisation for monomorphic closure call sites
//! - Escape analysis to promote some environments to stack where safe
//! - Tail call optimisation hints for final position indirect calls
//! - Debug metadata emission (DWARF) for capture names & offsets
//! - Link‑time optimisation pass to fold identical impl fn bodies differing only in capture order
//!
//! ### Summary Table
//!
//! | Aspect              | Representation / Mechanism               | Location (Primary)                |
//! |---------------------|------------------------------------------|-----------------------------------|
//! | Closure value       | `{ i8*, i8* }` struct                     | `get_closure_struct_type`         |
//! | Impl fn signature   | `(i8* env, params...) -> ret`             | `create_closure_impl_fn_type`     |
//! | Construction        | insert erased fn*, env* into struct       | `build_closure_value`             |
//! | Extraction (fn)     | extract field 0 (generic ptr)             | `extract_closure_fn_ptr`          |
//! | Extraction (env)    | extract field 1                           | `extract_closure_env_ptr`         |
//! | Capture metadata    | stored from type checker                  | `store_lambda_captures`           |
//! | Capturing lambda    | heap env build + closure wrap             | `expressions/lambda.rs`           |
//! | Non‑capturing lambda| null env pointer + closure wrap           | `store_lambda` / lambda codegen   |
//! | Call dispatch       | decision + indirect call assembly         | `expressions/postfix.rs`          |
//! | Uniform storage     | functions inserted as variable fn ptr     | `store_function` (to be expanded) |
//!
//! ### ASCII Diagram: Memory & Call Flow
//!
//! ```text
//! +---------------------------+          +-------------------------------+
//! | Closure Value (on stack) |          | Environment (heap, optional)  |
//! | { fn_ptr_i8*, env_i8* }  |          | struct Captures {             |
//! |   0 ───────────────┐     |    env ─►|   field0: <captured var 0>    |
//! |   1 ───────┐       |     |          |   field1: <captured var 1>    |
//! +------------|-------+     |          +-------------------------------+
//!              |             |
//!              v (generic)   | call site extracts
//!        +---------------------------+
//!        | Impl fn (typed)          |
//!        | (i8* env, params...) -> R|
//!        +---------------------------+
//! ```
//!
//! ### Minimal IR Sketches
//!
//! Construction (non‑capturing):
//! ```llvm
//! %closure = insertvalue { i8*, i8* } undef, i8* bitcast (void (...)* @lambda_impl to i8*), 0
//! %closure1 = insertvalue { i8*, i8* } %closure, i8* null, 1
//! ```
//!
//! Indirect call (capturing):
//! ```llvm
//! %fn_i8  = extractvalue { i8*, i8* } %closure, 0
//! %env    = extractvalue { i8*, i8* } %closure, 1
//! %call   = call i64 %fn_typed(i8* %env, i64 %arg0) ; where %fn_typed : (i8*, i64) -> i64
//! ```
//!
//! Verifier/ABI invariants:
//! - env pointer is argument 0 for capturing impls
//! - env is null for non‑capturing closures
//! - closure layout is exactly two pointers
//! - named functions may be stored as raw pointers and wrapped lazily at use sites
//!
//! ### Rationale for i8* Erasure
//!
//! Using `i8*` (opaque pointer) fields permits storing any function/environment without
//! constructing a distinct struct per signature, minimising IR bloat and simplifying
//! equality / hashing potential for future optimisations.
//!
//! ## Key Design Decisions
//!
//! ### Function Representation
//! At the type level, function types are represented as closure structs `{i8*, i8*}` where:
//! - First field: Function pointer (cast to i8*)
//! - Second field: Environment pointer (i8*, null for non-capturing lambdas)
//!
//! This uniform representation allows seamless interoperability between regular functions
//! and lambdas. Named functions may be stored internally as raw function pointers and wrapped
//! lazily into closure structs when used in higher‑order contexts. See `store_function` and
//! `store_lambda` for storage differences and lazy wrapping behavior.
//!
//! ### Memory Management
//! - **Stack allocation**: Local variables, function parameters, temporary values
//! - **Heap allocation**: Lambda environments (captured variables)
//! - **Static allocation**: Global functions, constants
//!
//! ### Type System Integration
//! The codegen assumes input from a validated type checker and relies on type information
//! for safe LLVM IR generation. Invalid type information will result in panics.

pub mod expressions;
pub mod statements;

use std::{cell::RefCell, collections::HashMap};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    values::{BasicValueEnum, FunctionValue},
};

use crate::typechecker::{CaptureInfo, Type};

/// Central context for LLVM code generation.
///
/// This structure maintains all the state needed for translating Y-lang AST nodes
/// into LLVM IR. It serves as the coordination point between different code generation
/// phases and manages the interaction with LLVM APIs.
///
/// ## Lifetime Management
///
/// The `'ctx` lifetime parameter ties this context to the LLVM context lifetime,
/// ensuring that all generated LLVM values remain valid for the duration of
/// the code generation process.
///
/// ## Thread Safety
///
/// Uses `RefCell` for interior mutability since LLVM operations require mutable
/// access but the visitor pattern passes immutable references. This is safe because
/// code generation is single-threaded.
pub struct CodegenContext<'ctx> {
    /// LLVM context - provides the global state for LLVM operations
    pub context: &'ctx Context,

    /// LLVM module - container for functions, globals, and metadata
    pub module: Module<'ctx>,

    /// LLVM IR builder - generates instructions within basic blocks
    pub builder: Builder<'ctx>,

    /// Type cache mapping Y-lang types to LLVM types
    /// Avoids expensive type reconstruction and ensures type consistency
    pub types: RefCell<HashMap<Type, BasicMetadataTypeEnum<'ctx>>>,

    /// Lexical scope stack for variable/function resolution
    /// Each scope frame contains variables, functions, and constants in that scope
    pub scopes: RefCell<Vec<ScopeFrame<'ctx>>>,

    /// Counter for generating unique lambda function names
    /// Ensures each lambda gets a distinct identifier in the LLVM module
    pub lambda_counter: RefCell<usize>,

    /// Storage for lambda capture information
    /// Maps lambda IDs to their captured variable information for closure generation
    pub lambda_captures: RefCell<HashMap<String, CaptureInfo>>,
}

/// A single scope frame in the lexical scoping stack.
///
/// Uses `RefCell` to allow mutation during variable binding and lookup operations
/// while maintaining the immutable interface of the visitor pattern.
pub type ScopeFrame<'ctx> = RefCell<Scope<'ctx>>;

/// Represents a single lexical scope containing named bindings.
///
/// Each scope maintains separate namespaces for variables, functions, and constants.
/// This separation ensures that identifiers can be properly resolved according to
/// Y-lang's scoping rules.
///
/// ## Storage Strategy
///
/// - **Variables**: Stored as `BasicValueEnum` which can be pointers (for mutable variables)
///   or direct values (for immutable bindings)
/// - **Functions**: Stored as `FunctionValue` representing LLVM function declarations
/// - **Constants**: Stored as `BasicValueEnum` representing compile-time constant values
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Scope<'ctx> {
    /// Variables bound in this scope (both mutable and immutable)
    variables: HashMap<String, BasicValueEnum<'ctx>>,

    /// Functions declared or defined in this scope
    functions: HashMap<String, FunctionValue<'ctx>>,

    /// Compile-time constants defined in this scope
    constants: HashMap<String, BasicValueEnum<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    /// Converts a Y-lang type to its corresponding LLVM type with caching.
    ///
    /// This is a critical function that bridges the Y-lang type system with LLVM's
    /// type system. It implements a caching strategy to avoid repeated expensive
    /// type conversions and ensures type consistency throughout code generation.
    ///
    /// ## Caching Strategy
    ///
    /// Types are cached in the `types` HashMap to ensure:
    /// 1. **Performance**: Avoid reconstructing complex types repeatedly
    /// 2. **Consistency**: The same Y-lang type always maps to the same LLVM type
    /// 3. **Memory efficiency**: LLVM types are reused rather than duplicated
    ///
    /// ## Type Mapping
    ///
    /// - Primitive types: Direct mapping to LLVM primitives (i64, f64, i1, i8)
    /// - Complex types: Converted to LLVM structs, arrays, or pointers
    /// - Function types: Mapped to closure struct representation `{i8*, i8*}`
    ///
    /// # Parameters
    ///
    /// * `our_type` - The Y-lang type to convert
    ///
    /// # Returns
    ///
    /// The corresponding LLVM type as `BasicMetadataTypeEnum`
    ///
    /// # Panics
    ///
    /// Panics if the Y-lang type cannot be converted to an LLVM type (e.g., `Type::Void` or `Type::Unknown`)
    pub fn get_llvm_type(&self, our_type: &Type) -> BasicMetadataTypeEnum<'ctx> {
        // Check cache first for performance and consistency
        {
            let types = self.types.borrow();
            if let Some(entry) = types.get(our_type) {
                return *entry;
            }
        }

        // Convert and cache the new type
        let new_type = convert_our_type_to_llvm_basic_metadata_type(our_type, self);
        {
            let mut types = self.types.borrow_mut();
            types.insert(our_type.clone(), new_type);
        }
        new_type
    }

    /// Returns the canonical closure struct type used for all function values.
    ///
    /// In Y-lang, all function types are represented as closures with the structure `{i8*, i8*}`:
    /// - Field 0: Function pointer (cast to generic i8*)
    /// - Field 1: Environment pointer (i8*, null for non-capturing functions)
    ///
    /// This uniform representation enables:
    /// - **Polymorphism**: All functions use the same calling convention
    /// - **Closure support**: Capturing lambdas store their environment
    /// - **Interoperability**: Regular functions and lambdas are interchangeable
    ///
    /// ## Memory Layout
    ///
    /// The struct is packed as `{i8*, i8*}` which on most platforms is 16 bytes.
    /// The `false` parameter indicates this is not a packed struct, allowing
    /// LLVM to insert padding for optimal alignment.
    ///
    /// # Returns
    ///
    /// LLVM struct type representing `{i8*, i8*}`
    pub fn get_closure_struct_type(&self) -> inkwell::types::StructType<'ctx> {
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        self.context
            .struct_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false)
    }

    /// Creates the function type for closure implementation functions.
    ///
    /// Closure implementation functions have the signature `(i8* env, params...) -> ret`
    /// where the first parameter is always the environment pointer, followed by the
    /// user-defined parameters.
    ///
    /// This convention allows capturing lambdas to access their environment while
    /// maintaining a consistent calling interface. Non-capturing lambdas receive
    /// a null environment pointer.
    ///
    /// ## Parameter Layout
    ///
    /// 1. **Environment pointer**: `i8*` - points to captured variables struct
    /// 2. **User parameters**: Converted from Y-lang types to LLVM types
    ///
    /// ## Return Type Handling
    ///
    /// - `Type::Void`: Maps to LLVM void type
    /// - Other types: Converted to corresponding LLVM basic types
    /// - Unconvertible types: Fall back to void (should not happen with valid input)
    ///
    /// # Parameters
    ///
    /// * `return_type` - The Y-lang return type
    /// * `param_types` - Slice of Y-lang parameter types
    ///
    /// # Returns
    ///
    /// LLVM function type with environment parameter prepended
    pub fn create_closure_impl_fn_type(
        &self,
        return_type: &Type,
        param_types: &[Type],
    ) -> inkwell::types::FunctionType<'ctx> {
        // Environment pointer as first parameter for closure implementations
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let mut llvm_param_types = vec![i8_ptr_type.into()];

        // Convert and append user-defined parameters
        for param_type in param_types {
            llvm_param_types.push(self.get_llvm_type(param_type));
        }

        // Handle return type conversion
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
                    // Fallback to void for unconvertible types
                    let void_type = self.context.void_type();
                    void_type.fn_type(&llvm_param_types, false)
                }
            }
        }
    }

    /// Constructs a closure value from function and environment pointers.
    ///
    /// This function creates the standard closure representation used throughout
    /// Y-lang for all function values. The closure struct contains:
    /// 1. Function pointer (cast to generic i8*)
    /// 2. Environment pointer (i8*, null for non-capturing functions)
    ///
    /// ## Construction Process
    ///
    /// 1. **Initialize**: Start with undefined closure struct
    /// 2. **Cast function pointer**: Convert typed function pointer to generic i8*
    /// 3. **Insert function**: Place cast pointer in field 0
    /// 4. **Insert environment**: Place environment pointer in field 1
    ///
    /// The casting to i8* is necessary because LLVM function pointers have
    /// specific types, but we need a uniform representation for all functions.
    ///
    /// # Parameters
    ///
    /// * `fn_ptr` - LLVM function pointer to wrap
    /// * `env_ptr` - Environment pointer (may be null)
    ///
    /// # Returns
    ///
    /// Complete closure struct value ready for use
    pub fn build_closure_value(
        &self,
        fn_ptr: inkwell::values::PointerValue<'ctx>,
        env_ptr: inkwell::values::PointerValue<'ctx>,
    ) -> inkwell::values::StructValue<'ctx> {
        let closure_type = self.get_closure_struct_type();
        let closure_undef = closure_type.get_undef();

        // Cast function pointer to generic i8* for uniform storage
        let fn_ptr_as_i8 = self
            .builder
            .build_bit_cast(
                fn_ptr,
                self.context.ptr_type(inkwell::AddressSpace::default()),
                "fn_ptr_cast",
            )
            .unwrap()
            .into_pointer_value();

        // Insert function pointer into field 0
        let closure_with_fn = self
            .builder
            .build_insert_value(closure_undef, fn_ptr_as_i8, 0, "closure_with_fn")
            .unwrap()
            .into_struct_value();

        // Insert environment pointer into field 1
        self.builder
            .build_insert_value(closure_with_fn, env_ptr, 1, "closure_complete")
            .unwrap()
            .into_struct_value()
    }

    /// Extracts the function pointer from a closure value.
    ///
    /// This function retrieves the function pointer from field 0 of the closure struct.
    /// It returns a generic pointer; the concrete `FunctionType` is supplied at the
    /// indirect call site (`build_indirect_call`).

    /// ## Type Safety
    ///
    /// The `target_fn_type` parameter is used only to type the indirect call; we do not
    /// materialise a strongly-typed function pointer here. LLVM enforces signature
    /// compatibility at the call site.
    ///
    /// ## Usage Context
    ///
    /// This function is typically called before making an indirect function call.
    /// The returned generic pointer is paired with `target_fn_type` in `build_indirect_call`.
    ///
    /// # Parameters
    ///
    /// * `closure_value` - The closure struct to extract from
    /// * `target_fn_type` - The expected function type for the indirect call
    ///
    /// # Returns
    ///
    /// Function pointer as a generic pointer; typing is provided at the call site
    pub fn extract_closure_fn_ptr(
        &self,
        closure_value: inkwell::values::StructValue<'ctx>,
        target_fn_type: inkwell::types::FunctionType<'ctx>,
    ) -> inkwell::values::PointerValue<'ctx> {
        // Extract the generic function pointer from field 0
        let fn_ptr_i8 = self
            .builder
            .build_extract_value(closure_value, 0, "extract_fn_ptr")
            .unwrap()
            .into_pointer_value();

        // Cast to generic pointer type (LLVM will handle function type checking)
        let target_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        self.builder
            .build_bit_cast(fn_ptr_i8, target_ptr_type, "cast_fn_ptr")
            .unwrap()
            .into_pointer_value()
    }

    /// Extracts the environment pointer from a closure value.
    ///
    /// Retrieves the environment pointer from field 1 of the closure struct.
    /// This pointer may be:
    /// - **Null**: For non-capturing functions and lambdas
    /// - **Valid**: Points to heap-allocated environment struct for capturing lambdas
    ///
    /// ## Environment Pointer Usage
    ///
    /// The environment pointer is passed as the first argument to closure
    /// implementation functions, allowing them to access captured variables.
    /// The caller is responsible for checking if the pointer is null before
    /// dereferencing.
    ///
    /// # Parameters
    ///
    /// * `closure_value` - The closure struct to extract from
    ///
    /// # Returns
    ///
    /// Environment pointer (may be null)
    pub fn extract_closure_env_ptr(
        &self,
        closure_value: inkwell::values::StructValue<'ctx>,
    ) -> inkwell::values::PointerValue<'ctx> {
        self.builder
            .build_extract_value(closure_value, 1, "extract_env_ptr")
            .unwrap()
            .into_pointer_value()
    }

    /// Stores capture information for a lambda expression.
    ///
    /// Lambda capture information is computed during type checking and stored
    /// here for use during code generation. This information determines whether
    /// a lambda needs environment allocation and which variables to capture.
    ///
    /// # Parameters
    ///
    /// * `lambda_id` - Unique identifier for the lambda (typically based on source position)
    /// * `captures` - Information about captured variables and their types
    pub fn store_lambda_captures(&self, lambda_id: String, captures: CaptureInfo) {
        self.lambda_captures
            .borrow_mut()
            .insert(lambda_id, captures);
    }

    /// Retrieves capture information for a lambda expression.
    ///
    /// Returns the capture information stored during type checking, or `None`
    /// if no capture information is available (indicating a non-capturing lambda).
    ///
    /// # Parameters
    ///
    /// * `lambda_id` - Unique identifier for the lambda
    ///
    /// # Returns
    ///
    /// Capture information if available, `None` otherwise
    pub fn get_lambda_captures(&self, lambda_id: &str) -> Option<CaptureInfo> {
        self.lambda_captures.borrow().get(lambda_id).cloned()
    }

    /// Enters a new lexical scope.
    ///
    /// Creates a new scope frame and pushes it onto the scope stack.
    /// This should be called when entering any construct that creates
    /// a new scope (functions, blocks, if expressions, etc.).
    ///
    /// The new scope starts empty and inherits nothing from parent scopes,
    /// implementing Y-lang's lexical scoping rules where inner scopes can
    /// shadow outer scope bindings.
    pub fn enter_scope(&self) {
        self.scopes.borrow_mut().push(ScopeFrame::default());
    }

    /// Exits the current lexical scope.
    ///
    /// Removes the current scope frame from the scope stack, discarding
    /// all bindings created in that scope. This should be called when
    /// exiting any scope-creating construct.
    ///
    /// # Panics
    ///
    /// Will panic if called when there are no scopes to exit (scope stack underflow).
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

    /// Stores a regular (named) function in the current scope.
    ///
    /// Functions are registered in two related namespaces:
    /// - `functions`: enables later resolution for direct calls
    /// - `variables`: inserted as the raw function pointer for uniform higher‑order use
    ///
    /// ## Representation Parity
    /// Unlike lambdas, named functions are currently stored as raw pointers and *not*
    /// immediately wrapped into the `{ i8*, i8* }` closure struct. This is an intentional
    /// space/time trade‑off: we avoid constructing closure structs for functions that may
    /// never be used as values. When (and only when) such a pointer is placed in a context
    /// requiring a closure (e.g. passed to higher‑order API), it will be normalised.
    ///
    /// ## Future Normalisation
    /// A later optimisation may proactively canonicalise all function values to the closure
    /// struct form to enable representation hashing, small closure optimisation, or partial
    /// application. Until then, we maintain the lean pointer form internally.
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

    /// Stores a non‑capturing lambda in the current scope as a canonical closure value.
    ///
    /// ## Rationale
    /// Non‑capturing lambdas are already constructed in expression position and therefore
    /// have an immediate value context; wrapping them eagerly removes an extra branch in
    /// downstream call dispatch (the call site can always assume a closure struct).
    ///
    /// ## Parity Note
    /// Contrasts with `store_function` which defers wrapping to minimise overhead for
    /// functions never used as first‑class values.
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

/// Core trait for LLVM code generation.
///
/// This trait is implemented by all AST node types that can generate LLVM IR.
/// It follows the visitor pattern where each node knows how to generate code
/// for itself using the provided `CodegenContext`.
///
/// ## Design Principles
///
/// - **Immutable AST**: The `&self` parameter ensures AST nodes are not modified
/// - **Contextual generation**: All LLVM operations go through the shared context
/// - **Type safety**: Return types are specified to match the node's semantic meaning
///
/// ## Return Value Types
///
/// - **Expressions**: `Option<BasicValueEnum<'ctx>>` - may or may not produce values
/// - **Statements**: `()` - perform side effects but don't produce values
/// - **Specialized**: Some nodes have custom return types for their specific needs
pub trait CodeGen<'ctx> {
    /// The type of value this AST node produces during code generation
    type ReturnValue;

    /// Generates LLVM IR for this AST node.
    ///
    /// # Parameters
    ///
    /// * `ctx` - The code generation context containing LLVM state and scoping information
    ///
    /// # Returns
    ///
    /// The result of code generation, type-specific to the implementing node
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue;
}

/// Converts a Y-lang type to its corresponding LLVM type representation.
///
/// This function handles the mapping between Y-lang's type system and LLVM's type system.
/// It's called by `CodegenContext::get_llvm_type` for types not in the cache.
///
/// ## Type Mapping Strategy
///
/// - **Primitives**: Direct mapping to LLVM built-in types
/// - **Strings**: Represented as `i8*` (pointer to character data)
/// - **Arrays**: Represented as pointers to element type
/// - **Structs**: Converted to LLVM struct types with field layout
/// - **Functions**: All mapped to uniform closure struct `{i8*, i8*}`
/// - **Tuples**: Converted to LLVM struct types
///
/// ## Memory Layout Considerations
///
/// - Structs use natural alignment (not packed)
/// - Arrays are represented as pointers for dynamic sizing
/// - Function pointers are wrapped in closure structs for uniformity
///
/// # Parameters
///
/// * `our_type` - The Y-lang type to convert
/// * `ctx` - Code generation context for recursive type conversion
///
/// # Returns
///
/// Corresponding LLVM type as `BasicMetadataTypeEnum`
///
/// # Panics
///
/// - `Type::Void`: Cannot be used as BasicMetadataTypeEnum
/// - `Type::Unknown`: Cannot convert unknown types
/// - Failed conversions for complex types
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

/// Converts LLVM metadata type enum to basic type enum.
///
/// LLVM distinguishes between "metadata" types (which can include additional
/// type information) and "basic" types (which can be used for values).
/// This function converts between these representations when possible.
///
/// ## Usage Context
///
/// This is primarily used when creating LLVM instructions that require
/// `BasicTypeEnum` parameters (like `alloca`, `load`, `store`) but we have
/// `BasicMetadataTypeEnum` from our type conversion system.
///
/// ## Conversion Coverage
///
/// Handles all standard LLVM basic types:
/// - Arrays, floats, integers, pointers, structs, vectors
/// - Returns `None` for metadata-only types that can't be used as basic types
///
/// # Parameters
///
/// * `ty` - The metadata type enum to convert
///
/// # Returns
///
/// `Some(BasicTypeEnum)` if conversion is possible, `None` otherwise
fn convert_metadata_to_basic(ty: BasicMetadataTypeEnum) -> Option<BasicTypeEnum> {
    match ty {
        // Standard conversions for all basic LLVM types
        BasicMetadataTypeEnum::ArrayType(t) => Some(BasicTypeEnum::ArrayType(t)),
        BasicMetadataTypeEnum::FloatType(t) => Some(BasicTypeEnum::FloatType(t)),
        BasicMetadataTypeEnum::IntType(t) => Some(BasicTypeEnum::IntType(t)),
        BasicMetadataTypeEnum::PointerType(t) => Some(BasicTypeEnum::PointerType(t)),
        BasicMetadataTypeEnum::StructType(t) => Some(BasicTypeEnum::StructType(t)),
        BasicMetadataTypeEnum::VectorType(t) => Some(BasicTypeEnum::VectorType(t)),

        // Metadata-only types that cannot be converted to basic types
        _ => None,
    }
}
