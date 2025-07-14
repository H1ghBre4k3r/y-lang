mod ast_string;
mod binary;
mod id;
mod num;
mod postfix;
mod prefix;

use inkwell::values::BasicValueEnum;

use crate::{parser::ast::Expression, typechecker::ValidatedTypeInformation};

use super::CodeGen;

impl<'ctx> CodeGen<'ctx> for Expression<ValidatedTypeInformation> {
    type ReturnValue = Option<BasicValueEnum<'ctx>>;

    fn codegen(&self, ctx: &super::CodegenContext<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        match self {
            Expression::Id(id) => Some(id.codegen(ctx)),
            Expression::Num(num) => Some(num.codegen(ctx)),
            Expression::Character(character) => todo!(),
            Expression::AstString(ast_string) => Some(ast_string.codegen(ctx)),
            Expression::Function(function) => todo!(),
            Expression::Lambda(lambda) => todo!(),
            Expression::If(_) => todo!(),
            Expression::Block(block) => todo!(),
            Expression::Parens(expression) => todo!(),
            Expression::Postfix(postfix) => postfix.codegen(ctx),
            Expression::Prefix(prefix) => Some(prefix.codegen(ctx)),
            Expression::Binary(binary_expression) => Some(binary_expression.codegen(ctx)),
            Expression::Array(array) => todo!(),
            Expression::StructInitialisation(struct_initialisation) => todo!(),
        }
    }
}
