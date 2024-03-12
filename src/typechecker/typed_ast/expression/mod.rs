mod num;

use crate::{
    parser::ast::Expression,
    typechecker::{context::Context, TypeCheckable, TypeInformation, TypeResult},
};

impl TypeCheckable for Expression<()> {
    type Output = Expression<TypeInformation>;

    fn check(self, context: &mut Context) -> TypeResult<Self::Output> {
        match self {
            Expression::Id(_) => todo!(),
            Expression::Num(num) => Ok(Expression::Num(num.check(context)?)),
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
