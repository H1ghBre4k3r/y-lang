use inkwell::values::PointerValue;

use crate::{
    codegen::{CodeGen, convert_metadata_to_basic},
    parser::ast::{Lambda, LambdaParameter},
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Lambda<ValidatedTypeInformation> {
    type ReturnValue = PointerValue<'ctx>;

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
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
                    context,
                },
            position,
        } = self
        else {
            unreachable!("Lambda should have function type during code generation: {self:#?}")
        };

        // Store current builder position
        let current_bb = ctx.builder.get_insert_block();

        let llvm_lambda_value = ctx.create_lambda(return_value, params);

        let llvm_lambda_bb = ctx.context.append_basic_block(llvm_lambda_value, "entry");
        ctx.builder.position_at_end(llvm_lambda_bb);

        ctx.enter_scope();
        for (i, param) in parameters.iter().enumerate() {
            let LambdaParameter { name, .. } = param;

            let llvm_param_value = llvm_lambda_value
                .get_nth_param(i as u32)
                .expect("There should be this parameter");

            // Create alloca for parameter to make it consistent with local variables
            let param_type = &params[i];
            let llvm_param_type = ctx.get_llvm_type(param_type);
            let Some(basic_type) = convert_metadata_to_basic(llvm_param_type) else {
                // For non-basic types, store parameter directly (fallback)
                ctx.store_variable(&name.name, llvm_param_value);
                continue;
            };

            let llvm_alloca = ctx
                .builder
                .build_alloca(basic_type, &name.name)
                .expect("build_alloca failed for parameter");

            if let Err(e) = ctx.builder.build_store(llvm_alloca, llvm_param_value) {
                panic!("Failed to store parameter value: {e}");
            }

            ctx.store_variable(&name.name, llvm_alloca.into());
        }

        // Generate lambda body
        let lambda_result = expression.codegen(ctx);

        // Only add return instruction if the basic block isn't already terminated
        let lambda_bb = ctx.builder.get_insert_block().unwrap();
        if lambda_bb.get_terminator().is_none() {
            // No terminator means we need to add a return instruction
            match return_value.as_ref() {
                Type::Void => {
                    // Void functions need explicit 'ret void' instruction
                    ctx.builder.build_return(None).unwrap();
                }
                _ => {
                    // Non-void functions should return the block result
                    if let Some(return_value) = lambda_result {
                        ctx.builder.build_return(Some(&return_value)).unwrap();
                    } else {
                        // If no value was produced, this is a function that should have
                        // an explicit return statement. This is likely a type checker issue.
                        panic!(
                            "Non-void function reached end without explicit return or yielding expression"
                        );
                    }
                }
            }
        }

        ctx.exit_scope();

        // Restore builder position
        if let Some(bb) = current_bb {
            ctx.builder.position_at_end(bb);
        }

        llvm_lambda_value.as_global_value().as_pointer_value()
    }
}
