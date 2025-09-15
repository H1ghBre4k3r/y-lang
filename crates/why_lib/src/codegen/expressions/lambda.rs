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
            info,
            ..
        } = self;

        // Extract function/closure info
        let (params, return_value, captures) = match &info.type_id {
            Type::Function { params, return_value } => (params, return_value, Vec::new()),
            Type::Closure { params, return_value, captures } => (params, return_value, captures.clone()),
            _ => panic!("Lambda should have function or closure type"),
        };

        // Generate a unique name for the lambda function
        let lambda_name = format!("lambda_{}", *ctx.lambda_counter.borrow());
        *ctx.lambda_counter.borrow_mut() += 1;

        // Build the LLVM function type
        let llvm_fn_type = if captures.is_empty() {
            // Non-capturing lambda: regular function type
            build_llvm_function_type_from_own_types(ctx, return_value, params)
        } else {
            // Capturing lambda: add environment pointer as first parameter
            let mut modified_params = vec![Type::Reference(Box::new(Type::Void))]; // Environment pointer
            modified_params.extend(params.iter().cloned());
            build_llvm_function_type_from_own_types(ctx, return_value, &modified_params)
        };

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
        let param_offset = if captures.is_empty() { 0 } else { 1 }; // Skip environment parameter for closures

        // For capturing lambdas, extract captured variables from environment
        if !captures.is_empty() {
            let environment_param = lambda_fn
                .get_nth_param(0)
                .expect("Environment parameter should exist")
                .into_pointer_value();

            // Generate environment struct type for this capture set
            let env_struct_type = ctx.get_environment_type(&captures);

            // Extract each captured variable from the environment
            for (field_index, (capture_name, _capture_type)) in captures.iter().enumerate() {
                let field_ptr = unsafe {
                    ctx.builder.build_gep(
                        env_struct_type,
                        environment_param,
                        &[
                            ctx.context.i32_type().const_zero(),
                            ctx.context.i32_type().const_int(field_index as u64, false),
                        ],
                        &format!("capture_{}", capture_name),
                    ).unwrap()
                };

                let field_type = env_struct_type
                    .get_field_type_at_index(field_index as u32)
                    .expect("Field type should exist");

                let captured_value = ctx.builder
                    .build_load(field_type, field_ptr, capture_name)
                    .unwrap();

                ctx.store_variable(capture_name, captured_value);
            }
        }

        // Set up declared parameters
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

        // Return based on whether this is a capturing closure or non-capturing lambda
        if captures.is_empty() {
            // Non-capturing lambda: return function pointer
            Some(lambda_fn.as_global_value().as_pointer_value().into())
        } else {
            // Capturing closure: create environment and closure struct

            // Allocate environment struct on the heap
            let env_struct_type = ctx.get_environment_type(&captures);
            let env_ptr = ctx.heap_allocate_environment(env_struct_type).into_pointer_value();

            // Populate environment with captured values
            for (field_index, (capture_name, _capture_type)) in captures.iter().enumerate() {
                // Get the current value of the captured variable
                let captured_value = ctx.find_variable(capture_name);

                // Get pointer to the field in the environment struct
                let field_ptr = unsafe {
                    ctx.builder.build_gep(
                        env_struct_type,
                        env_ptr,
                        &[
                            ctx.context.i32_type().const_zero(),
                            ctx.context.i32_type().const_int(field_index as u64, false),
                        ],
                        &format!("env_field_{}", capture_name),
                    ).unwrap()
                };

                // Store the captured value in the environment
                ctx.builder.build_store(field_ptr, captured_value).unwrap();
            }

            // Create closure struct {function_ptr, environment_ptr}
            let closure_struct_type = ctx.get_closure_struct_type();
            let closure_alloca = ctx.builder
                .build_alloca(closure_struct_type, "closure_struct")
                .unwrap();

            // Store function pointer in closure struct (field 0)
            let func_ptr_field = unsafe {
                ctx.builder.build_gep(
                    closure_struct_type,
                    closure_alloca,
                    &[
                        ctx.context.i32_type().const_zero(),
                        ctx.context.i32_type().const_zero(),
                    ],
                    "closure_func_ptr",
                ).unwrap()
            };
            let func_ptr = lambda_fn.as_global_value().as_pointer_value();
            ctx.builder.build_store(func_ptr_field, func_ptr).unwrap();

            // Store environment pointer in closure struct (field 1)
            let env_ptr_field = unsafe {
                ctx.builder.build_gep(
                    closure_struct_type,
                    closure_alloca,
                    &[
                        ctx.context.i32_type().const_zero(),
                        ctx.context.i32_type().const_int(1, false),
                    ],
                    "closure_env_ptr",
                ).unwrap()
            };
            ctx.builder.build_store(env_ptr_field, env_ptr).unwrap();

            // Return pointer to closure struct (to match function pointer return type)
            Some(closure_alloca.into())
        }
    }
}
