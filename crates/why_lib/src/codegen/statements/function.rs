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
            return_type,
            statements,
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
            unreachable!()
        };

        let llvm_return_type = ctx.get_llvm_type(return_value);

        let llvm_param_types = params
            .iter()
            .map(|param_type| ctx.get_llvm_type(param_type))
            .collect::<Vec<_>>();

        let llvm_fn_type = get_function_type(&llvm_return_type, &llvm_param_types);

        // get function value and store it in the scope (such that it can be referenced later)
        let llvm_fn_value = ctx.module.add_function(&id.name, llvm_fn_type, None);
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

        for statement in statements {
            statement.codegen(ctx);
        }

        ctx.exit_scope();
    }
}

fn get_function_type<'ctx>(
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
