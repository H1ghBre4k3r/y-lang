mod id;
mod num;

use inkwell::values::BasicValueEnum;

use crate::{parser::ast::Expression, typechecker::ValidatedTypeInformation};

use super::CodeGen;

impl<'ctx> CodeGen<'ctx> for Expression<ValidatedTypeInformation> {
    type ReturnValue = BasicValueEnum<'ctx>;

    fn codegen(&self, ctx: &super::CodegenContext<'ctx>) -> BasicValueEnum<'ctx> {
        match self {
            Expression::Id(id) => id.codegen(ctx),
            Expression::Num(num) => num.codegen(ctx),
            Expression::Character(character) => todo!(),
            Expression::AstString(ast_string) => todo!(),
            Expression::Function(function) => todo!(),
            Expression::Lambda(lambda) => todo!(),
            Expression::If(_) => todo!(),
            Expression::Block(block) => todo!(),
            Expression::Parens(expression) => todo!(),
            Expression::Postfix(postfix) => todo!(),
            Expression::Prefix(prefix) => todo!(),
            Expression::Binary(binary_expression) => todo!(),
            Expression::Array(array) => todo!(),
            Expression::StructInitialisation(struct_initialisation) => todo!(),
        }
    }
}
