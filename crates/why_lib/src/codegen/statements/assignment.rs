use crate::{
    codegen::CodeGen,
    parser::ast::{Assignment, LValue},
    typechecker::ValidatedTypeInformation,
};

impl<'ctx> CodeGen<'ctx> for Assignment<ValidatedTypeInformation> {
    type ReturnValue = ();

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        let Some(rvalue) = self.rvalue.codegen(ctx) else {
            unreachable!("Assignment rvalue must produce a value")
        };

        match &self.lvalue {
            LValue::Id(id) => {
                // Simple variable assignment - store to existing variable
                let variable_ptr = ctx.find_variable(&id.name);
                ctx.builder
                    .build_store(variable_ptr.into_pointer_value(), rvalue)
                    .unwrap();
            }
            LValue::Postfix(_postfix) => {
                // TODO: Handle array indexing and property access assignments
                todo!("Complex lvalue assignment not yet implemented")
            }
        }
    }
}
