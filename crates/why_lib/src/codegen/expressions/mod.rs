//! # Expression Code Generation
//!
//! This module implements LLVM code generation for all expression types in Y-lang.
//! Expressions are the value-producing constructs of the language, ranging from
//! simple literals to complex control flow and function calls.
//!
//! ## Expression System Architecture
//!
//! Y-lang expressions follow a unified design where every expression either produces
//! an LLVM value or explicitly returns `None` for void operations. This enables
//! Y-lang's expression-oriented programming style where most constructs can produce values.
//!
//! ### Expression Categories
//!
//! #### Literal Expressions
//! Simple value expressions that produce compile-time or runtime constants:
//! - **Numeric literals** (`num.rs`): Integer and floating-point constants
//! - **Boolean literals** (`bool.rs`): True/false values as i1 constants
//! - **Character literals** (`character.rs`): ASCII/UTF-8 character codes
//! - **String literals** (`ast_string.rs`): Global string constants
//!
//! #### Variable and Identifier Expressions
//! - **Identifiers** (`id.rs`): Variable and function name resolution
//!
//! #### Arithmetic and Logic Expressions
//! - **Binary expressions** (`binary.rs`): Arithmetic, comparison, and logical operations
//! - **Prefix expressions** (`prefix.rs`): Unary operations like negation
//!
//! #### Control Flow Expressions
//! - **If expressions** (`if_expression.rs`): Conditional expressions with phi nodes
//! - **Block expressions** (`block.rs`): Scoped expression sequences
//!
//! #### Function and Method Expressions
//! - **Lambda expressions** (`lambda.rs`): Anonymous function creation with closures
//! - **Postfix expressions** (`postfix.rs`): Function calls, method calls, array indexing
//!
//! #### Composite Data Expressions
//! - **Struct initialization** (`struct_initialisation.rs`): Struct literal creation
//! - **Array literals**: Stack-allocated array creation with element initialization
//!
//! ## Code Generation Strategy
//!
//! ### Value Production Model
//! All expressions implement the `CodeGen` trait with `ReturnValue = Option<BasicValueEnum>`:
//! - **`Some(value)`**: Expression produces an LLVM value
//! - **`None`**: Expression performs side effects without producing a value
//!
//! ### Type System Integration
//! Expression code generation relies heavily on validated type information:
//! - **Type-driven generation**: Uses `ValidatedTypeInformation` for correct LLVM types
//! - **LLVM type mapping**: Converts Y-lang types to appropriate LLVM representations
//! - **Type safety**: Ensures generated code maintains type correctness
//!
//! ### Memory Management
//! Expressions follow consistent memory management patterns:
//! - **Stack allocation**: Local values allocated on the stack using `alloca`
//! - **Value semantics**: Most expressions return values by value, not reference
//! - **Automatic cleanup**: LLVM handles memory cleanup for stack-allocated values
//!
//! ## LLVM Integration Patterns
//!
//! ### Instruction Generation
//! Each expression type generates appropriate LLVM instructions:
//! - **Arithmetic**: Uses LLVM arithmetic instructions (add, sub, mul, etc.)
//! - **Comparisons**: Uses LLVM comparison instructions (icmp, fcmp)
//! - **Memory access**: Uses load/store instructions for variable access
//! - **Control flow**: Uses basic blocks and branch instructions
//!
//! ### Basic Block Management
//! Complex expressions manage LLVM basic blocks:
//! - **Linear expressions**: Use single basic block for straightforward evaluation
//! - **Control flow**: Create multiple basic blocks with proper termination
//! - **Phi nodes**: Merge values from different control flow paths
//!
//! ### Value Conversion
//! The module handles conversion between Y-lang and LLVM value representations:
//! - **Constant folding**: Compile-time constants become LLVM constants
//! - **Type conversions**: Automatic conversion between compatible types
//! - **Pointer handling**: Proper pointer/value distinctions for memory operations
//!
//! ## Expression Composition
//!
//! ### Recursive Evaluation
//! Complex expressions are built from simpler expressions:
//! - **Tree traversal**: Expression AST is traversed recursively
//! - **Bottom-up generation**: Subexpressions evaluated before parent expressions
//! - **Value propagation**: Values flow up the expression tree
//!
//! ### Side Effect Management
//! The system carefully manages expression side effects:
//! - **Pure expressions**: Literals and arithmetic have no side effects
//! - **Impure expressions**: Function calls and variable access may have side effects
//! - **Ordering preservation**: Side effects occur in evaluation order
//!
//! ## Error Handling Philosophy
//!
//! ### Fail-fast Strategy
//! Expression generation uses fail-fast error handling:
//! - **`unreachable!()` macros**: For conditions that should never occur after type checking
//! - **`expect()` calls**: For operations that should succeed with valid types
//! - **Panic messages**: Provide context for debugging compiler issues
//!
//! ### Type Safety Guarantees
//! The type checker ensures that expression generation will succeed:
//! - **Pre-validated types**: All expressions have validated type information
//! - **Type compatibility**: Operations only performed on compatible types
//! - **Memory safety**: No undefined behavior in generated LLVM code
//!
//! ## Performance Considerations
//!
//! ### Code Generation Efficiency
//! The expression system is designed for efficient code generation:
//! - **Single-pass generation**: Most expressions generated in a single traversal
//! - **Minimal allocations**: Reuses LLVM builder and context efficiently
//! - **Direct translation**: Y-lang expressions map directly to LLVM patterns
//!
//! ### Runtime Performance
//! Generated code is optimized for runtime performance:
//! - **LLVM optimizations**: Generated IR is optimization-friendly
//! - **Efficient patterns**: Uses well-known LLVM optimization patterns
//! - **Minimal overhead**: Direct mapping without unnecessary indirection
//!
//! ## Extension Points
//!
//! The expression system is designed for extensibility:
//! - **New expression types**: Can be added by implementing `CodeGen`
//! - **Custom operators**: Binary and prefix systems support new operators
//! - **Type-specific handling**: Each expression type can have specialized logic

