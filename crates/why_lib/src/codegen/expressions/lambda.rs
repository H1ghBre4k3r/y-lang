//! # Lambda Expression Code Generation
//!
//! Comprehensive implementation of closure (lambda) lowering from validated AST to
//! uniform runtime representation. This header augments the high-level closure system
//! overview in `codegen/mod.rs` with a lambda‑focused execution narrative, invariants,
//! and environment lifecycle details.
//!
//! ## End-to-End Lambda Lifecycle
//!
//! ```text
//!  Parse                    Type Check                         Codegen (this module)                      Runtime
//!  -----                    ----------                         ---------------------                      -------
//!  \(a, b) => a + x   →    infer function type          →      build impl fn (env? + params)       →     call dispatch extracts {fn*, env*}
//!                           capture analysis (x?)               allocate & populate env (if needed)       passes env*, params*
//!                                                               generate body w/ captured bindings         indirect call executes
//!                                                               wrap { fn*, env* }
//! ```
//!
//! ## Core Responsibilities
//! - Decide capturing vs non‑capturing lowering path
//! - Allocate & populate environment for capturing lambdas
//! - Produce implementation function with correct leading env parameter (capturing)
//! - Rebind captured variables inside implementation scope via environment struct loads
//! - Wrap function + environment pointer into `{ i8*, i8* }` closure struct
//! - Return unified closure value to caller expression
//!
//! ## Invariants (Lambda-Specific)
//! - Capturing implementation always has first param `i8* env`
//! - Non‑capturing implementation has no env param (pure `(params...) -> ret`)
//! - Environment struct field ordering matches `capture_info.captures` ordering
//! - Captured values copied by value at creation time (no alias back to originals)
//! - Environment pointer stored in closure is exactly the heap allocation base as `i8*`
//! - Builder insertion position restored after body emission
//! - Scope pushed exactly once per lambda body and popped exactly once
//!
//! ## Environment Memory Model
//! - Allocation: raw `malloc(size_of(env_struct))`
//! - Lifetime: intentionally unbounded (leak) until future lifetime / GC strategy
//! - Mutability: fields treated as immutable post‑construction (writes only during population)
//! - Sharing: multiple uses of the same closure share identical environment pointer
//! - Escape: safe because env is heap‑allocated, may outlive defining lexical stack frames
//!
//! ## Error Handling Philosophy
//! All panics correspond to logically unreachable states under a validated AST:
//! - Missing function type for lambda
//! - Missing environment parameter when capture info present
//! - GEP failure for known field indices
//! - Metadata→basic type conversion failures for capture types
//!
//! ## Future Evolution Points
//! - Reference / move / mutable capture kinds
//! - Environment deallocation / reference counting
//! - Small closure optimisation (represent null-env closures as tagged fn pointer)
//! - Partial application layering additional env frames
//! - Environment shape hashing + interning
//! - Debug metadata for capture variable names & offsets
//!
//!
//! This module implements LLVM code generation for lambda expressions (closures) in Y-lang.
//! It handles both capturing and non-capturing lambdas with a unified closure representation.
//!
//! ## Closure Implementation Strategy
//!
//! All lambdas are represented as closure structs `{i8*, i8*}` containing:
//! - **Function pointer**: Points to the lambda implementation function
//! - **Environment pointer**: Points to captured variables (null for non-capturing lambdas)
//!
//! ## Capturing vs Non-Capturing Lambdas
//!
//! - **Non-capturing**: Simple function with null environment, wrapped in closure struct
//! - **Capturing**: Function with environment parameter, heap-allocated environment struct
//!
//! ## Environment Management
//!
//! Captured variables are stored in a heap-allocated struct with fields for each captured variable.
//! The environment pointer is passed as the first parameter to the lambda implementation function.

