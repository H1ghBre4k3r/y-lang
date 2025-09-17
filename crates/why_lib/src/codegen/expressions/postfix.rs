//! # Postfix Expression Code Generation
//!
//! This module implements LLVM code generation for postfix expressions in Y-lang.
//! Postfix expressions include array indexing, property access, and function calls.
//!
//! ## Supported Operations
//!
//! ### Array Indexing (`expr[index]`)
//! - **GEP Operations**: Uses LLVM's `build_gep` for safe pointer arithmetic
//! - **Type Safety**: Validates array types and converts indices to appropriate types
//! - **Memory Access**: Loads values from computed array element addresses
//!
//! ### Property Access (`expr.field`)
//! - **Struct Field Access**: Uses `build_struct_gep` for field pointer calculation
//! - **Temporary Allocation**: Handles value structs by allocating temporary storage
//! - **Type Validation**: Ensures accessed fields exist in the struct type
//!
//! ### Function Calls (`expr(args...)`)
//! - **Direct Calls**: Named functions resolved from the module
//! - **Method Calls**: Instance methods with `this` parameter injection
//! - **Closure Calls**: Indirect calls through closure structs with environment handling
//!
//! ## LLVM Operations Used
//!
//! - **GEP**: For safe pointer arithmetic in arrays and structs
//! - **Load/Store**: For memory access and temporary allocation
//! - **Function Calls**: Direct, indirect, and method call patterns
//! - **Type Casting**: For closure function pointer extraction and casting

use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

