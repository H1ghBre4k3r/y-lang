//! # Assignment Type Checking: Immutability by Default
//!
//! Assignment statements in Y enforce immutability by default, requiring explicit
//! mutability annotations for variable reassignment. This design choice reflects
//! several key language philosophy decisions:
//!
//! - Memory safety through compile-time mutability tracking
//! - Functional programming encouragement via immutable-first design
//! - LLVM optimization enablement through aliasing guarantees
//! - Predictable state changes that prevent action-at-a-distance bugs
//!
//! The type compatibility checking ensures that LLVM can generate efficient
//! assignment instructions without runtime type conversion overhead.

use std::cell::RefCell;
use std::rc::Rc;

use crate::typechecker::{Type, TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Assignment, LValue},
    typechecker::{
        context::Context,
        error::{ImmutableReassign, TypeCheckError, TypeMismatch},
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Assignment<()> {
    type Typed = Assignment<TypeInformation>;

    /// Assignment type checking enforces mutability constraints to prevent aliasing bugs.
    ///
    /// The mutability check occurs first because it's cheaper than type checking and
    /// provides clearer error messages. This ordering also matches developer mental
    /// models where permission to modify comes before type compatibility.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let context = ctx.clone();
        let Assignment {
            lvalue,
            rvalue,
            position,
            ..
        } = self;

        // Step 1: Verify the target variable is mutable before allowing assignment
        // Extract the root variable name from the lvalue (handles both direct IDs and property access)
        let name = lvalue.get_original_variable_name().name;
        if let Some(false) = ctx.scope.is_variable_mutable(&name) {
            // Variable was declared as immutable - assignment is not allowed
            return Err(TypeCheckError::ImmutableReassign(
                ImmutableReassign {
                    variable_name: name,
                },
                position,
            ));
        }

        // Step 2: Type check both the assignment target and the value being assigned
        let lvalue = lvalue.check(ctx)?;
        let mut rvalue = rvalue.check(ctx)?;
        let info = rvalue.get_info();

        // Step 3: Extract types for compatibility verification
        let variable_type_id = { lvalue.get_info().type_id.borrow().clone() };
        let rvalue_type_id = { rvalue.get_info().type_id.borrow().clone() };

        // Step 4: Ensure type compatibility between assignment target and value
        match (variable_type_id, rvalue_type_id) {
            // Both sides have concrete types - they must match exactly
            (Some(variable_type_id), Some(rvalue_type_id)) => {
                if variable_type_id != rvalue_type_id {
                    // Type mismatch between variable and assigned value
                    return Err(TypeCheckError::TypeMismatch(
                        TypeMismatch {
                            expected: variable_type_id,
                            actual: rvalue_type_id,
                        },
                        rvalue.position(),
                    ));
                }
            }
            // Variable has concrete type, rvalue has unknown type - propagate variable's type
            (Some(variable_type_id), None) => {
                // Update the rvalue to match the variable's type
                rvalue.update_type(variable_type_id.clone())?;
                *info.type_id.borrow_mut() = Some(variable_type_id);
            }
            // Other cases (unknown variable type) - defer type checking
            _ => {}
        }

        // Step 5: Assignment statements always yield void (no return value)
        Ok(Assignment {
            lvalue,
            rvalue,
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Void))),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Assignment {
            lvalue: id,
            rvalue,
            position,
            ..
        } = this;

        Assignment {
            lvalue: TypeCheckable::revert(id),
            rvalue: TypeCheckable::revert(rvalue),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Assignment<TypeInformation> {
    type Validated = Assignment<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Assignment {
            lvalue,
            rvalue,
            info,
            position,
        } = self;

        Ok(Assignment {
            lvalue: lvalue.validate()?,
            rvalue: rvalue.validate()?,
            info: info.validate(&position)?,
            position,
        })
    }
}

impl TypeCheckable for LValue<()> {
    type Typed = LValue<TypeInformation>;

    /// LValue type checking delegates to expression checkers to maintain modularity.
    ///
    /// Rather than duplicating assignment-specific logic, this delegation leverages
    /// existing expression type checking for consistency. This design prevents
    /// assignment semantics from diverging from expression semantics unexpectedly.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        // LValue type checking delegates to the appropriate expression type checker
        // LValues represent assignable locations (variables, struct fields, array elements)
        match self {
            // Simple variable assignment - delegate to identifier type checking
            LValue::Id(id) => Ok(LValue::Id(id.check(ctx)?)),
            // Complex assignment target (struct.field, array[index]) - delegate to postfix type checking
            LValue::Postfix(postfix) => Ok(LValue::Postfix(postfix.check(ctx)?)),
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            LValue::Id(id) => LValue::Id(TypeCheckable::revert(id)),
            LValue::Postfix(postfix) => LValue::Postfix(TypeCheckable::revert(postfix)),
        }
    }
}

