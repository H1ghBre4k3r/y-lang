//! # Function Statement Code Generation
//!
//! This module implements LLVM code generation for function declarations in Y-lang.
//! It uses a two-pass approach to handle forward references and provides special
//! handling for the main function to ensure C runtime compatibility.
//!
//! ## Two-Pass Function Generation
//!
//! ### Pass 1: Declaration Registration
//! - Creates LLVM function declarations without bodies
//! - Registers functions in the symbol table for forward references
//! - Enables recursive functions and mutual recursion
//! - Handles main function renaming for wrapper generation
//!
//! ### Pass 2: Body Generation
//! - Generates function bodies using pre-registered declarations
//! - Sets up parameter bindings and function scopes
//! - Handles return value generation and terminator placement
//! - Creates main wrapper for void main functions
//!
//! ## Main Function Handling
//!
//! Y-lang's main function may return void, but C runtime expects `int main()`.
//! The solution:
//! - Rename Y-lang void main to `y_main`
//! - Generate C-compatible `main()` wrapper that calls `y_main()` and returns 0
//! - Non-void main functions use their original names
//!
//! ## Function Type Construction
//!
//! The module provides utilities for converting Y-lang function types to LLVM types:
//! - Parameter type conversion with proper LLVM mapping
//! - Return type handling including void and closure types
//! - Support for all Y-lang types including structs and arrays

use inkwell::types::{BasicMetadataTypeEnum, FunctionType};

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::{Function, FunctionParameter},
    typechecker::{Type, ValidatedTypeInformation},
};

impl Function<ValidatedTypeInformation> {
    /// Registers a function declaration in the LLVM module (Pass 1 of two-pass generation).
    ///
    /// This method creates an LLVM function declaration without generating the body,
    /// enabling forward references and recursive function calls. The function is
    /// registered in the symbol table for later body generation.
    ///
    /// ## Main Function Renaming
    ///
    /// Special handling for void main functions:
    /// - Y-lang `void main()` becomes LLVM `y_main()`
    /// - Enables generation of C-compatible `main()` wrapper later
    /// - Non-void main functions keep their original names
    ///
    /// ## Type System Integration
    ///
    /// Uses validated type information from the type checker to:
    /// - Extract function signature (parameters and return type)
    /// - Convert Y-lang types to LLVM function types
    /// - Ensure type consistency across compilation phases
    ///
    /// ## Symbol Table Management
    ///
    /// The function is stored in the current scope under its Y-lang name,
    /// even if the LLVM function has a different name (like y_main).
    /// This allows Y-lang code to reference it naturally.
    ///
    /// # Parameters
    ///
    /// * `ctx` - Code generation context for module and scope access
    pub fn register_declaration<'ctx>(&self, ctx: &CodegenContext<'ctx>) {
        let Function {
            id,
            info:
                ValidatedTypeInformation {
                    type_id:
                        Type::Function {
                            params,
                            return_value,
                        },
                    ..
                },
            ..
        } = self
        else {
            unreachable!()
        };

        // Apply main function renaming for C runtime compatibility
        let actual_fn_name = if id.name == "main" && **return_value == Type::Void {
            "y_main" // Rename to allow wrapper generation
        } else {
            id.name.as_str() // Use original name
        };

        // Convert Y-lang function signature to LLVM function type
        let llvm_fn_type = build_llvm_function_type_from_own_types(ctx, return_value, params);

        // Create LLVM function declaration and register in symbol table
        let llvm_fn_value = ctx.module.add_function(actual_fn_name, llvm_fn_type, None);
        ctx.store_function(&id.name, llvm_fn_value); // Store under Y-lang name
    }
}

impl<'ctx> CodeGen<'ctx> for Function<ValidatedTypeInformation> {
    type ReturnValue = ();