use crate::{
    codegen::{
        convert_metadata_to_basic, statements::function::build_llvm_function_type_from_own_types,
        CodeGen, CodegenContext,
    },
    parser::ast::{Expression, Postfix},
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Postfix<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    /// Generates LLVM IR for postfix expressions.
    ///
    /// This method dispatches to specialized handlers for each type of postfix operation.
    /// Each operation has different LLVM IR generation requirements and memory access patterns.
    ///
    /// ## Operation Dispatch
    ///
    /// - **`Call`**: Handled by `codegen_call` with complex function resolution logic
    /// - **`Index`**: Array element access using GEP and load operations
    /// - **`PropertyAccess`**: Struct field access with type validation and GEP
    ///
    /// ## Memory Access Patterns
    ///
    /// Different postfix operations have different memory access characteristics:
    /// - **Array indexing**: Always results in a load from computed address
    /// - **Property access**: May require temporary allocation for value structs
    /// - **Function calls**: May be direct, indirect, or method calls with varying conventions
    ///
    /// # Returns
    ///
    /// `Some(BasicValueEnum)` containing the result value, or `None` for void operations
    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        match self {
            Postfix::Call { expr, args, .. } => Self::codegen_call(ctx, expr, args),
            // Array indexing: expr[index] -> element value
            Postfix::Index { expr, index, .. } => {
                // Generate code for the array expression (should produce a pointer to array)
                let Some(array_value) = expr.codegen(ctx) else {
                    unreachable!("Array expression must produce a value")
                };

                // Generate code for the index expression (should produce an integer)
                let Some(index_value) = index.codegen(ctx) else {
                    unreachable!("Index expression must produce a value")
                };

                let array_ptr = array_value.into_pointer_value();
                let index_int = index_value.into_int_value();

                // Extract array element type from Y-lang type system
                let expr_type = &expr.get_info().type_id;
                let Type::Array(element_type) = expr_type else {
                    unreachable!("Index expression must be on array type")
                };

                // Convert element type to LLVM representation
                let llvm_element_type = ctx.get_llvm_type(element_type);
                let element_basic_type = convert_metadata_to_basic(llvm_element_type)
                    .expect("Array element type must be basic");

                // Use GEP (GetElementPtr) to calculate the address of the indexed element
                // This is safe pointer arithmetic - LLVM ensures bounds are respected
                let element_ptr = unsafe {
                    ctx.builder
                        .build_gep(
                            element_basic_type,
                            array_ptr,
                            &[index_int], // Single index for linear array access
                            "array_index",
                        )
                        .unwrap()
                };

                // Load the actual value from the computed address
                let element_value = ctx
                    .builder
                    .build_load(element_basic_type, element_ptr, "array_elem")
                    .unwrap();

                Some(element_value)
            }
            // Property access: expr.field -> field value
            Postfix::PropertyAccess { expr, property, .. } => {
                // Generate code for the struct expression
                let Some(struct_value) = expr.codegen(ctx) else {
                    panic!("Struct expression must produce a value for property access");
                };

                let property_name = &property.name;

                // Extract struct type information and validate field existence
                // This uses Y-lang's type system to ensure type safety
                let (struct_name, field_types, field_index) = match &expr.get_info().type_id {
                    Type::Struct(struct_name, field_types) => {
                        // Linear search for field index by name
                        let field_index = field_types
                            .iter()
                            .position(|(name, _)| name == property_name)
                            .unwrap_or_else(|| {
                                panic!(
                                    "Field {} not found in struct {}",
                                    property_name, struct_name
                                )
                            });
                        (struct_name.clone(), field_types.clone(), field_index)
                    }
                    other_type => {
                        panic!(
                            "Property access only supported on struct types, got: {:?}",
                            other_type
                        );
                    }
                };

                // Retrieve the corresponding LLVM struct type from the type cache
                let struct_type = {
                    let types_guard = ctx.types.borrow();
                    let struct_type_id = Type::Struct(struct_name.clone(), field_types.clone());

                    match types_guard.get(&struct_type_id) {
                        Some(llvm_type) => {
                            if let inkwell::types::BasicMetadataTypeEnum::StructType(struct_type) =
                                llvm_type
                            {
                                *struct_type
                            } else {
                                panic!(
                                    "Expected struct type for property access, got: {:?}",
                                    llvm_type
                                )
                            }
                        }
                        None => {
                            panic!(
                                "Struct type {} not found in type context for property access",
                                struct_name
                            );
                        }
                    }
                };

                // Handle both pointer and value structs
                // Value structs need temporary allocation for GEP operations
                let struct_ptr = if struct_value.is_pointer_value() {
                    // Already a pointer - use directly
                    struct_value.into_pointer_value()
                } else {
                    // Value struct - allocate temporary storage and store the value
                    let temp_ptr = ctx
                        .builder
                        .build_alloca(struct_type, "temp_struct")
                        .unwrap();
                    ctx.builder.build_store(temp_ptr, struct_value).unwrap();
                    temp_ptr
                };

                // Use struct GEP to get pointer to the specific field
                // Requires two indices: [0, field_index] for struct field access
                let field_ptr = unsafe {
                    ctx.builder
                        .build_gep(
                            struct_type,
                            struct_ptr,
                            &[
                                ctx.context.i32_type().const_zero(), // Struct base offset
                                ctx.context.i32_type().const_int(field_index as u64, false), // Field offset
                            ],
                            &format!(
                                "{}_{}",
                                struct_ptr.get_name().to_string_lossy(),
                                property_name
                            ),
                        )
                        .unwrap()
                };

                // Load the field value from the computed address
                let field_value = ctx
                    .builder
                    .build_load(
                        struct_type
                            .get_field_type_at_index(field_index as u32)
                            .expect("Field type must exist"),
                        field_ptr,
                        property_name,
                    )
                    .unwrap();

                Some(field_value)
            }
        }
    }
}

