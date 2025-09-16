mod array;
mod binary;
mod block;
mod bool;
mod character;
mod function;
mod id;
mod if_expression;
pub mod lambda;
mod num;
mod postfix;
mod prefix;
mod string;
mod struct_initialisation;

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::Expression,
    typechecker::{
        context::Context, error::TypeCheckError, types::Type, TypeCheckable, TypeInformation,
        TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Expression<()> {
    type Typed = Expression<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        match self {
            Expression::Id(id) => Ok(Expression::Id(id.check(ctx)?)),
            Expression::Num(num) => Ok(Expression::Num(num.check(ctx)?)),
            Expression::Bool(bool) => Ok(Expression::Bool(bool.check(ctx)?)),
            Expression::Character(character) => Ok(Expression::Character(character.check(ctx)?)),
            Expression::AstString(string) => Ok(Expression::AstString(string.check(ctx)?)),
            Expression::Function(func) => Ok(Expression::Function(func.check(ctx)?)),
            Expression::Lambda(lambda) => Ok(Expression::Lambda(lambda.check(ctx)?)),
            Expression::If(if_exp) => Ok(Expression::If(if_exp.check(ctx)?)),
            Expression::Block(block) => Ok(Expression::Block(block.check(ctx)?)),
            Expression::Parens(exp) => Ok(Expression::Parens(Box::new(exp.check(ctx)?))),
            Expression::Postfix(post) => Ok(Expression::Postfix(post.check(ctx)?)),
            Expression::Prefix(pref) => Ok(Expression::Prefix(pref.check(ctx)?)),
            Expression::Binary(bin) => Ok(Expression::Binary(Box::new(bin.check(ctx)?))),
            Expression::Array(arr) => Ok(Expression::Array(arr.check(ctx)?)),
            Expression::StructInitialisation(init) => {
                Ok(Expression::StructInitialisation(init.check(ctx)?))
            }
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            Expression::Id(id) => Expression::Id(TypeCheckable::revert(id)),
            Expression::Num(num) => Expression::Num(TypeCheckable::revert(num)),
            Expression::Bool(bool) => Expression::Bool(TypeCheckable::revert(bool)),
            Expression::Character(character) => {
                Expression::Character(TypeCheckable::revert(character))
            }
            Expression::AstString(string) => Expression::AstString(TypeCheckable::revert(string)),
            Expression::Function(func) => Expression::Function(TypeCheckable::revert(func)),
            Expression::Lambda(lambda) => Expression::Lambda(TypeCheckable::revert(lambda)),
            Expression::If(if_exp) => Expression::If(TypeCheckable::revert(if_exp)),
            Expression::Block(block) => Expression::Block(TypeCheckable::revert(block)),
            Expression::Parens(exp) => {
                Expression::Parens(Box::new(TypeCheckable::revert(exp.as_ref())))
            }
            Expression::Postfix(post) => Expression::Postfix(TypeCheckable::revert(post)),
            Expression::Prefix(pref) => Expression::Prefix(TypeCheckable::revert(pref)),
            Expression::Binary(bin) => {
                Expression::Binary(Box::new(TypeCheckable::revert(bin.as_ref())))
            }
            Expression::Array(arr) => Expression::Array(TypeCheckable::revert(arr)),
            Expression::StructInitialisation(_) => todo!(),
        }
    }
}

impl TypedConstruct for Expression<TypeInformation> {
    type Validated = Expression<ValidatedTypeInformation>;

    fn update_type(&mut self, type_id: Type) -> Result<(), TypeCheckError> {
        match self {
            Expression::Id(_) => unreachable!(),
            Expression::Num(num) => num.update_type(type_id),
            Expression::Bool(_) => unreachable!(),
            Expression::Character(_) => unreachable!(),
            Expression::AstString(_) => unreachable!(),
            Expression::Function(_) => unreachable!(),
            Expression::Lambda(lambda) => lambda.update_type(type_id),
            Expression::If(_) => unreachable!(),
            Expression::Block(_) => unreachable!(),
            Expression::Parens(inner_expression) => inner_expression.update_type(type_id),
            Expression::Postfix(_) => unreachable!(),
            Expression::Prefix(_) => unreachable!(),
            Expression::Binary(_) => unreachable!(),
            Expression::Array(array) => array.update_type(type_id),
            Expression::StructInitialisation(_) => unreachable!(),
        }
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            Expression::Id(id) => Ok(Expression::Id(id.validate()?)),
            Expression::Num(num) => Ok(Expression::Num(num.validate()?)),
            Expression::Bool(bool) => Ok(Expression::Bool(bool.validate()?)),
            Expression::Character(character) => Ok(Expression::Character(character.validate()?)),
            Expression::AstString(string) => Ok(Expression::AstString(string.validate()?)),
            Expression::Function(function) => Ok(Expression::Function(function.validate()?)),
            Expression::Lambda(lambda) => Ok(Expression::Lambda(lambda.validate()?)),
            Expression::If(if_expression) => Ok(Expression::If(if_expression.validate()?)),
            Expression::Block(block) => Ok(Expression::Block(block.validate()?)),
            Expression::Parens(parens) => Ok(Expression::Parens(Box::new(parens.validate()?))),
            Expression::Postfix(postfix) => Ok(Expression::Postfix(postfix.validate()?)),
            Expression::Prefix(prefix) => Ok(Expression::Prefix(prefix.validate()?)),
            Expression::Binary(binary) => Ok(Expression::Binary(Box::new(binary.validate()?))),
            Expression::Array(array) => Ok(Expression::Array(array.validate()?)),
            Expression::StructInitialisation(struct_initialisation) => Ok(
                Expression::StructInitialisation(struct_initialisation.validate()?),
            ),
        }
    }
}