use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{
        statements::function::build_llvm_function_type_from_own_types, CodeGen, CodegenContext,
    },
    parser::ast::{Lambda, LambdaParameter},
    typechecker::{get_lambda_captures, CaptureInfo, Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Lambda<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    /// Generates LLVM IR for a lambda expression.
    ///
    /// This is the main entry point for lambda code generation. It dispatches to either
    /// capturing or non-capturing lambda generation based on capture analysis from the type checker.
    ///
    /// ## Process Overview
    ///
    /// 1. **Extract type information**: Ensure lambda has function type
    /// 2. **Generate unique identifier**: Based on source position for lambda naming
    /// 3. **Retrieve capture info**: From type checker to determine if lambda captures variables
    /// 4. **Dispatch generation**: Call appropriate method based on capture status
    /// 5. **Return closure struct**: All lambdas return closure struct representation
    ///
    /// ## Lambda Identification
    ///
    /// Each lambda gets a unique identifier based on its source position, ensuring that
    /// multiple lambdas in the same file get distinct function names in the LLVM module.
    ///
    /// # Returns
    ///
    /// `Some(BasicValueEnum)` containing the closure struct, or `None` on failure
    ///
    /// ## Rationale
    /// Always returns the uniform closure representation to keep higher‑order constructs
    /// agnostic to capturing status. Returning raw function values for non‑capturing cases
    /// would complicate call dispatch polymorphism.
    ///
    /// ## Performance
    /// Minimal overhead: constructing a two‑field struct; enables potential future inline
    /// caching of call sites because representation is stable.
    ///
    /// # Panics
    ///
    /// Panics if the lambda doesn't have a function type (should not happen with validated AST)
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let Lambda {
            parameters,
            expression,
            info:
                ValidatedTypeInformation {
                    type_id:
                        Type::Function {
                            params,
                            return_value,
                        },
                    ..
                },
            position,
            ..
        } = self
        else {
            panic!("Lambda should have function type");
        };

        // Generate unique lambda identifier from position
        let lambda_id = format!(
            "lambda_{}_{}_{}_{}",
            position.start.0, position.start.1, position.end.0, position.end.1
        );

        // Retrieve capture information
        let captures = get_lambda_captures(&lambda_id);

        // Generate a unique name for the lambda implementation function
        let lambda_name = format!("lambda_impl_{}", *ctx.lambda_counter.borrow());
        *ctx.lambda_counter.borrow_mut() += 1;

        if let Some(capture_info) = captures.as_ref() {
            if !capture_info.is_empty() {
                // This lambda captures variables - generate closure implementation
                self.codegen_capturing_lambda(
                    ctx,
                    &lambda_name,
                    parameters,
                    expression,
                    params,
                    return_value,
                    capture_info,
                )
            } else {
                // Non-capturing lambda - generate simple function pointer
                self.codegen_non_capturing_lambda(
                    ctx,
                    &lambda_name,
                    parameters,
                    expression,
                    params,
                    return_value,
                )
            }
        } else {
            // No capture info found - assume non-capturing
            self.codegen_non_capturing_lambda(
                ctx,
                &lambda_name,
                parameters,
                expression,
                params,
                return_value,
            )
        }
    }
}

