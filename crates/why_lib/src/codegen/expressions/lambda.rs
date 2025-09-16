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
    /// Generate code for a non-capturing lambda (simple function pointer wrapped as closure)
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

    /// Generate code for a capturing lambda (closure with environment)
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

    /// Generate the lambda function body
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

    /// Create environment struct and populate with captured values
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
        for (i, (var_name, _var_type)) in capture_info.captures.iter().enumerate() {
            let captured_value = ctx.find_variable(var_name);
            let field_ptr = ctx
                .builder
                .build_struct_gep(
                    env_struct_type,
                    env_ptr,
                    i as u32,
                    &format!("env_field_{}", i),
                )
                .unwrap();

            ctx.builder.build_store(field_ptr, captured_value).unwrap();
        }

        // Return struct type and pointer as i8*
        (env_struct_type, env_ptr_i8)
    }
}
