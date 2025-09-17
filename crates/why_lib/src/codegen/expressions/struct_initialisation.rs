//! # Struct Initialization Code Generation
//!
//! This module implements LLVM code generation for struct initialization expressions in Y-lang.
//! It handles creating struct instances with proper field initialization and memory layout.
//!
//! ## Struct Construction Process
//!
//! 1. **Type lookup**: Retrieve LLVM struct type from the type cache
//! 2. **Stack allocation**: Allocate memory for the struct instance
//! 3. **Field initialization**: Generate code for each field value
//! 4. **Field assignment**: Use GEP operations to store values in correct positions
//! 5. **Value production**: Load and return the complete struct value
//!
//! ## Memory Management
//!
//! - **Stack allocation**: Struct instances are allocated on the stack using `alloca`
//! - **Field layout**: Fields are stored in the order they appear in initialization
//! - **GEP operations**: Used for safe field pointer calculation
//! - **Value semantics**: Structs are returned by value, not by reference
//!
//! ## Field Index Resolution
//!
//! The implementation uses the order of fields in the initialization expression
//! to determine field indices. This matches the declaration order in the struct definition.

use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::{StructFieldInitialisation, StructInitialisation},
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for StructInitialisation<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    /// Generates LLVM IR for struct initialization expressions.
    ///
    /// Creates a new struct instance by allocating memory and initializing
    /// each field with the provided values. The struct is constructed on the
    /// stack and returned by value.
    ///
    /// ## Implementation Steps
    ///
    /// 1. **Type resolution**: Look up the LLVM struct type in the type cache
    /// 2. **Memory allocation**: Allocate struct storage on the stack
    /// 3. **Field processing**: Generate code for each field initialization
    /// 4. **Field assignment**: Use GEP to compute field addresses and store values
    /// 5. **Value return**: Load complete struct value for return
    ///
    /// ## GEP Operations
    ///
    /// Field addresses are computed using LLVM's GetElementPtr with:
    /// - Base pointer to the allocated struct
    /// - Index [0, field_index] for struct field access
    /// - Type-safe field offset calculation
    ///
    /// # Returns
    ///
    /// `Some(BasicValueEnum)` containing the initialized struct value
    ///
    /// # Panics
    ///
    /// - Struct type not found in type cache
    /// - Field value generation fails
    /// - LLVM instruction building failures
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        let StructInitialisation {
            id, fields, info, ..
        } = self;

        // Extract struct name for LLVM naming and error messages
        let struct_name = &id.name;

        // Retrieve the LLVM struct type from the type cache
        let struct_type = {
            let types_guard = ctx.types.borrow();
            match types_guard.get(&info.type_id) {
                Some(llvm_type) => {
                    if let inkwell::types::BasicMetadataTypeEnum::StructType(struct_type) =
                        llvm_type
                    {
                        *struct_type
                    } else {
                        panic!(
                            "Expected struct type for {}, got: {:?}",
                            struct_name, llvm_type
                        )
                    }
                }
                None => {
                    panic!("Struct type {} not found in type context", struct_name);
                }
            }
        };

        // Allocate stack memory for the struct instance
        // Using alloca ensures proper cleanup when scope exits
        let struct_ptr = ctx.builder.build_alloca(struct_type, struct_name).unwrap();

        // Initialize each field by generating code for values and storing them
        for field_init in fields {
            let StructFieldInitialisation { name, value, .. } = field_init;
            let field_name = &name.name;

            // Generate LLVM IR for the field's initialization value
            let Some(field_value) = value.codegen(ctx) else {
                panic!(
                    "Failed to generate code for field {} in struct {}",
                    field_name, struct_name
                );
            };

            // Calculate the address of this field using GEP (GetElementPtr)
            // Uses [0, field_index] indices for struct field access
            let field_ptr = unsafe {
                ctx.builder
                    .build_gep(
                        struct_type,
                        struct_ptr,
                        &[
                            ctx.context.i32_type().const_zero(), // Struct base offset
                            ctx.context.i32_type().const_int(
                                // Field offset
                                self.get_field_index(struct_name, field_name) as u64,
                                false,
                            ),
                        ],
                        &format!("{}_{}", struct_name, field_name),
                    )
                    .unwrap()
            };

            // Store the field value at the computed address
            ctx.builder.build_store(field_ptr, field_value).unwrap();
        }

        // Load and return the complete struct value (value semantics)
        Some(
            ctx.builder
                .build_load(struct_type, struct_ptr, struct_name)
                .unwrap(),
        )
    }
}

impl StructInitialisation<ValidatedTypeInformation> {
    /// Determines the LLVM field index for a named struct field.
    ///
    /// This method maps Y-lang field names to LLVM struct field indices
    /// for use in GEP operations. The current implementation uses the
    /// order of fields in the initialization expression.
    ///
    /// ## Field Index Resolution Strategy
    ///
    /// Currently uses a simplified approach where field indices are determined
    /// by the order of fields in the initialization expression. This assumes
    /// that initialization order matches declaration order.
    ///
    /// ## Future Improvements
    ///
    /// A more robust implementation would:
    /// - Store field layout information in the context during struct declaration
    /// - Handle partial initialization with default values
    /// - Support field reordering and optional fields
    ///
    /// # Parameters
    ///
    /// * `struct_name` - Name of the struct (for error messages)
    /// * `field_name` - Name of the field to find
    ///
    /// # Returns
    ///
    /// The zero-based field index for use in LLVM GEP operations
    ///
    /// # Panics
    ///
    /// Panics if the field name is not found in the initialization list
    fn get_field_index(&self, struct_name: &str, field_name: &str) -> u32 {
        // Linear search through initialization fields to find the index
        // This is O(n) but acceptable for typical struct sizes
        for (i, field) in self.fields.iter().enumerate() {
            if field.name.name == field_name {
                return i as u32;
            }
        }

        // Field not found - this indicates a programming error
        panic!("Field {} not found in struct {}", field_name, struct_name);
    }
}
