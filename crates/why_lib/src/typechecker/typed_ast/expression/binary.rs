use crate::{
    parser::ast::BinaryExpression,
    typechecker::{context::Context, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for BinaryExpression<()> {
    type Output = BinaryExpression<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        match self {
            BinaryExpression::Addition {
                left,
                right,
                info,
                position,
            } => todo!(),
            BinaryExpression::Substraction {
                left,
                right,
                info,
                position,
            } => todo!(),
            BinaryExpression::Multiplication {
                left,
                right,
                info,
                position,
            } => todo!(),
            BinaryExpression::Division {
                left,
                right,
                info,
                position,
            } => todo!(),
            BinaryExpression::Equal {
                left,
                right,
                info,
                position,
            } => todo!(),
            BinaryExpression::GreaterThan {
                left,
                right,
                info,
                position,
            } => todo!(),
            BinaryExpression::LessThen {
                left,
                right,
                info,
                position,
            } => todo!(),
            BinaryExpression::GreaterOrEqual {
                left,
                right,
                info,
                position,
            } => todo!(),
            BinaryExpression::LessOrEqual {
                left,
                right,
                info,
                position,
            } => todo!(),
        }
        todo!()
    }

    fn revert(this: &Self::Output) -> Self {
        todo!()
    }
}
