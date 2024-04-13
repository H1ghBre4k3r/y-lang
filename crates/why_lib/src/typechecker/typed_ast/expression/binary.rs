use crate::{
    parser::ast::BinaryExpression,
    typechecker::{context::Context, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for BinaryExpression<()> {
    type Output = BinaryExpression<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        todo!()
    }

    fn revert(this: &Self::Output) -> Self {
        todo!()
    }
}
