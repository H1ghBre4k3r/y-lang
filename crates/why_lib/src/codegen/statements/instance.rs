//! # Instance Method Code Generation
//!
//! This module implements LLVM code generation for instance methods in Y-lang.
//! Instance methods are functions associated with specific types that receive
//! an implicit `this` parameter pointing to the instance being operated on.
//!
//! ## Instance Method System
//!
//! Y-lang supports object-oriented programming through instance methods:
//! ```y-lang
//! instance Point {
//!     func distance_from_origin(this) -> float {
//!         sqrt(this.x * this.x + this.y * this.y)
//!     }
//! }
//! ```
//!
//! ## Method Compilation Strategy
//!
//! ### Two-Pass Compilation
//! Instance methods use the same two-pass approach as regular functions:
//! 1. **Declaration Pass**: Register method signatures for forward references
//! 2. **Implementation Pass**: Generate method bodies with `this` parameter injection
//!
//! ### Name Mangling
//! Instance methods are compiled as regular LLVM functions with mangled names:
//! - **Original**: `Point.distance_from_origin`
//! - **Mangled**: `Point_distance_from_origin`
//! - **Reason**: LLVM doesn't support namespaces, so mangling avoids conflicts
//!
//! ## This Parameter Injection
//!
//! ### Implicit This Parameter
//! All instance methods receive an implicit `this` parameter:
//! - **Type**: Reference to the instance type (`&StructType`)
//! - **Position**: First parameter in the LLVM function signature
//! - **Access**: Available as `this` variable within method body
//!
//! ### Parameter Reordering
//! The compiler automatically adjusts parameter lists:
//! - **Y-lang signature**: `func method(param1: int) -> string`
//! - **LLVM signature**: `@Type_method(ptr %this, i32 %param1) -> ptr`
//! - **User parameters**: Shifted to positions 1, 2, 3, etc.
//!
//! ## Type System Integration
//!
//! ### Struct Type Lookup
//! Methods must find their associated struct type:
//! - **Type registry**: Search through registered types in the compilation context
//! - **Name matching**: Match instance block type name with struct definitions
//! - **Error handling**: Panic if struct type is not found (type system error)
//!
//! ### Reference Types
//! The `this` parameter uses Y-lang's reference type system:
//! - **Memory safety**: References ensure valid instance access
//! - **LLVM representation**: Compiled as pointers for efficient access
//! - **Lifetime management**: Caller ensures instance lifetime exceeds method call
//!
//! ## Method Registration
//!
//! ### Symbol Table Integration
//! Methods are registered in the symbol table using:
//! - **Mangled names**: For LLVM function identification
//! - **Original names**: For recursive method calls within the method
//! - **Scope isolation**: Each method has its own parameter scope
//!
//! ### Forward References
//! The declaration pass enables:
//! - **Recursive methods**: Methods can call themselves
//! - **Mutual recursion**: Methods can call other methods on the same type
//! - **Cross-references**: Methods can call other instance methods
//!
//! ## LLVM Function Generation
//!
//! ### Function Creation
//! Each method becomes an LLVM function with:
//! - **Mangled name**: Unique identifier in the LLVM module
//! - **Modified signature**: Including implicit `this` parameter
//! - **Standard linkage**: Internal linkage for optimization
//!
//! ### Basic Block Management
//! Methods use standard function compilation:
//! - **Entry block**: Single entry point for method execution
//! - **Return handling**: Void and value-returning methods
//! - **Terminator management**: Ensures all code paths are properly terminated

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::{FunctionParameter, Instance, TypeName},
    typechecker::{Type, ValidatedTypeInformation},
};

use super::function::build_llvm_function_type_from_own_types;