impl TypedConstruct for LValue<TypeInformation> {
    type Validated = LValue<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            LValue::Id(id) => Ok(LValue::Id(id.validate()?)),
            LValue::Postfix(postfix) => Ok(LValue::Postfix(postfix.validate()?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{
            Assignment, Expression, Id, LValue, Num, Postfix, StructFieldInitialisation,
            StructInitialisation,
        },
        typechecker::{
            context::Context,
            error::{ImmutableReassign, TypeCheckError, TypeMismatch, UndefinedVariable},
            types::Type,
            TypeCheckable, TypeInformation,
        },
    };

    #[test]
    fn test_simple_reassign() -> Result<()> {
        let mut ctx = Context::default();
        ctx.scope.add_variable(
            "foo",
            Expression::Id(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            true,
        )?;

        let ass = Assignment {
            lvalue: LValue::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
            info: (),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            position: Span::default(),
        };

        ass.check(&mut ctx)?;

        Ok(())
    }

    #[test]
    fn test_assign_type_missmatch() -> Result<()> {
        let mut ctx = Context::default();
        ctx.scope.add_variable(
            "foo",
            Expression::Id(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::FloatingPoint))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            true,
        )?;

        let ass = Assignment {
            lvalue: LValue::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
            info: (),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            position: Span::default(),
        };

        let result = ass.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::FloatingPoint,
                    actual: Type::Integer
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_immutable_assign_error() -> Result<()> {
        let mut ctx = Context::default();
        ctx.scope.add_variable(
            "foo",
            Expression::Id(Id {
                name: "foo".into(),
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            false,
        )?;

        let ass = Assignment {
            lvalue: LValue::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
            info: (),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            position: Span::default(),
        };

        let result = ass.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::ImmutableReassign(
                ImmutableReassign {
                    variable_name: "foo".into()
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_undefined_reassign_error() -> Result<()> {
        let mut ctx = Context::default();

        let ass = Assignment {
            lvalue: LValue::Id(Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            }),
            info: (),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            position: Span::default(),
        };

        let result = ass.check(&mut ctx);

        assert_eq!(
            result,
            Err(TypeCheckError::UndefinedVariable(
                UndefinedVariable {
                    variable_name: "foo".into()
                },
                Span::default()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_struct_property_assign() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_type(
            "Foo",
            Type::Struct("Foo".to_string(), vec![("bar".to_string(), Type::Integer)]),
        )?;

        ctx.scope.add_variable(
            "foo",
            Expression::StructInitialisation(StructInitialisation {
                id: Id {
                    name: "foo".to_string(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Struct(
                            "Foo".to_string(),
                            vec![("bar".to_string(), Type::Integer)],
                        )))),
                        context: ctx.clone(),
                    },
                    position: Span::default(),
                },
                fields: vec![StructFieldInitialisation {
                    name: Id {
                        name: "bar".into(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: ctx.clone(),
                        },
                        position: Span::default(),
                    },
                    value: Expression::Num(Num::Integer(
                        1337,
                        TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: ctx.clone(),
                        },
                        Span::default(),
                    )),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                        context: ctx.clone(),
                    },
                    position: Span::default(),
                }],
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Struct(
                        "Foo".to_string(),
                        vec![("bar".to_string(), Type::Integer)],
                    )))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            true,
        )?;

        let assignment = Assignment {
            lvalue: LValue::Postfix(Postfix::PropertyAccess {
                expr: Box::new(Expression::Id(Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default(),
                })),
                property: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                },
                info: (),
                position: Span::default(),
            }),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        assignment.check(&mut ctx)?;

        Ok(())
    }
    #[test]
    fn test_immutable_struct_property_assign_error() -> Result<()> {
        let mut ctx = Context::default();

        ctx.scope.add_type(
            "Foo",
            Type::Struct("Foo".to_string(), vec![("bar".to_string(), Type::Integer)]),
        )?;

        ctx.scope.add_variable(
            "foo",
            Expression::StructInitialisation(StructInitialisation {
                id: Id {
                    name: "foo".to_string(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Struct(
                            "Foo".to_string(),
                            vec![("bar".to_string(), Type::Integer)],
                        )))),
                        context: ctx.clone(),
                    },
                    position: Span::default(),
                },
                fields: vec![StructFieldInitialisation {
                    name: Id {
                        name: "bar".into(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: ctx.clone(),
                        },
                        position: Span::default(),
                    },
                    value: Expression::Num(Num::Integer(
                        1337,
                        TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                            context: ctx.clone(),
                        },
                        Span::default(),
                    )),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                        context: ctx.clone(),
                    },
                    position: Span::default(),
                }],
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Struct(
                        "Foo".to_string(),
                        vec![("bar".to_string(), Type::Integer)],
                    )))),
                    context: ctx.clone(),
                },
                position: Span::default(),
            }),
            false,
        )?;

        let assignment = Assignment {
            lvalue: LValue::Postfix(Postfix::PropertyAccess {
                expr: Box::new(Expression::Id(Id {
                    name: "foo".into(),
                    info: (),
                    position: Span::default(),
                })),
                property: Id {
                    name: "bar".into(),
                    info: (),
                    position: Span::default(),
                },
                info: (),
                position: Span::default(),
            }),
            rvalue: Expression::Num(Num::Integer(42, (), Span::default())),
            info: (),
            position: Span::default(),
        };

        let res = assignment.check(&mut ctx);

        assert!(res.is_err());

        Ok(())
    }
}
