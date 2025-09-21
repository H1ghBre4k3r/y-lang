use inkwell::values::BasicValueEnum;

use crate::{
    codegen::{CodeGen, CodegenContext},
    parser::ast::If,
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for If<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    fn codegen(&self, ctx: &CodegenContext<'ctx>) -> Self::ReturnValue {
        let If {
            condition,
            then_block,
            else_block,
            info,
            ..
        } = self;

        let condition_llvm_value = condition.codegen(ctx)?;

        let function_llvm_value = ctx.builder.get_insert_block()?.get_parent()?;

        let then_label = ctx
            .context
            .append_basic_block(function_llvm_value, "if.then");
        let else_label = ctx
            .context
            .append_basic_block(function_llvm_value, "if.else");
        let merge_label = ctx
            .context
            .append_basic_block(function_llvm_value, "if.merge");

        ctx.builder
            .build_conditional_branch(
                condition_llvm_value.into_int_value(),
                then_label,
                else_label,
            )
            .ok()?;

        ctx.builder.position_at_end(then_label);

        ctx.enter_scope();
        let then_value = then_block.codegen(ctx);
        ctx.exit_scope();

        ctx.builder.build_unconditional_branch(merge_label).ok()?;

        ctx.builder.position_at_end(else_label);

        ctx.enter_scope();
        let else_value = else_block.codegen(ctx);
        ctx.exit_scope();

        ctx.builder.position_at_end(merge_label);

        let yields_value = info.type_id != Type::Void;
        match (then_value, else_value) {
            (Some(then_value), Some(else_value)) if yields_value => {
                let phi = ctx
                    .builder
                    .build_phi(then_value.get_type(), "if_result")
                    .ok()?;

                phi.add_incoming(&[(&then_value, then_label), (&else_value, else_label)]);
                Some(phi.as_basic_value())
            }
            other if yields_value => panic!("Unexpected {other:?} for yielding if expression"),
            _ => None,
        }
    }
}
