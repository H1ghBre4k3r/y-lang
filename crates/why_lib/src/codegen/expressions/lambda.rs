use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{
        statements::function::build_llvm_function_type_from_own_types, CodeGen, CodegenContext,
    },
    parser::ast::{Lambda, LambdaParameter},
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Lambda<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

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
            ..
        } = self
        else {
            panic!("Lambda should have function type");
        };

        // Generate a unique name for the lambda function
        let lambda_name = format!("lambda_{}", *ctx.lambda_counter.borrow());
        *ctx.lambda_counter.borrow_mut() += 1;

        // Build the LLVM function type
        let llvm_fn_type = build_llvm_function_type_from_own_types(ctx, return_value, params);

        // Create the lambda function
        let lambda_fn = ctx.module.add_function(&lambda_name, llvm_fn_type, None);

        // Store lambda in scope so it can be called later
        ctx.store_lambda(&lambda_name, lambda_fn);

        // Create the entry basic block
        let entry_bb = ctx.context.append_basic_block(lambda_fn, "entry");

        // Store current builder position
        let current_bb = ctx.builder.get_insert_block();

        // Position builder at the entry block
        ctx.builder.position_at_end(entry_bb);

        // Enter scope for lambda body execution
        ctx.enter_scope();

        // Set up parameters in the lambda scope
        for (i, param) in parameters.iter().enumerate() {
            let LambdaParameter { name, .. } = param;
            let llvm_param_value = lambda_fn
                .get_nth_param(i as u32)
                .expect("Lambda parameter should exist");

            ctx.store_variable(&name.name, llvm_param_value);
        }

        // Generate code for the lambda body
        let result = expression.codegen(ctx);

        // Add return instruction based on return type
        match return_value.as_ref() {
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

        // Return the lambda function as a function pointer
        Some(lambda_fn.as_global_value().as_pointer_value().into())
    }
}

