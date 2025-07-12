use crate::{
    codegen::{convert_metadata_to_basic, CodeGen, IntoLLVMType},
    parser::ast::Initialisation,
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Initialisation<ValidatedTypeInformation> {
    type ReturnValue = ();
    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let Initialisation { id, value, .. } = self;

        let ValidatedTypeInformation { type_id, .. } = value.get_info();

        let llvm_value = value.codegen(ctx);

        let llvm_alloca = ctx
            .builder
            .build_alloca(
                convert_metadata_to_basic(type_id.to_llvm_type(ctx.context))
                    .expect("This should definetly not happen"),
                &id.name,
            )
            .expect("build_alloca failed");

        if let Err(e) = ctx.builder.build_store(llvm_alloca, llvm_value) {
            panic!("{e}");
        };

        ctx.store_variable(&id.name, llvm_alloca.into());
    }
}
