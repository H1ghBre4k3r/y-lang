use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::StructDeclaration,
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for StructDeclaration<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &CodegenContext<'ctx>) {
        let StructDeclaration {
            id, fields, info, ..
        } = self;

        let struct_name = id.name.clone();

        // Extract field types from the AST fields
        let field_types: Vec<(String, Type)> = fields
            .iter()
            .map(|field| {
                let field_name = field.name.name.clone();
                // Convert TypeName to Type using the context
                let field_type = Type::try_from((field.type_name.clone(), &info.context))
                    .unwrap_or(Type::Unknown);
                (field_name, field_type)
            })
            .collect();

        // Create LLVM struct type
        let llvm_field_types: Vec<_> = field_types
            .iter()
            .map(|(_, field_type)| {
                let llvm_metadata_type = ctx.get_llvm_type(field_type);
                super::super::convert_metadata_to_basic(llvm_metadata_type)
                    .expect("Struct field type must be basic")
            })
            .collect();

        let struct_type = ctx.context.struct_type(&llvm_field_types, false);

        // Store the struct type in the context for later use
        // This allows struct initialization and field access to find the type
        ctx.types.borrow_mut().insert(
            Type::Struct(struct_name.clone(), field_types.clone()),
            struct_type.into(),
        );
    }
}