    /// Generates the function body (Pass 2 of two-pass generation).
    ///
    /// This method generates the actual function implementation, assuming the
    /// declaration was already registered in Pass 1. It handles parameter binding,
    /// body generation, return value processing, and main wrapper creation.
    ///
    /// ## Implementation Process
    ///
    /// 1. **Function Resolution**: Retrieve pre-registered function declaration
    /// 2. **Scope Setup**: Create function scope and bind parameters
    /// 3. **Basic Block Creation**: Set up entry block for function body
    /// 4. **Body Generation**: Generate IR for function body expression/block
    /// 5. **Return Handling**: Add appropriate return or unreachable terminators
    /// 6. **Main Wrapper**: Generate C-compatible main wrapper if needed
    /// 7. **Scope Cleanup**: Exit function scope and restore context
    ///
    /// ## Parameter Binding
    ///
    /// Function parameters are bound to their LLVM values in the function scope:
    /// - Parameters are accessible by name within the function body
    /// - Parameter indices match the order in the function signature
    /// - Type information ensures correct LLVM value retrieval
    ///
    /// ## Terminator Management
    ///
    /// Functions must have proper basic block terminators:
    /// - **Void functions**: `ret void` if no explicit return
    /// - **Value functions**: `ret <value>` if body produces value, `unreachable` otherwise
    /// - **Early returns**: Handled by checking existing terminators
    ///
    /// ## Main Wrapper Generation
    ///
    /// For void main functions, generates a C-compatible wrapper:
    /// ```llvm
    /// define i32 @main() {
    ///   call void @y_main()
    ///   ret i32 0
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// `()` - Function generation is a statement-level operation
    fn codegen(&self, ctx: &CodegenContext<'ctx>) {
        let Function {
            id,
            parameters,
            body,
            info:
                ValidatedTypeInformation {
                    type_id: Type::Function { return_value, .. },
                    ..
                },
            ..
        } = self
        else {
            unreachable!()
        };

        // Retrieve the pre-registered function from the scope
        let llvm_fn_value = ctx.resolve_function(&id.name);

        // Special handling for void main function wrapper
        let create_main_wrapper = id.name == "main" && **return_value == Type::Void;

        // enter scope for function parameters and local variables
        ctx.enter_scope();
        for (i, param) in parameters.iter().enumerate() {
            let FunctionParameter { name, .. } = param;

            let llvm_param_value = llvm_fn_value
                .get_nth_param(i as u32)
                .expect("There should be this parameter");

            ctx.store_variable(&name.name, llvm_param_value);
        }

        let llvm_fn_bb = ctx.context.append_basic_block(llvm_fn_value, "entry");
        ctx.builder.position_at_end(llvm_fn_bb);

        // Delegate to unified block code generation and capture yielded value
        let yielded_value = body.codegen(ctx);

        // Add terminator instruction if the basic block doesn't have one
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            match return_value.as_ref() {
                Type::Void => {
                    ctx.builder.build_return(None).unwrap();
                }
                _ => {
                    // For non-void functions, use the yielded value if available
                    if let Some(value) = yielded_value {
                        ctx.builder.build_return(Some(&value)).unwrap();
                    } else {
                        // Only add unreachable if there's truly no value to return
                        ctx.builder.build_unreachable().unwrap();
                    }
                }
            }
        }

        // Create main wrapper if needed
        if create_main_wrapper {
            let main_fn_type = ctx.context.i32_type().fn_type(&[], false);
            let main_fn = ctx.module.add_function("main", main_fn_type, None);
            let main_bb = ctx.context.append_basic_block(main_fn, "entry");

            // Store current builder position
            let current_bb = ctx.builder.get_insert_block();

            // Build the wrapper
            ctx.builder.position_at_end(main_bb);
            ctx.builder.build_call(llvm_fn_value, &[], "").unwrap();
            ctx.builder
                .build_return(Some(&ctx.context.i32_type().const_int(0, false)))
                .unwrap();

            // Restore builder position if it existed
            if let Some(bb) = current_bb {
                ctx.builder.position_at_end(bb);
            }
        }

        ctx.exit_scope();
    }
}

