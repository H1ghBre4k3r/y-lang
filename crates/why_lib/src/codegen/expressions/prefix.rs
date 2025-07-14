use inkwell::values::BasicValueEnum;

use crate::{
    codegen::CodeGen,
    parser::ast::Prefix,
    typechecker::{Type, ValidatedTypeInformation},
};

impl<'ctx> CodeGen<'ctx> for Prefix<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &crate::codegen::CodegenContext<'ctx>) -> Self::ReturnValue {
        match self {
            Prefix::Negation { expr, position } => todo!(),
            Prefix::Minus { expr, .. } => {
                let expr_type = expr.get_info().type_id;
                let expr = expr.codegen(ctx);

                match expr_type {
                    Type::Integer => {
                        let expr = expr.into_int_value();

                        ctx.builder.build_int_neg(expr, "").unwrap().into()
                    }
                    Type::FloatingPoint => {
                        let expr = expr.into_float_value();

                        ctx.builder.build_float_neg(expr, "").unwrap().into()
                    }

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
    }
}
