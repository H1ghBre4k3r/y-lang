mod ast_string;
mod binary;
mod block;
mod bool;
mod character;
mod id;
mod if_expression;
mod lambda;
mod num;
mod postfix;
mod prefix;
mod struct_initialisation;

use inkwell::{types::BasicType, values::BasicValueEnum};
use crate::typechecker::{ValidatedTypeInformation, Type};

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
                                let array_alloca = ctx.builder.build_alloca(array_type, "empty_array").unwrap();

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