/// Builds an LLVM function type from Y-lang types.
///
/// This function converts Y-lang function signatures to LLVM function types,
/// handling the mapping between Y-lang's type system and LLVM's representation.
/// It provides special handling for various return types and ensures proper
/// parameter type conversion.
///
/// ## Type Conversion Strategy
///
/// ### Return Types
/// - **Primitives**: Direct mapping to LLVM types (bool->i1, char->i8, etc.)
/// - **String**: Mapped to pointer type (i8*)
/// - **Void**: Mapped to LLVM void type
/// - **Function**: Returns closure struct for first-class functions
/// - **Complex types**: Delegated to general type conversion system
///
/// ### Parameter Types
/// All parameter types are converted using the general type conversion
/// system for consistency with the rest of the codebase.
///
/// ## Closure Integration
///
/// When a function returns another function, it returns a closure struct
/// `{i8*, i8*}` to maintain Y-lang's first-class function semantics.
///
/// # Parameters
///
/// * `ctx` - Code generation context for type conversion
/// * `return_type` - Y-lang return type to convert
/// * `param_types` - Slice of Y-lang parameter types
///
/// # Returns
///
/// LLVM function type ready for function declaration/definition
pub fn build_llvm_function_type_from_own_types<'ctx>(
    ctx: &CodegenContext<'ctx>,
    return_type: &Type,
    param_types: &[Type],
) -> FunctionType<'ctx> {
    let llvm_param_types = param_types
        .iter()
        .map(|param_type| ctx.get_llvm_type(param_type))
        .collect::<Vec<_>>();

    match return_type {
        Type::Boolean => {
            let llvm_bool_type = ctx.context.bool_type();
            llvm_bool_type.fn_type(&llvm_param_types, false)
        }
        Type::Character => {
            let llvm_char_type = ctx.context.i8_type();
            llvm_char_type.fn_type(&llvm_param_types, false)
        }
        Type::String => {
            // String is represented as a pointer to i8
            let llvm_string_type = ctx.context.ptr_type(Default::default());
            llvm_string_type.fn_type(&llvm_param_types, false)
        }
        Type::Void => {
            let llvm_void_type = ctx.context.void_type();
            llvm_void_type.fn_type(&llvm_param_types, false)
        }
        Type::Unknown => todo!(),
        Type::Function {
            params: _fn_params,
            return_value: _fn_return_value,
        } => {
            // Function returning another function - return closure struct
            let closure_struct_type = ctx.get_closure_struct_type();
            closure_struct_type.fn_type(&llvm_param_types, false)
        }
        return_type => {
            let llvm_return_type = ctx.get_llvm_type(return_type);

            build_llvm_function_type_from_llvm_types(&llvm_return_type, &llvm_param_types)
        }
    }
}

/// Builds an LLVM function type from LLVM types.
///
/// This is a helper function that creates LLVM function types when both
/// return type and parameter types are already in LLVM representation.
/// It handles the dispatch to the appropriate LLVM type constructor.
///
/// ## LLVM Type Dispatch
///
/// Each LLVM type has its own `fn_type` method for creating function types.
/// This function dispatches to the appropriate method based on the return type.
///
/// ## Type Coverage
///
/// Handles all LLVM metadata types including:
/// - Basic types: Arrays, floats, integers, pointers, structs, vectors
/// - Advanced types: Scalable vectors, metadata types
///
/// # Parameters
///
/// * `llvm_type` - LLVM return type for the function
/// * `llvm_params` - LLVM parameter types
///
/// # Returns
///
/// LLVM function type with the specified signature
fn build_llvm_function_type_from_llvm_types<'ctx>(
    llvm_type: &BasicMetadataTypeEnum<'ctx>,
    llvm_params: &[BasicMetadataTypeEnum<'ctx>],
) -> FunctionType<'ctx> {
    match llvm_type {
        BasicMetadataTypeEnum::ArrayType(array_type) => array_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::FloatType(float_type) => float_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::IntType(int_type) => int_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::PointerType(pointer_type) => {
            pointer_type.fn_type(llvm_params, false)
        }
        BasicMetadataTypeEnum::StructType(struct_type) => struct_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::VectorType(vector_type) => vector_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::ScalableVectorType(scalable_vector_type) => {
            scalable_vector_type.fn_type(llvm_params, false)
        }
        BasicMetadataTypeEnum::MetadataType(metadata_type) => {
            metadata_type.fn_type(llvm_params, false)
        }
    }
}
