//! # Statement Type Checking: Control Flow and Declaration Management
//!
//! Statement type checking in Y handles declarations, control flow, and
//! side effects while maintaining expression-oriented language semantics.
//! This design balances imperative convenience with functional principles:
//!
//! - Two-phase checking enables forward references and mutual recursion
//! - Statement type delegation maintains modularity and code organization
//! - Type information threading preserves context across statement boundaries
//! - LLVM optimization benefits from clear statement vs expression distinctions
//!
//! The trait implementations provide consistent type checking patterns while
//! enabling each statement type to implement domain-specific validation logic.
mod assignment;
/// Constant (immutable value) declarations with required explicit type
mod constant;
/// Variable / function forward declarations
mod declaration;
/// Variable initialisation with optional explicit annotation
mod initialisation;
/// Instance block declarations adding methods to a struct
mod instance;
/// Method declaration within an instance block
mod method_declaration;
/// Struct type declarations (fields registered in shallow pass)
mod struct_declaration;
/// While loop control flow construct
mod while_loop;

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Statement, TopLevelStatement},
    typechecker::{
        context::Context, error::TypeCheckError, types::Type, ShallowCheck, TypeCheckable,
        TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for TopLevelStatement<()> {
    type Typed = TopLevelStatement<TypeInformation>;

    /// Top-level statement type checking delegates to specific statement implementations.
    ///
    /// This delegation pattern maintains separation of concerns while enabling
    /// consistent type checking behavior across different statement types.
    /// Each statement type handles its own validation logic and type constraints.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        match self {
            TopLevelStatement::Function(func) => Ok(TopLevelStatement::Function(func.check(ctx)?)),
            TopLevelStatement::Constant(constant) => {
                Ok(TopLevelStatement::Constant(constant.check(ctx)?))
            }
            TopLevelStatement::Comment(c) => Ok(TopLevelStatement::Comment(c)),
            TopLevelStatement::Declaration(dec) => {
                Ok(TopLevelStatement::Declaration(dec.check(ctx)?))
            }
            TopLevelStatement::StructDeclaration(dec) => {
                Ok(TopLevelStatement::StructDeclaration(dec.check(ctx)?))
            }
            TopLevelStatement::Instance(inst) => Ok(TopLevelStatement::Instance(inst.check(ctx)?)),
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            TopLevelStatement::Function(func) => {
                TopLevelStatement::Function(TypeCheckable::revert(func))
            }
            TopLevelStatement::Constant(_) => {
                unimplemented!("TypeCheckable::revert is not implemented for Constants")
            }
            TopLevelStatement::Comment(c) => TopLevelStatement::Comment(c.to_owned()),
            TopLevelStatement::Declaration(dec) => {
                TopLevelStatement::Declaration(TypeCheckable::revert(dec))
            }
            TopLevelStatement::StructDeclaration(dec) => {
                TopLevelStatement::StructDeclaration(TypeCheckable::revert(dec))
            }
            TopLevelStatement::Instance(inst) => {
                TopLevelStatement::Instance(TypeCheckable::revert(inst))
            }
        }
    }
}

impl ShallowCheck for TopLevelStatement<()> {
    /// Shallow checking delegates to individual statement shallow check implementations.
    ///
    /// This enables forward references and complex declaration ordering without
    /// requiring explicit dependency resolution. Each statement registers its
    /// declarations early to support mutual references between statements.
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()> {
        match self {
            TopLevelStatement::Comment(_) => Ok(()),
            TopLevelStatement::Function(inner) => inner.shallow_check(ctx),
            TopLevelStatement::Constant(inner) => inner.shallow_check(ctx),
            TopLevelStatement::Declaration(inner) => inner.shallow_check(ctx),
            TopLevelStatement::StructDeclaration(inner) => inner.shallow_check(ctx),
            TopLevelStatement::Instance(inner) => inner.shallow_check(ctx),
        }
    }
}

impl TypedConstruct for TopLevelStatement<TypeInformation> {
    type Validated = TopLevelStatement<ValidatedTypeInformation>;

    fn update_type(&mut self, _type_id: Type) -> TypeResult<()> {
        unreachable!()
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            TopLevelStatement::Comment(c) => Ok(TopLevelStatement::Comment(c)),
            TopLevelStatement::Function(function) => {
                Ok(TopLevelStatement::Function(function.validate()?))
            }
            TopLevelStatement::Constant(constant) => {
                Ok(TopLevelStatement::Constant(constant.validate()?))
            }
            TopLevelStatement::Declaration(declaration) => {
                Ok(TopLevelStatement::Declaration(declaration.validate()?))
            }
            TopLevelStatement::StructDeclaration(struct_declaration) => Ok(
                TopLevelStatement::StructDeclaration(struct_declaration.validate()?),
            ),
            TopLevelStatement::Instance(instance) => {
                Ok(TopLevelStatement::Instance(instance.validate()?))
            }
        }
    }
}