impl<'ctx> Lambda<ValidatedTypeInformation> {
    /// Generates code for a non-capturing lambda.
    ///
    /// Non-capturing lambdas are simpler because they don't need environment management.
    /// They're implemented as regular functions wrapped in a closure struct with a null
    /// environment pointer.
    ///
    /// ## Implementation Strategy
    ///
    /// 1. **Create standard function**: Use normal function type (no environment parameter)
    /// 2. **Generate function body**: Delegate to shared body generation logic
    /// 3. **Wrap in closure**: Create closure struct with null environment
    ///
    /// ## Function Signature
    ///
    /// Non-capturing lambda functions have the signature `(params...) -> ret`
    /// without any environment parameter, making them compatible with regular functions.
    ///
    /// ## Memory Efficiency
    ///
    /// Since there's no environment to allocate, non-capturing lambdas have minimal
    /// memory overhead - just the closure struct on the stack.
    ///
    /// # Parameters
    ///
    /// * `ctx` - Code generation context
    /// * `lambda_name` - Unique name for the lambda function
    /// * `parameters` - Lambda parameter declarations
    /// * `expression` - Lambda body expression
    /// * `params` - Parameter types from type checker
    /// * `return_value` - Return type from type checker
    ///
    /// # Returns
    ///
    /// `Some(BasicValueEnum)` containing the closure struct
    ///
    /// ## Rationale
    /// We materialise a closure struct even for non‑capturing lambdas to preserve a single
    /// uniform higher‑order function ABI. This avoids monomorphisation of call sites on
    /// the presence/absence of an environment pointer. A future Small Closure Optimisation
    /// (SCO) could elide the struct and use raw fn pointers with a tagged null, but the
    /// current design favours simplicity and predictable IR patterns.
    ///
    /// ## Performance Notes
    /// - Zero heap allocation path
    /// - Only cost beyond a plain function pointer is constructing the two‑field struct
    /// - Bitcast of function pointer happens inside `build_closure_value` once.
    fn codegen_non_capturing_lambda(
        &self,
        ctx: &CodegenContext<'ctx>,
        lambda_name: &str,
        parameters: &[LambdaParameter<ValidatedTypeInformation>],
        expression: &Box<crate::parser::ast::Expression<ValidatedTypeInformation>>,
        params: &[Type],
        return_value: &Type,
    ) -> Option<BasicValueEnum<'ctx>> {
        // Build standard function type (without env parameter)
        let llvm_fn_type = build_llvm_function_type_from_own_types(ctx, return_value, params);

        // Create the lambda function
        let lambda_fn = ctx.module.add_function(lambda_name, llvm_fn_type, None);

        // Note: We don't store non-capturing lambdas by name since they're typically used inline

        // Generate function body
        self.generate_lambda_body(ctx, lambda_fn, parameters, expression, return_value, None);

        // Create closure struct with env = null
        let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
        let null_env = ctx
            .context
            .ptr_type(inkwell::AddressSpace::default())
            .const_null();
        let closure_struct = ctx.build_closure_value(fn_ptr, null_env);

        Some(closure_struct.into())
    }

    /// Generates code for a capturing lambda (closure with environment).
    ///
    /// Capturing lambdas are more complex because they need to:
    /// 1. Create an environment struct for captured variables
    /// 2. Allocate the environment on the heap
    /// 3. Use a modified function signature with environment parameter
    ///
    /// ## Implementation Strategy
    ///
    /// 1. **Create closure function type**: With environment pointer as first parameter
    /// 2. **Build environment**: Allocate and populate struct with captured variables
    /// 3. **Generate function body**: With access to captured variables via environment
    /// 4. **Return closure**: Struct containing function pointer and environment pointer
    ///
    /// ## Function Signature
    ///
    /// Capturing lambda functions have the signature `(i8* env, params...) -> ret`
    /// where the first parameter is always the environment pointer.
    ///
    /// ## Memory Management
    ///
    /// The environment is heap-allocated using malloc and contains all captured variables.
    /// This enables the lambda to outlive the scope where it was created.
    ///
    /// ## Environment Access
    ///
    /// Inside the lambda body, captured variables are accessed by:
    /// 1. Casting environment pointer to correct struct type
    /// 2. Using GEP to get field pointers
    /// 3. Loading values from the environment struct
    ///
    /// # Parameters
    ///
    /// * `ctx` - Code generation context
    /// * `lambda_name` - Unique name for the lambda function
    /// * `parameters` - Lambda parameter declarations
    /// * `expression` - Lambda body expression
    /// * `params` - Parameter types from type checker
    /// * `return_value` - Return type from type checker
    /// * `capture_info` - Information about captured variables and their types
    ///
    /// # Returns
    ///
    /// `Some(BasicValueEnum)` containing the closure struct with valid environment
    ///
    /// ## Rationale
    /// Environment allocation precedes body generation so that recursive references (if
    /// later enabled) or nested lambdas could capture the just‑allocated environment pointer.
    /// The environment is copied by value to decouple lifetime from original stack slots.
    ///
    /// ## LLVM Operation Ordering
    /// 1. Type creation before allocation: ensures size query stability
    /// 2. Allocation before body: permits potential future writes (e.g. for mutable captures)
    /// 3. Body generation last: ensures all symbols (captures + params) are in scope.
    ///
    /// ## Optimisation Opportunities
    /// - Promote small fixed environments to stack if proven non‑escaping
    /// - Reuse environment layouts across identical capture shape via interning
    /// - Fuse allocation + population via `llvm.memcpy` if many fields.
    fn codegen_capturing_lambda(
        &self,
        ctx: &CodegenContext<'ctx>,
        lambda_name: &str,
        parameters: &[LambdaParameter<ValidatedTypeInformation>],
        expression: &Box<crate::parser::ast::Expression<ValidatedTypeInformation>>,
        params: &[Type],
        return_value: &Type,
        capture_info: &CaptureInfo,
    ) -> Option<BasicValueEnum<'ctx>> {
        // Create closure implementation function type (i8* env, params...) -> ret
        let closure_fn_type = ctx.create_closure_impl_fn_type(return_value, params);

        // Create the closure implementation function
        let closure_fn = ctx.module.add_function(lambda_name, closure_fn_type, None);

        // Generate environment struct type and allocate on heap
        let (env_struct_type, env_ptr) = self.create_and_populate_environment(ctx, capture_info);

        // Generate function body (with environment parameter)
        self.generate_lambda_body(
            ctx,
            closure_fn,
            parameters,
            expression,
            return_value,
            Some((env_struct_type, capture_info)),
        );

        // Create closure struct
        let fn_ptr = closure_fn.as_global_value().as_pointer_value();
        let closure_struct = ctx.build_closure_value(fn_ptr, env_ptr);

        Some(closure_struct.into())
    }

    /// Generates the lambda function body with proper scoping and parameter binding.
    ///
    /// This method handles the complex task of setting up the lambda's execution context,
    /// including parameter binding, environment variable access, and return value handling.
    ///
    /// ## Implementation Process
    ///
    /// 1. **Create entry block**: Set up the function's entry point
    /// 2. **Setup environment**: If capturing, bind captured variables from environment struct
    /// 3. **Bind parameters**: Add user parameters to the lambda's scope
    /// 4. **Generate body**: Execute the lambda expression code generation
    /// 5. **Handle return**: Add appropriate return instruction based on return type
    /// 6. **Restore context**: Clean up scopes and restore IR builder position
    ///
    /// ## Environment Parameter Handling
    ///
    /// For capturing lambdas, the environment parameter (first parameter) is:
    /// 1. Cast from generic `i8*` to the specific environment struct type
    /// 2. Each captured variable is extracted using GEP and loaded into the scope
    /// 3. User parameters are offset by 1 to account for the environment parameter
    ///
    /// ## Scope Management
    ///
    /// The lambda body executes in its own scope containing:
    /// - Captured variables (if any)
    /// - Lambda parameters
    /// - Any variables declared within the lambda body
    ///
    /// ## Return Value Generation
    ///
    /// - **Void lambdas**: Generate `ret void`
    /// - **Value-returning lambdas**: Generate `ret <value>` with the expression result
    /// - **No value**: Generate `unreachable` (indicates a bug in the lambda body)
    ///
    /// # Parameters
    ///
    /// * `ctx` - Code generation context
    /// * `lambda_fn` - LLVM function value for the lambda implementation
    /// * `parameters` - User-defined lambda parameters
    /// * `expression` - Lambda body expression to generate code for
    /// * `return_value` - Expected return type
    /// * `env_info` - Optional environment information for capturing lambdas
    ///
    /// ## Rationale
    /// Centralises all per‑lambda IR construction to keep the specialised capturing / non‑capturing
    /// entry points minimal. Restoring the previous builder insertion point avoids accidental
    /// instruction emission after the lambda body into the parent function tail.
    ///
    /// ## Performance Considerations
    /// Parameter rebinding is O(P + C) (params + captures). No additional heap traffic occurs here.
    /// Loads of captured values happen once at entry; subsequent uses are SSA values.
    ///
    /// ## Safety Invariants Reinforced Here
    /// - Scope push/pop symmetry
    /// - Return insertion in all control paths (or explicit unreachable) to satisfy LLVM verifier.
    ///
    /// ## Future Extensions
    /// - Insert debug metadata for parameter & capture names
    /// - Support destructors / deferred drops prior to return (post lifetime work).
    fn generate_lambda_body(
        &self,
        ctx: &CodegenContext<'ctx>,
        lambda_fn: inkwell::values::FunctionValue<'ctx>,
        parameters: &[LambdaParameter<ValidatedTypeInformation>],
        expression: &Box<crate::parser::ast::Expression<ValidatedTypeInformation>>,
        return_value: &Type,
        env_info: Option<(inkwell::types::StructType<'ctx>, &CaptureInfo)>,
    ) {
        // Create the entry basic block
        let entry_bb = ctx.context.append_basic_block(lambda_fn, "entry");

        // Store current builder position
        let current_bb = ctx.builder.get_insert_block();

        // Position builder at the entry block
        ctx.builder.position_at_end(entry_bb);

        // Enter scope for lambda body execution
        ctx.enter_scope();

        let mut param_offset = 0;

        // Handle environment parameter if this is a capturing lambda
        if let Some((env_struct_type, capture_info)) = env_info {
            let env_param = lambda_fn
                .get_nth_param(0)
                .expect("Environment parameter should exist")
                .into_pointer_value();

            // Cast environment pointer back to struct type
            let env_struct_ptr = ctx
                .builder
                .build_bit_cast(
                    env_param,
                    ctx.context.ptr_type(inkwell::AddressSpace::default()),
                    "env_cast",
                )
                .unwrap()
                .into_pointer_value();

            // Bind captured variables into scope
            for (i, (var_name, _var_type)) in capture_info.captures.iter().enumerate() {
                let field_ptr = ctx
                    .builder
                    .build_struct_gep(
                        env_struct_type,
                        env_struct_ptr,
                        i as u32,
                        &format!("capture_{}_ptr", var_name),
                    )
                    .unwrap();

                let field_type = env_struct_type.get_field_type_at_index(i as u32).unwrap();
                let field_value = ctx
                    .builder
                    .build_load(field_type, field_ptr, &format!("capture_{}", var_name))
                    .unwrap();

                ctx.store_variable(var_name, field_value);
            }

            param_offset = 1; // Skip environment parameter for user parameters
        }

        // Set up user parameters in the lambda scope
        for (i, param) in parameters.iter().enumerate() {
            let LambdaParameter { name, .. } = param;
            let llvm_param_value = lambda_fn
                .get_nth_param((i + param_offset) as u32)
                .expect("Lambda parameter should exist");

            ctx.store_variable(&name.name, llvm_param_value);
        }

        // Generate code for the lambda body
        let result = expression.codegen(ctx);

        // Add return instruction based on return type
        match return_value {
            Type::Void => {
                ctx.builder.build_return(None).unwrap();
            }
            _ => {
                if let Some(value) = result {
                    ctx.builder.build_return(Some(&value)).unwrap();
                } else {
                    // If no value returned, this is an error but we'll add unreachable
                    ctx.builder.build_unreachable().unwrap();
                }
            }
        }

        // Exit lambda scope
        ctx.exit_scope();

        // Restore builder position
        if let Some(bb) = current_bb {
            ctx.builder.position_at_end(bb);
        }
    }

    /// Creates and populates the environment struct for capturing lambdas.
    ///
    /// This method handles the complex process of creating a heap-allocated environment
    /// that contains all captured variables. The environment enables the lambda to access
    /// variables from its lexical scope even after those scopes have been exited.
    ///
    /// ## Environment Struct Creation
    ///
    /// 1. **Determine field types**: Convert captured variable types to LLVM types
    /// 2. **Create struct type**: Build LLVM struct with fields for each captured variable
    /// 3. **Allocate on heap**: Use malloc to allocate environment struct
    /// 4. **Populate fields**: Copy current values of captured variables into the struct
    ///
    /// ## Memory Management Strategy
    ///
    /// The environment is allocated on the heap using malloc because:
    /// - **Lifetime**: Must outlive the scope where the lambda was created
    /// - **Sharing**: Multiple lambda invocations may share the same environment
    /// - **Size**: Unknown at compile time (depends on captured variables)
    ///
    /// ## Field Layout
    ///
    /// Environment fields are laid out in the order they appear in `capture_info.captures`:
    /// - Field 0: First captured variable
    /// - Field 1: Second captured variable
    /// - etc.
    ///
    /// ## GEP Operations for Environment Access
    ///
    /// Each captured variable is stored using LLVM's `build_struct_gep` to get a pointer
    /// to the appropriate field, then `build_store` to copy the current value.
    ///
    /// ## Malloc Integration
    ///
    /// If malloc is not already declared in the module, it's automatically declared
    /// with the signature `i8* malloc(i64 size)` (assumes size_t=i64 on current targets).
    /// The size is computed using LLVM's `size_of` on the struct type. On 32‑bit targets,
    /// this should be adapted to use i32 for the size parameter.
    ///
    /// # Parameters
    ///
    /// * `ctx` - Code generation context
    /// * `capture_info` - Information about captured variables and their types
    ///
    /// # Returns
    ///
    /// Tuple of (environment struct type, pointer to allocated and populated environment)
    ///
    /// # LLVM Operations Used
    ///
    /// - `struct_type()`: Create the environment struct type
    /// - `size_of()`: Get struct size for malloc
    /// - `build_call()`: Call malloc
    /// - `build_bit_cast()`: Cast malloc result to struct pointer
    /// - `build_struct_gep()`: Get field pointers
    /// - `build_store()`: Store captured values
    ///
    /// ## Rationale
    /// Captured values are copied eagerly to guarantee stability even if original stack
    /// allocations go out of scope. Using a contiguous struct enables single-pointer
    /// passing and potential future hashing / interning of layouts.
    ///
    /// ## Alignment & Padding
    /// We rely on LLVM's natural (non‑packed) struct layout for correct alignment. This may
    /// introduce padding; future optimisation could pack when all fields share alignment.
    ///
    /// ## Potential Improvements
    /// - Stack promotion for non‑escaping environments (escape analysis)
    /// - Reference / move capture kinds (avoid copies for large aggregates)
    /// - Deferred / pooled allocation strategy to reduce malloc pressure.
    ///
    /// ## Complexity
    /// Time: O(C) where C = number of captures. Space: sizeof(struct(captures)).
    fn create_and_populate_environment(
        &self,
        ctx: &CodegenContext<'ctx>,
        capture_info: &CaptureInfo,
    ) -> (
        inkwell::types::StructType<'ctx>,
        inkwell::values::PointerValue<'ctx>,
    ) {
        // Create environment struct type
        let mut field_types = Vec::new();
        for (_name, var_type) in &capture_info.captures {
            let llvm_type = ctx.get_llvm_type(var_type);
            if let Some(basic_type) = crate::codegen::convert_metadata_to_basic(llvm_type) {
                field_types.push(basic_type);
            } else {
                panic!("Cannot convert captured variable type to basic type");
            }
        }

        let env_struct_type = ctx.context.struct_type(&field_types, false);

        // Allocate environment on heap (using malloc-like allocation)
        let env_size = env_struct_type.size_of().unwrap();
        let malloc_fn = ctx.module.get_function("malloc").unwrap_or_else(|| {
            // Declare malloc if not already declared
            let i8_ptr_type = ctx.context.ptr_type(inkwell::AddressSpace::default());
            let size_t_type = ctx.context.i64_type(); // Assuming size_t is i64
            let malloc_type = i8_ptr_type.fn_type(&[size_t_type.into()], false);
            ctx.module.add_function("malloc", malloc_type, None)
        });

        let env_ptr_i8 = ctx
            .builder
            .build_call(malloc_fn, &[env_size.into()], "env_malloc")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();

        // Cast to struct pointer
        let env_ptr = ctx
            .builder
            .build_bit_cast(
                env_ptr_i8,
                ctx.context.ptr_type(inkwell::AddressSpace::default()),
                "env_cast",
            )
            .unwrap()
            .into_pointer_value();

        // Populate environment with captured values
        for (i, (var_name, var_type)) in capture_info.captures.iter().enumerate() {
            let captured_ptr = ctx.find_variable(var_name);
            let field_ptr = ctx
                .builder
                .build_struct_gep(
                    env_struct_type,
                    env_ptr,
                    i as u32,
                    &format!("env_field_{}", i),
                )
                .unwrap();

            // Handle both direct values and pointer values
            let captured_value = match captured_ptr {
                inkwell::values::BasicValueEnum::PointerValue(ptr) => {
                    // Load the actual value from the pointer
                    let llvm_type = ctx.get_llvm_type(var_type);
                    if let Some(basic_type) = crate::codegen::convert_metadata_to_basic(llvm_type) {
                        ctx.builder.build_load(basic_type, ptr, var_name).unwrap()
                    } else {
                        // Store the pointer directly for complex types
                        captured_ptr
                    }
                }
                _ => {
                    // Value is already loaded, use it directly
                    captured_ptr
                }
            };
            ctx.builder.build_store(field_ptr, captured_value).unwrap();
        }

        // Return struct type and pointer as i8*
        (env_struct_type, env_ptr_i8)
    }
}