impl Instance<ValidatedTypeInformation> {
    /// Registers method declarations for forward reference support (Pass 1 of two-pass compilation).
    ///
    /// This function implements the first pass of instance method compilation,
    /// registering method signatures without generating implementation code.
    /// This enables forward references, recursive methods, and mutual recursion
    /// between methods within the same instance block.
    ///
    /// ## Two-Pass Compilation Rationale
    ///
    /// Instance methods require two-pass compilation for the same reasons as regular functions:
    /// - **Forward references**: Methods can call other methods defined later in the block
    /// - **Recursive calls**: Methods can call themselves recursively
    /// - **Mutual recursion**: Methods can call each other in complex patterns
    /// - **Type checking**: All method signatures must be known for type validation
    ///
    /// ## Declaration Registration Process
    ///
    /// For each method in the instance block:
    /// 1. **Extract signature**: Get method name, parameters, and return type
    /// 2. **Register declaration**: Create LLVM function declaration without body
    /// 3. **Symbol table entry**: Add method to symbol table for lookup
    /// 4. **Type information**: Preserve type data for implementation pass
    ///
    /// ## Method Signature Handling
    ///
    /// The registration process delegates to the standard function declaration system:
    /// - **Reuse infrastructure**: Uses existing function declaration mechanisms
    /// - **Name mangling**: Method names will be mangled during declaration registration
    /// - **Parameter handling**: Standard parameter type conversion applies
    /// - **Return types**: Standard return type conversion applies
    ///
    /// ## Symbol Table Integration
    ///
    /// Registered methods become available for:
    /// - **Method calls**: Other methods can call registered methods
    /// - **Recursive calls**: Methods can call themselves
    /// - **Type checking**: Type checker can validate method call types
    /// - **Code generation**: Implementation pass can generate call instructions
    ///
    /// # Parameters
    ///
    /// * `ctx` - Code generation context containing LLVM module and symbol tables
    ///
    /// # Implementation Note
    ///
    /// Currently delegates to the standard function declaration system. Future
    /// enhancements might include instance-specific declaration handling for:
    /// - Method visibility controls
    /// - Instance-specific name mangling schemes
    /// - Method overloading support
    /// - Virtual method table generation
    pub fn register_declarations<'ctx>(&self, ctx: &CodegenContext<'ctx>) {
        // Extract methods from the instance block for declaration registration
        let Instance { functions, .. } = self;

        // Register each method's declaration using the standard function declaration system
        // This enables forward references and recursive method calls
        for function in functions {
            function.register_declaration(ctx);
        }
    }
}

impl<'ctx> CodeGen<'ctx> for Instance<ValidatedTypeInformation> {
    type ReturnValue = ();

    /// Generates LLVM IR for instance method implementations (Pass 2 of two-pass compilation).
    ///
    /// This function implements the second pass of instance method compilation,
    /// generating the actual method implementations with proper `this` parameter
    /// injection and name mangling. It processes all methods within an instance
    /// block and creates corresponding LLVM functions.
    ///
    /// ## Implementation Strategy
    ///
    /// The method implementation process follows these steps:
    /// 1. **Type name extraction**: Get the struct type name from the instance declaration
    /// 2. **Method processing**: Generate each method with `this` parameter injection
    /// 3. **Name mangling**: Create unique LLVM function names to avoid conflicts
    /// 4. **Declaration handling**: Process external method declarations if present
    ///
    /// ## Method Compilation Process
    ///
    /// For each method in the instance block:
    /// - **Name mangling**: `TypeName_methodName` to ensure uniqueness
    /// - **Parameter injection**: Add implicit `this` parameter as first argument
    /// - **Function generation**: Create LLVM function with modified signature
    /// - **Body compilation**: Generate method body with access to `this`
    ///
    /// ## This Parameter System
    ///
    /// ### Automatic Injection
    /// All instance methods automatically receive a `this` parameter:
    /// - **Type**: Reference to the instance type (`&StructType`)
    /// - **Position**: First parameter in the generated LLVM function
    /// - **Availability**: Accessible as `this` variable within method body
    /// - **Usage**: Enables access to instance fields and method calls
    ///
    /// ### Type Safety
    /// The `this` parameter maintains type safety through:
    /// - **Type validation**: Ensures `this` points to correct struct type
    /// - **Reference semantics**: Uses Y-lang's reference type system
    /// - **Lifetime management**: Caller guarantees instance lifetime
    ///
    /// ## Name Mangling Strategy
    ///
    /// ### Conflict Avoidance
    /// Name mangling prevents conflicts by:
    /// - **Type prefixing**: Each method name includes the type name
    /// - **Separator convention**: Uses underscore to separate type and method
    /// - **Uniqueness guarantee**: No two methods can have the same mangled name
    ///
    /// ### LLVM Integration
    /// Mangled names work well with LLVM because:
    /// - **Valid identifiers**: Follow LLVM naming conventions
    /// - **Debugger friendly**: Preserve enough information for debugging
    /// - **Optimization compatible**: Don't interfere with LLVM optimizations
    ///
    /// ## Error Handling
    ///
    /// ### Type Validation
    /// The function validates instance block constraints:
    /// - **Literal types only**: Instance blocks must target literal type names
    /// - **Type existence**: Referenced struct types must exist in type system
    /// - **Panic on failure**: Invalid configurations cause compilation to abort
    ///
    /// ### Declaration Handling
    /// External method declarations are currently placeholder-handled:
    /// - **Future feature**: Reserved for external method integration
    /// - **Current behavior**: Acknowledged but not processed
    /// - **Extension point**: Ready for future implementation
    ///
    /// # Returns
    ///
    /// `()` - Instance method generation is a statement-level operation
    ///
    /// # Panics
    ///
    /// - **Complex type names**: When instance block targets non-literal types
    /// - **Missing struct types**: When referenced struct type doesn't exist
    /// - **Method compilation failures**: When individual method compilation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Y-lang instance block:
    /// instance Point {
    ///     func distance_to(other: Point) -> float { ... }
    /// }
    ///
    /// // Generated LLVM function:
    /// define double @Point_distance_to(ptr %this, %Point %other) {
    ///   ; method body with access to %this
    /// }
    /// ```
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let Instance {
            name,
            functions,
            declarations,
            ..
        } = self;

