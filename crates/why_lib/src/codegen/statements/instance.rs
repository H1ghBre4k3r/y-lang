use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::{FunctionParameter, Instance, TypeName},
    typechecker::{Type, ValidatedTypeInformation},
};

use super::function::build_llvm_function_type_from_own_types;

impl<'ctx> CodeGen<'ctx> for Instance<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let Instance {
            name,
            functions,
            declarations,
            ..
        } = self;

        // Instance methods are methods associated with a specific type
        // For each function in the instance block, we need to:
        // 1. Generate the LLVM function with an implicit 'this' parameter
        // 2. Register the function in a way that it can be called on instances of the type

        let type_name = match name {
            TypeName::Literal(name, _) => name,
            _ => {
                panic!(
                    "Instance blocks can only be applied to literal type names, not complex types"
                );
            }
        };

        // Process function implementations
        for function in functions {
            // Generate the function with 'this' parameter injection
            // The function name will be something like "TypeName_methodName"
            let method_name = format!("{}_{}", type_name, function.id.name);

            // Compile instance method with 'this' parameter injection
            self.compile_instance_method(ctx, function, type_name, &method_name);
        }

        // Process method declarations (external methods)
        for _declaration in declarations {
            // Method declarations are just forward declarations
            // They don't need implementation here - they're handled elsewhere
            // TODO: Implement method declaration registration
        }
    }
}

impl<'ctx> Instance<ValidatedTypeInformation> {
    fn compile_instance_method(
        &self,
        ctx: &CodegenContext<'ctx>,
        function: &crate::parser::ast::Function<ValidatedTypeInformation>,
        struct_type_name: &str,
        method_name: &str,
    ) {
        // Extract function information
        let crate::parser::ast::Function {
            id,
            parameters,
            body,
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
        } = function
        else {
            unreachable!("Instance method should have function type")
        };

        // Create 'this' parameter type - get actual struct type from type system
        let struct_type = self
            .find_struct_type(ctx, struct_type_name)
            .unwrap_or_else(|| {
                panic!(
                    "Struct type {} not found in type system for instance method",
                    struct_type_name
                )
            });
        let this_type = Type::Reference(Box::new(struct_type));

        // Build modified parameter list with 'this' as first parameter
        let mut method_params = vec![this_type];
        method_params.extend(params.clone());

        // Build LLVM function type with 'this' parameter
        let llvm_fn_type =
            build_llvm_function_type_from_own_types(ctx, return_value, &method_params);

        // Create LLVM function with modified name
        let llvm_fn_value = ctx.module.add_function(method_name, llvm_fn_type, None);

        // Store the function in scope (for potential recursive calls)
        ctx.store_function(&id.name, llvm_fn_value);

        // Enter scope for method parameters and local variables
        ctx.enter_scope();

        // Inject 'this' parameter as first parameter
        let this_param_value = llvm_fn_value
            .get_nth_param(0)
            .expect("'this' parameter should exist");
        ctx.store_variable("this", this_param_value);

        // Handle regular function parameters (starting from index 1)
        for (i, param) in parameters.iter().enumerate() {
            let FunctionParameter { name, .. } = param;

            let llvm_param_value = llvm_fn_value
                .get_nth_param((i + 1) as u32) // +1 because 'this' is at index 0
                .expect("Function parameter should exist");

            ctx.store_variable(&name.name, llvm_param_value);
        }

        // Create basic block and set insertion point
        let llvm_fn_bb = ctx.context.append_basic_block(llvm_fn_value, "entry");
        ctx.builder.position_at_end(llvm_fn_bb);

        // Compile method body
        body.codegen(ctx);

        // Add terminator instruction if the basic block doesn't have one
        if ctx
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            match return_value.as_ref() {
                Type::Void => {
                    ctx.builder.build_return(None).unwrap();
                }
                _ => {
                    // Non-void function without explicit return - add unreachable
                    ctx.builder.build_unreachable().unwrap();
                }
            }
        }

        // Exit method scope
        ctx.exit_scope();
    }

    fn find_struct_type(&self, ctx: &CodegenContext<'ctx>, struct_name: &str) -> Option<Type> {
        // Look up the struct type from the types registry
        // Iterate through registered types to find one that matches the struct name
        for (type_key, _llvm_type) in ctx.types.borrow().iter() {
            if let Type::Struct(name, fields) = type_key {
                if name == struct_name {
                    return Some(Type::Struct(name.clone(), fields.clone()));
                }
            }
        }
        None
    }
}