pub mod ast_string;
pub mod binary;
pub mod block;
pub mod bool;
pub mod character;
pub mod id;
pub mod if_expression;
pub mod lambda;
pub mod num;
pub mod postfix;
pub mod prefix;
pub mod struct_initialisation;

use crate::typechecker::{Type, ValidatedTypeInformation};
use inkwell::{types::BasicType, values::BasicValueEnum};

use crate::parser::ast::Expression;

use super::CodeGen;

impl<'ctx> CodeGen<'ctx> for Expression<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    fn codegen(&self, ctx: &super::CodegenContext<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        match self {
            Expression::Id(id) => Some(id.codegen(ctx)),
            Expression::Num(num) => Some(num.codegen(ctx)),
            Expression::Bool(bool) => Some(bool.codegen(ctx)),
            Expression::Character(character) => Some(character.codegen(ctx)),
            Expression::AstString(ast_string) => Some(ast_string.codegen(ctx)),
            Expression::Function(function) => todo!(),
            Expression::Lambda(lambda) => lambda.codegen(ctx),
            Expression::If(if_expr) => if_expr.codegen(ctx),
            Expression::Block(block) => block.codegen(ctx),
            Expression::Parens(expression) => expression.codegen(ctx),
            Expression::Postfix(postfix) => postfix.codegen(ctx),
            Expression::Prefix(prefix) => Some(prefix.codegen(ctx)),
            Expression::Binary(binary_expression) => Some(binary_expression.codegen(ctx)),
            Expression::Array(array) => {
                match array {
                    crate::parser::ast::Array::Literal { values, info, .. } => {
                        // For now, create a stack-allocated array
                        if values.is_empty() {
                            // Handle empty arrays: get element type from type information
                            let ValidatedTypeInformation { type_id, .. } = info;
                            if let Type::Array(element_type) = type_id {
                                let llvm_element_type = ctx.get_llvm_type(element_type);
                                let element_basic_type =
                                    super::convert_metadata_to_basic(llvm_element_type)
                                        .expect("Array element type must be basic");

                                // Create zero-length array type
                                let array_type = element_basic_type.array_type(0);

                                // Allocate array on stack
                                let array_alloca =
                                    ctx.builder.build_alloca(array_type, "empty_array").unwrap();

                                return Some(array_alloca.into());
                            } else {
                                // If we don't have proper type information, we can't create the array
                                return None;
                            }
                        }

                        // Get the element type from the first element
                        let first_element_type = values[0].get_info().type_id;
                        let llvm_element_type = ctx.get_llvm_type(&first_element_type);
                        let element_basic_type =
                            super::convert_metadata_to_basic(llvm_element_type)
                                .expect("Array element type must be basic");

                        // Create array type
                        let array_type = element_basic_type.array_type(values.len() as u32);

                        // Allocate array on stack
                        let array_alloca = ctx.builder.build_alloca(array_type, "array").unwrap();

                        // Initialize each element
                        for (i, value) in values.iter().enumerate() {
                            let Some(llvm_value) = value.codegen(ctx) else {
                                continue;
                            };

                            // Get pointer to array element
                            let element_ptr = unsafe {
                                ctx.builder
                                    .build_gep(
                                        array_type,
                                        array_alloca,
                                        &[
                                            ctx.context.i32_type().const_zero(),
                                            ctx.context.i32_type().const_int(i as u64, false),
                                        ],
                                        &format!("array_elem_{}", i),
                                    )
                                    .unwrap()
                            };

                            // Store the value
                            ctx.builder.build_store(element_ptr, llvm_value).unwrap();
                        }

                        Some(array_alloca.into())
                    }
                    crate::parser::ast::Array::Default {
                        initial_value,
                        length,
                        ..
                    } => {
                        // TODO: Implement default arrays (&[value; length])
                        todo!("Default array initialization not yet implemented")
                    }
                }
            }
            Expression::StructInitialisation(struct_initialisation) => {
                struct_initialisation.codegen(ctx)
            }
        }
    }
}
