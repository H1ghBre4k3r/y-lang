use inkwell::types::{BasicMetadataTypeEnum, FunctionType};

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::{Function, FunctionParameter},
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Function<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) {
        let Function {
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
        } = self
        else {
            unreachable!()
        };

        // Special handling for void main function
        let (actual_fn_name, create_main_wrapper) =
            if id.name == "main" && **return_value == Type::Void {
                ("y_main", true)
            } else {
                (id.name.as_str(), false)
            };

        let llvm_fn_type = build_llvm_function_type_from_own_types(ctx, return_value, params);

        // get function value and store it in the scope (such that it can be referenced later)
        let llvm_fn_value = ctx.module.add_function(actual_fn_name, llvm_fn_type, None);
        ctx.store_function(&id.name, llvm_fn_value);

        // enter scope for function parameters and local variables
        ctx.enter_scope();
        for (i, param) in parameters.iter().enumerate() {
            let FunctionParameter { name, .. } = param;

            let llvm_param_value = llvm_fn_value
                .get_nth_param(i as u32)
                .expect("There should be this parameter");

            ctx.store_variable(&name.name, llvm_param_value);
        }

        let llvm_fn_bb = ctx.context.append_basic_block(llvm_fn_value, "entry");
        ctx.builder.position_at_end(llvm_fn_bb);

        // Delegate to unified block code generation
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
                    // Non-void function without explicit return is an error, but we'll add unreachable
                    ctx.builder.build_unreachable().unwrap();
                }
            }
        }

        // Create main wrapper if needed
        if create_main_wrapper {
            let main_fn_type = ctx.context.i32_type().fn_type(&[], false);
            let main_fn = ctx.module.add_function("main", main_fn_type, None);
            let main_bb = ctx.context.append_basic_block(main_fn, "entry");

            // Store current builder position
            let current_bb = ctx.builder.get_insert_block();

            // Build the wrapper
            ctx.builder.position_at_end(main_bb);
            ctx.builder.build_call(llvm_fn_value, &[], "").unwrap();
            ctx.builder
                .build_return(Some(&ctx.context.i32_type().const_int(0, false)))
                .unwrap();

            // Restore builder position if it existed
            if let Some(bb) = current_bb {
                ctx.builder.position_at_end(bb);
            }
        }

        ctx.exit_scope();
    }
}

pub fn build_llvm_function_type_from_own_types<'ctx>(
    ctx: &CodegenContext<'ctx>,
    return_type: &Type,
    param_types: &[Type],
) -> FunctionType<'ctx> {
    let llvm_param_types = param_types
        .iter()
        .map(|param_type| ctx.get_llvm_type(param_type))
        .collect::<Vec<_>>();

    match return_type {
        Type::Boolean => {
            let llvm_bool_type = ctx.context.bool_type();
            llvm_bool_type.fn_type(&llvm_param_types, false)
        }
        Type::Character => {
            let llvm_char_type = ctx.context.i8_type();
            llvm_char_type.fn_type(&llvm_param_types, false)
        }
        Type::String => {
            // String is represented as a pointer to i8
            let llvm_string_type = ctx.context.ptr_type(Default::default());
            llvm_string_type.fn_type(&llvm_param_types, false)
        }
        Type::Void => {
            let llvm_void_type = ctx.context.void_type();
            llvm_void_type.fn_type(&llvm_param_types, false)
        }
        Type::Unknown => todo!(),
        Type::Function {
            params: fn_params,
            return_value: fn_return_value,
        } => {
            // Function returning another function - return function pointer
            ctx.context
                .ptr_type(Default::default())
                .fn_type(&llvm_param_types, false)
        }
        return_type => {
            let llvm_return_type = ctx.get_llvm_type(return_type);

            build_llvm_function_type_from_llvm_types(&llvm_return_type, &llvm_param_types)
        }
    }
}

fn build_llvm_function_type_from_llvm_types<'ctx>(
    llvm_type: &BasicMetadataTypeEnum<'ctx>,
    llvm_params: &[BasicMetadataTypeEnum<'ctx>],
) -> FunctionType<'ctx> {
    match llvm_type {
        BasicMetadataTypeEnum::ArrayType(array_type) => array_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::FloatType(float_type) => float_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::IntType(int_type) => int_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::PointerType(pointer_type) => {
            pointer_type.fn_type(llvm_params, false)
        }
        BasicMetadataTypeEnum::StructType(struct_type) => struct_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::VectorType(vector_type) => vector_type.fn_type(llvm_params, false),
        BasicMetadataTypeEnum::ScalableVectorType(scalable_vector_type) => {
            scalable_vector_type.fn_type(llvm_params, false)
        }
        BasicMetadataTypeEnum::MetadataType(metadata_type) => {
            metadata_type.fn_type(llvm_params, false)
        }
    }
}