impl TypeCheckable for Statement<()> {
    type Typed = Statement<TypeInformation>;

    /// Statement type checking provides consistent dispatch to statement-specific logic.
    ///
    /// This centralized dispatch ensures all statement types follow the same
    /// type checking protocol while allowing each statement to implement its
    /// own validation rules and type constraints.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        match self {
            Statement::Function(func) => Ok(Statement::Function(func.check(ctx)?)),
            Statement::WhileLoop(while_l) => Ok(Statement::WhileLoop(while_l.check(ctx)?)),
            Statement::Initialization(init) => Ok(Statement::Initialization(init.check(ctx)?)),
            Statement::Constant(constant) => Ok(Statement::Constant(constant.check(ctx)?)),
            Statement::Assignment(assign) => Ok(Statement::Assignment(assign.check(ctx)?)),
            Statement::Expression(exp) => Ok(Statement::Expression(exp.check(ctx)?)),
            Statement::YieldingExpression(exp) => {
                Ok(Statement::YieldingExpression(exp.check(ctx)?))
            }
            Statement::Return(exp) => Ok(Statement::Return(exp.check(ctx)?)),
            Statement::Comment(c) => Ok(Statement::Comment(c)),
            Statement::Declaration(dec) => Ok(Statement::Declaration(dec.check(ctx)?)),
            Statement::StructDeclaration(dec) => Ok(Statement::StructDeclaration(dec.check(ctx)?)),
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            Statement::Function(func) => Statement::Function(TypeCheckable::revert(func)),
            Statement::WhileLoop(while_l) => Statement::WhileLoop(TypeCheckable::revert(while_l)),
            Statement::Initialization(init) => {
                Statement::Initialization(TypeCheckable::revert(init))
            }
            Statement::Constant(_) => todo!(),
            Statement::Assignment(assign) => Statement::Assignment(TypeCheckable::revert(assign)),
            Statement::Expression(expr) => Statement::Expression(TypeCheckable::revert(expr)),
            Statement::YieldingExpression(expr) => {
                Statement::YieldingExpression(TypeCheckable::revert(expr))
            }
            Statement::Return(expr) => Statement::Return(TypeCheckable::revert(expr)),
            Statement::Comment(c) => Statement::Comment(c.to_owned()),
            Statement::Declaration(dec) => Statement::Declaration(TypeCheckable::revert(dec)),
            Statement::StructDeclaration(dec) => {
                Statement::StructDeclaration(TypeCheckable::revert(dec))
            }
        }
    }
}

impl TypedConstruct for Statement<TypeInformation> {
    type Validated = Statement<ValidatedTypeInformation>;

    fn update_type(&mut self, type_id: Type) -> std::result::Result<(), TypeCheckError> {
        match self {
            Statement::Function(_) => todo!(),
            Statement::WhileLoop(_) => todo!(),
            Statement::Initialization(init) => init.update_type(type_id),
            Statement::Constant(constant) => constant.update_type(type_id),
            Statement::Assignment(_) => todo!(),
            Statement::Expression(expr) => expr.update_type(type_id),
            Statement::YieldingExpression(expr) => expr.update_type(type_id),
            Statement::Return(expr) => expr.update_type(type_id),
            Statement::Comment(_) => Ok(()),
            Statement::Declaration(dec) => dec.update_type(type_id),
            Statement::StructDeclaration(dec) => dec.update_type(type_id),
        }
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            Statement::Function(function) => Ok(Statement::Function(function.validate()?)),
            Statement::WhileLoop(while_loop) => Ok(Statement::WhileLoop(while_loop.validate()?)),
            Statement::Initialization(initialisation) => {
                Ok(Statement::Initialization(initialisation.validate()?))
            }
            Statement::Constant(constant) => Ok(Statement::Constant(constant.validate()?)),
            Statement::Assignment(assignment) => Ok(Statement::Assignment(assignment.validate()?)),
            Statement::Expression(expression) => Ok(Statement::Expression(expression.validate()?)),
            Statement::YieldingExpression(yielding_expression) => Ok(
                Statement::YieldingExpression(yielding_expression.validate()?),
            ),
            Statement::Return(expression) => Ok(Statement::Return(expression.validate()?)),
            Statement::Comment(comment) => Ok(Statement::Comment(comment)),
            Statement::Declaration(declaration) => {
                Ok(Statement::Declaration(declaration.validate()?))
            }
            Statement::StructDeclaration(struct_declaration) => {
                Ok(Statement::StructDeclaration(struct_declaration.validate()?))
            }
        }
    }
}
