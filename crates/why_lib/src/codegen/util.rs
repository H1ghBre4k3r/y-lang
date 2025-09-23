use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};

use crate::{codegen::CodegenContext, typechecker::Type};

pub fn convert_our_type_to_llvm_basic_metadata_type<'ctx>(
    our_type: &Type,
    ctx: &CodegenContext<'ctx>,
) -> BasicMetadataTypeEnum<'ctx> {
    match our_type {
        Type::Integer => ctx.context.i64_type().into(),
        Type::FloatingPoint => ctx.context.f64_type().into(),
        Type::Boolean => ctx.context.bool_type().into(),
        Type::Character => ctx.context.i8_type().into(), // UTF-8 char representation
        Type::String => {
            // Represent strings as pointer to i8 (C-style strings or slices)
            ctx.context.ptr_type(Default::default()).into()
        }
        Type::Void => {
            // Void isn't a valid BasicMetadataTypeEnum — can return pointer or dummy
            panic!("Void cannot be used as a BasicMetadataTypeEnum")
        }
        Type::Unknown => {
            panic!("Cannot convert unknown type to LLVM")
        }
        Type::Reference(_) => ctx.context.ptr_type(Default::default()).into(),
        Type::Tuple(items) => {
            let types: Vec<_> = items
                .iter()
                .map(|item_type| {
                    // TODO: what about functions?
                    convert_metadata_to_basic(ctx.get_llvm_type(item_type)).unwrap_or_else(|| {
                        panic!("{item_type:?} can not be converted to a tuple item")
                    })
                })
                .collect();
            let struct_type = ctx.context.struct_type(&types, false);
            struct_type.into()
        }
        Type::Array(_) => todo!(),
        Type::Struct(_, fields) => {
            let llvm_fields: Vec<_> = fields
                .iter()
                .map(|(_, field_type)| {
                    // TODO: what about functions?
                    convert_metadata_to_basic(ctx.get_llvm_type(field_type)).unwrap_or_else(|| {
                        panic!("{field_type:?} can not be converted to a struct field")
                    })
                })
                .collect();
            let struct_type = ctx.context.struct_type(&llvm_fields, false);
            struct_type.into()
        }
        Type::Function { .. } => ctx.context.ptr_type(Default::default()).into(),
    }
}

pub fn convert_metadata_to_basic(ty: BasicMetadataTypeEnum) -> Option<BasicTypeEnum> {
    match ty {
        BasicMetadataTypeEnum::ArrayType(t) => Some(BasicTypeEnum::ArrayType(t)),
        BasicMetadataTypeEnum::FloatType(t) => Some(BasicTypeEnum::FloatType(t)),
        BasicMetadataTypeEnum::IntType(t) => Some(BasicTypeEnum::IntType(t)),
        BasicMetadataTypeEnum::PointerType(t) => Some(BasicTypeEnum::PointerType(t)),
        BasicMetadataTypeEnum::StructType(t) => Some(BasicTypeEnum::StructType(t)),
        BasicMetadataTypeEnum::VectorType(t) => Some(BasicTypeEnum::VectorType(t)),
        _ => None, // For metadata-only types that aren't BasicType-compatible
    }
}
