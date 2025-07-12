use inkwell::types::{BasicMetadataTypeEnum, FunctionType};

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::Function,
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

        let llvm_fn = ctx.module.add_function(&id.name, llvm_fn_type, None);
        let llvm_fn_bb = ctx.context.append_basic_block(llvm_fn, "entry");
        ctx.builder.position_at_end(llvm_fn_bb);

        ctx.enter_scope();

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
