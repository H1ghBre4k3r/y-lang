mod expressions;
mod statements;

use std::{cell::RefCell, collections::HashMap};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, FunctionType},
};

use crate::{
    parser::ast::{Block, Function, Statement},
    typechecker::{Type, ValidatedTypeInformation},
};

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub types: RefCell<HashMap<Type, BasicMetadataTypeEnum<'ctx>>>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn get_llvm_type(&self, our_type: &Type) -> BasicMetadataTypeEnum<'ctx> {
        let mut types = self.types.borrow_mut();
        if let Some(entry) = types.get(our_type) {
            return *entry;
        }

        let new_type = our_type.to_llvm_type(self.context);
        types.insert(our_type.clone(), new_type);
        new_type
    }
}

pub trait CodeGen<'ctx> {
    type ReturnValue;
    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue;
}

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

        let llvm_params = params
            .iter()
            .map(|param_type| ctx.get_llvm_type(param_type))
            .collect::<Vec<_>>();

        let llvm_fn_type = get_function_type(&llvm_return_type, &llvm_params);

        let llvm_fn = ctx.module.add_function(&id.name, llvm_fn_type, None);
        let llvm_fn_bb = ctx.context.append_basic_block(llvm_fn, "entry");
        ctx.builder.position_at_end(llvm_fn_bb);

        for statement in statements {
            statement.codegen(ctx);
        }
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

pub trait IntoLLVMType {
    fn to_llvm_type<'ctx>(&self, ctx: &'ctx Context) -> BasicMetadataTypeEnum<'ctx>;
}

impl IntoLLVMType for Type {
    fn to_llvm_type<'ctx>(&self, ctx: &'ctx Context) -> BasicMetadataTypeEnum<'ctx> {
        match self {
            Type::Integer => ctx.i64_type().into(),
            Type::FloatingPoint => todo!(),
            Type::Boolean => todo!(),
            Type::Character => todo!(),
            Type::String => todo!(),
            Type::Void => todo!(),
            Type::Unknown => todo!(),
            Type::Reference(_) => todo!(),
            Type::Tuple(items) => todo!(),
            Type::Array(_) => todo!(),
            Type::Struct(_, items) => todo!(),
            Type::Function {
                params,
                return_value,
            } => todo!(),
        }
    }
}
