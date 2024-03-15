mod id;
mod num;

use crate::{
    parser::ast::Expression,
    typechecker::{context::Context, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for Expression<()> {
    type Output = Expression<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Output> {
        match self {
            Expression::Id(id) => Ok(Expression::Id(id.check(ctx)?)),
            Expression::Num(num) => Ok(Expression::Num(num.check(ctx)?)),
            Expression::Function(_) => todo!(),
            Expression::Lambda(_) => todo!(),
            Expression::If(_) => todo!(),
            Expression::Block(_) => todo!(),
            Expression::Parens(_) => todo!(),
            Expression::Postfix(_) => todo!(),
            Expression::Prefix(_) => todo!(),
            Expression::Binary(_) => todo!(),
            Expression::Array(_) => todo!(),
            Expression::StructInitialisation(_) => todo!(),
        }
    }
}