        // Extract the struct type name that this instance block targets
        // Instance blocks can only target literal type names for simplicity
        let type_name = match name {
            TypeName::Literal(name, _) => name,
            _ => {
                panic!(
                    "Instance blocks can only be applied to literal type names, not complex types"
                );
            }
        };

        // Process each method implementation in the instance block
        for function in functions {
            // Generate mangled name to avoid conflicts with other functions
            // Pattern: TypeName_methodName ensures uniqueness across the module
            let method_name = format!("{}_{}", type_name, function.id.name);

            // Compile the method with automatic 'this' parameter injection
            self.compile_instance_method(ctx, function, type_name, &method_name);
        }

        // Process external method declarations (currently placeholder)
        // These represent methods declared elsewhere but associated with this type
        for _declaration in declarations {
            // TODO: Implement external method declaration handling
            // External methods would be registered without implementation
            // This supports interfaces, abstract methods, or external linking
        }
    }
}

impl<'ctx> Instance<ValidatedTypeInformation> {
    /// Compiles a single instance method with `this` parameter injection.
    ///
    /// This function handles the complex process of transforming a Y-lang method
    /// definition into an LLVM function with an implicit `this` parameter. It
    /// manages parameter reordering, type conversion, scope management, and
    /// method body compilation.
    ///
    /// ## Method Transformation Process
    ///
    /// The compilation involves several key transformations:
    /// 1. **Signature modification**: Add `this` parameter as first argument
    /// 2. **Parameter reordering**: Shift user parameters to positions 1, 2, 3...
    /// 3. **Function creation**: Generate LLVM function with modified signature
    /// 4. **Scope management**: Create method scope with proper parameter bindings
    /// 5. **Body compilation**: Generate method implementation with `this` access
    ///
    /// ## This Parameter Implementation
    ///
    /// ### Type Construction
    /// The `this` parameter is constructed as:
    /// - **Base type**: The struct type being implemented on
    /// - **Reference wrapper**: Wrapped in Y-lang's reference type system
    /// - **LLVM representation**: Compiled as a pointer for efficient access
    ///
    /// ### Parameter Injection
    /// The `this` parameter becomes the first LLVM function parameter:
    /// - **Position 0**: Reserved for `this` in all instance methods
    /// - **Name binding**: Available as `this` variable within method body
    /// - **Type safety**: Maintains type information for field access
    ///
    /// ## Parameter Management
    ///
    /// ### Parameter List Modification
    /// User-defined parameters are shifted to accommodate `this`:
    /// - **Original positions**: User parameters at 0, 1, 2...
    /// - **Modified positions**: User parameters at 1, 2, 3...
    /// - **Index adjustment**: All parameter access is offset by +1
    ///
    /// ### Type Preservation
    /// Parameter types are preserved during transformation:
    /// - **Type conversion**: User parameter types converted to LLVM types
    /// - **Signature construction**: LLVM function type includes all parameters
    /// - **Binding maintenance**: Parameter names remain accessible in method body
    ///
    /// ## Scope and Symbol Management
    ///
    /// ### Method Scope Creation
    /// Each method gets its own isolated scope:
    /// - **Parameter isolation**: Method parameters don't conflict with outer scope
    /// - **Local variables**: Method locals are contained within method scope
    /// - **Automatic cleanup**: Scope is cleaned up when method compilation completes
    ///
    /// ### Symbol Registration
    /// Variables are registered in the method scope:
    /// - **This binding**: `this` parameter registered as `this` variable
    /// - **Parameter binding**: User parameters registered by name
    /// - **Name resolution**: Variables accessible by name within method body
    ///
    /// ## Function Generation Strategy
    ///
    /// ### LLVM Function Creation
    /// The method becomes a standard LLVM function:
    /// - **Mangled name**: Uses the provided method name for uniqueness
    /// - **Function type**: Constructed from modified parameter list
    /// - **Module registration**: Added to the LLVM module for linking
    ///
    /// ### Basic Block Management
    /// Methods use standard function structure:
    /// - **Entry block**: Single entry point for method execution
    /// - **Return handling**: Proper terminator management for all code paths
    /// - **Void vs value**: Handles both void and value-returning methods
    ///
    /// ## Error Handling
    ///
    /// ### Type System Integration
    /// The function validates several type system constraints:
    /// - **Function type**: Method must have validated function type
    /// - **Struct existence**: Target struct type must exist in type system
    /// - **Parameter compatibility**: All parameter types must be valid
    ///
    /// ### Compilation Failures
    /// Various compilation failures are handled:
    /// - **Type lookup failures**: Missing struct types cause panics
    /// - **Parameter access failures**: Missing LLVM parameters cause panics
    /// - **Body compilation failures**: Delegated to body compilation system
    ///
    /// # Parameters
    ///
    /// * `ctx` - Code generation context for LLVM operations
    /// * `function` - The method definition to compile
    /// * `struct_type_name` - Name of the struct type this method belongs to
    /// * `method_name` - Mangled name for the generated LLVM function
    ///
    /// # Panics
    ///
    /// - **Invalid function type**: When method doesn't have function type information
    /// - **Missing struct type**: When target struct type isn't found in type system
    /// - **Parameter mismatch**: When LLVM parameter count doesn't match expectation
    ///
    /// # Implementation Details
    ///
    /// The function carefully manages the parameter index offset to account for
    /// the injected `this` parameter, ensuring that user parameters are correctly
    /// mapped to their LLVM function parameter positions.
    fn compile_instance_method(
        &self,
        ctx: &CodegenContext<'ctx>,
        function: &crate::parser::ast::Function<ValidatedTypeInformation>,
        struct_type_name: &str,
        method_name: &str,
    ) {
        // Extract function information
        let crate::parser::ast::Function {
            id,
            parameters,
            body,
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
        } = function
        else {
            unreachable!("Instance method should have function type")
        };

        // Create 'this' parameter type - get actual struct type from type system
        let struct_type = self
            .find_struct_type(ctx, struct_type_name)
            .unwrap_or_else(|| {
                panic!(
                    "Struct type {} not found in type system for instance method",
                    struct_type_name
                )
            });
        let this_type = Type::Reference(Box::new(struct_type));

        // Build modified parameter list with 'this' as first parameter
        let mut method_params = vec![this_type];
        method_params.extend(params.clone());

        // Build LLVM function type with 'this' parameter
        let llvm_fn_type =
            build_llvm_function_type_from_own_types(ctx, return_value, &method_params);

        // Create LLVM function with modified name
        let llvm_fn_value = ctx.module.add_function(method_name, llvm_fn_type, None);

        // Store the function in scope (for potential recursive calls)
        ctx.store_function(&id.name, llvm_fn_value);

        // Enter scope for method parameters and local variables
        ctx.enter_scope();

        // Inject 'this' parameter as first parameter
        let this_param_value = llvm_fn_value
            .get_nth_param(0)
            .expect("'this' parameter should exist");
        ctx.store_variable("this", this_param_value);

        // Handle regular function parameters (starting from index 1)
        for (i, param) in parameters.iter().enumerate() {
            let FunctionParameter { name, .. } = param;

            let llvm_param_value = llvm_fn_value
                .get_nth_param((i + 1) as u32) // +1 because 'this' is at index 0
                .expect("Function parameter should exist");

            ctx.store_variable(&name.name, llvm_param_value);
        }

        // Create basic block and set insertion point
        let llvm_fn_bb = ctx.context.append_basic_block(llvm_fn_value, "entry");
        ctx.builder.position_at_end(llvm_fn_bb);

        // Compile method body and capture yielded value
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

        // Exit method scope
        ctx.exit_scope();
    }

    fn find_struct_type(&self, ctx: &CodegenContext<'ctx>, struct_name: &str) -> Option<Type> {
        // Look up the struct type from the types registry
        // Iterate through registered types to find one that matches the struct name
        for (type_key, _llvm_type) in ctx.types.borrow().iter() {
            if let Type::Struct(name, fields) = type_key {
                if name == struct_name {
                    return Some(Type::Struct(name.clone(), fields.clone()));
                }
            }
        }
        None
    }
}