impl<'ctx> Postfix<ValidatedTypeInformation> {
    /// Generates LLVM IR for function calls with comprehensive dispatch logic.
    ///
    /// ## Dispatch Decision Tree (Closure / Function / Method)
    /// ```text
    ///  Expression Kind
    ///    │
    ///    ├─ Postfix::PropertyAccess(struct_expr . method) & struct_expr.type == Struct
    ///    │    └─► Instance Method Call Path
    ///    │         name = "{Struct}_{method}"; prepend 'this' pointer; direct call
    ///    │
    ///    ├─ Id(name)
    ///    │    ├─ module.contains_function(name) ─► Direct Named Function Call
    ///    │    └─ else ─► treat as value expression (fall through to closure path)
    ///    │
    ///    └─ any other expression
    ///         └─► Closure Value Path
    ///              extract { fn*, env* }
    ///              if env* == null → non‑capturing indirect call
    ///              else            → capturing indirect call (env first)
    /// ```
    ///
    /// ## Representation Rationale
    /// A unified closure struct `{ i8*, i8* }` allows all higher‑order call sites to share
    /// identical indirect call machinery, reducing IR variance and simplifying optimisation
    /// opportunities (e.g. devirtualisation heuristics, inline caching).
    ///
    /// ## Invariants
    /// - Non‑capturing closures always have null environment pointer
    /// - Capturing closures always supply environment pointer as first argument
    /// - Method calls always materialise a pointer for `this` even if receiver is a value struct
    /// - Id + module hit: bypass closure extraction (direct call); otherwise treat as closure value
    ///
    /// ## Performance Notes
    /// - Direct function & method calls avoid closure struct extraction
    /// - Indirect closure calls pay: 2 extract_value + 1 bitcast + indirect call
    /// - Non‑capturing indirect path could be SCO‑optimised later to raw fn pointer dispatch
    ///
    /// ## Failure Modes (Panics / Unreachables)
    /// - Type mismatch of call expression (validated earlier)
    /// - Missing method/function after positive structural identification (should not happen)
    /// - Closure expression not yielding struct value (rep invariant violation)
    ///
    /// ## Future Enhancements
    /// - Inline fast path for frequent non‑capturing closures
    /// - Partial application (would synthesise intermediate closure on call)
    /// - Tail call marking when return position & compatible signature
    /// - Devirtualisation attempt for monomorphic closure sites with known fn pointer.
    ///
    ///
    /// This method handles multiple types of function calls in Y-lang:
    /// 1. **Method calls**: `struct_instance.method_name(args)`
    /// 2. **Direct function calls**: `function_name(args)`
    /// 3. **Closure calls**: `lambda_expr(args)` or `function_variable(args)`
    ///
    /// ## Call Type Detection
    ///
    /// The method uses expression analysis to determine the call type:
    /// - **PropertyAccess on Struct**: Treated as method call with `this` parameter
    /// - **Id expression**: Direct function lookup in LLVM module
    /// - **Other expressions**: Indirect call through closure struct
    ///
    /// ## Method Call Handling
    ///
    /// Method calls use the naming convention `{struct_name}_{method_name}` and:
    /// - Pass the struct instance as the first parameter (`this`)
    /// - Convert the struct to a pointer if needed for the `this` parameter
    /// - Handle both value and pointer struct instances
    ///
    /// ## Closure Call Mechanism
    ///
    /// Closure calls extract function pointer and environment from the closure struct:
    /// - **Non-capturing closures**: Call with original parameters (env pointer is null)
    /// - **Capturing closures**: Call with environment as first parameter
    /// - Use indirect call with extracted function pointer
    ///
    /// ## Parameter Handling
    ///
    /// All function arguments are converted to `BasicMetadataValueEnum` for LLVM compatibility.
    /// Method calls prepend the `this` parameter, while closure calls may prepend environment.
    ///
    /// # Parameters
    ///
    /// * `ctx` - Code generation context
    /// * `expr` - Expression being called (function name, method access, or closure expression)
    /// * `args` - Function arguments to be passed
    ///
    /// # Returns
    ///
    /// `Some(BasicValueEnum)` for non-void functions, `None` for void functions
    ///
    /// # LLVM Operations Used
    ///
    /// - **`build_call`**: For direct function calls
    /// - **`build_indirect_call`**: For closure calls
    /// - **`build_extract_value`**: For extracting function/environment pointers from closures
    /// - **`build_bit_cast`**: For function pointer type casting
    ///
    /// ### IR Sketch for Indirect Call
    /// ```llvm
    /// %fn_i8  = extractvalue { i8*, i8* } %clos, 0
    /// %env    = extractvalue { i8*, i8* } %clos, 1
    /// %res    = call T %fn_typed(i8* %env, <args...>)
    /// ```
    fn codegen_call(
        ctx: &CodegenContext<'ctx>,
        expr: &Expression<ValidatedTypeInformation>,
        args: &[Expression<ValidatedTypeInformation>],
    ) -> Option<BasicValueEnum<'ctx>> {
        let Type::Function {
            params,
            return_value,
        } = expr.get_info().type_id
        else {
            unreachable!()
        };

        let args = args
            .iter()
            .map(|arg| {
                let Some(arg) = arg.codegen(ctx) else {
                    unreachable!()
                };
                arg.into()
            })
            .collect::<Vec<BasicMetadataValueEnum<'ctx>>>();

        // Check if this is a method call (PropertyAccess on struct)
        if let Expression::Postfix(postfix) = expr {
            if let Postfix::PropertyAccess {
                expr: struct_expr,
                property,
                ..
            } = postfix
            {
                // Check if the struct expression has a struct type
                if let Type::Struct(struct_name, _) = &struct_expr.get_info().type_id {
                    // This is a method call: struct_instance.method_name()
                    let method_name = format!("{}_{}", struct_name, property.name);

                    // Look up the instance method
                    if let Some(llvm_method) = ctx.module.get_function(&method_name) {
                        // Generate the struct instance as 'this' parameter
                        let Some(struct_instance) = struct_expr.codegen(ctx) else {
                            unreachable!("Struct expression must produce a value")
                        };

                        // Convert struct instance to pointer if needed for 'this' parameter
                        let struct_pointer = if struct_instance.is_pointer_value() {
                            struct_instance.into_pointer_value().into()
                        } else {
                            // If it's not a pointer, create a temporary allocation and store the value
                            let temp_ptr = ctx
                                .builder
                                .build_alloca(struct_instance.get_type(), "temp_struct_for_method")
                                .unwrap();
                            ctx.builder.build_store(temp_ptr, struct_instance).unwrap();
                            temp_ptr.into()
                        };

                        // Create argument list with 'this' as first parameter (passed as pointer)
                        let mut method_args = vec![struct_pointer];
                        method_args.extend(args);

                        // Call the instance method with 'this' parameter
                        return ctx
                            .builder
                            .build_call(llvm_method, &method_args, "")
                            .unwrap()
                            .try_as_basic_value()
                            .left();
                    }
                }
            }
        }

        // Check if this is a direct function call (Id expression)
        if let Expression::Id(id) = expr {
            // Try to find the function in the LLVM module by name
            let function_name = &id.name;
            if let Some(llvm_function) = ctx.module.get_function(function_name) {
                // Direct call to declared/defined function
                return ctx
                    .builder
                    .build_call(llvm_function, &args, "")
                    .unwrap()
                    .try_as_basic_value()
                    .left();
            }
        }

        // Fallback to indirect call through closure struct
        let Some(expr_value) = expr.codegen(ctx) else {
            unreachable!()
        };

        let BasicValueEnum::StructValue(closure_struct) = expr_value else {
            unreachable!("The Expression in a Call-Postfix should always return a closure struct");
        };

        // Extract function pointer and environment from closure
        let env_ptr = ctx.extract_closure_env_ptr(closure_struct);

        // Check if this is a non-capturing closure (env_ptr is null)
        let null_env = ctx
            .context
            .ptr_type(inkwell::AddressSpace::default())
            .const_null();

        if env_ptr == null_env {
            // Non-capturing closure - call as normal function
            let llvm_function_type =
                build_llvm_function_type_from_own_types(ctx, &return_value, &params);
            let fn_ptr = ctx.extract_closure_fn_ptr(closure_struct, llvm_function_type);

            ctx.builder
                .build_indirect_call(llvm_function_type, fn_ptr, &args, "")
                .unwrap()
                .try_as_basic_value()
                .left()
        } else {
            // Capturing closure - call with environment as first parameter
            let closure_fn_type = ctx.create_closure_impl_fn_type(&return_value, &params);
            let fn_ptr = ctx.extract_closure_fn_ptr(closure_struct, closure_fn_type);

            // Add environment as first argument
            let mut closure_args = vec![env_ptr.into()];
            closure_args.extend(args);

            ctx.builder
                .build_indirect_call(closure_fn_type, fn_ptr, &closure_args, "")
                .unwrap()
                .try_as_basic_value()
                .left()
        }
    }
}
