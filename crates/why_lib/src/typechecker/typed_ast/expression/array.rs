//! # Array Type Checking: Homogeneity for LLVM Memory Layout
//!
//! Arrays in Y are intentionally homogeneous to guarantee predictable memory layouts
//! for efficient LLVM code generation. This design choice enables:
//!
//! - Contiguous memory allocation without padding
//! - Compile-time size calculations for stack allocation
//! - Efficient element access with pointer arithmetic
//! - Type-safe array operations without runtime type checks
//!
//! The trade-off is reduced flexibility compared to heterogeneous collections,
//! but this aligns with Y's performance-first philosophy.

use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, TypedConstruct, ValidatedTypeInformation};
use crate::{
    parser::ast::Array,
    typechecker::{
        context::Context,
        error::{TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult,
    },
};

impl TypeCheckable for Array<()> {
    type Typed = Array<TypeInformation>;

    /// Array homogeneity is enforced because Y prioritizes LLVM optimization opportunities.
    ///
    /// Homogeneous arrays enable LLVM's vectorization passes and allow for efficient
    /// bounds checking elimination. The strict type enforcement here prevents runtime
    /// type confusion that would require expensive dynamic checks.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let context = ctx.clone();
        match self {
            Array::Literal {
                values, position, ..
            } => {
                // Type check each expression in the array literal
                let mut checked_values = vec![];
                for value in values.into_iter() {
                    checked_values.push(value.check(ctx)?);
                }

                // Extract the type information from the first element to establish array type
                let type_id = checked_values.first().map(|val| val.get_info().type_id);

                if let Some(type_id) = &type_id {
                    // Get the concrete type from the first element
                    let type_id = { type_id.borrow() }.clone();
                    for value in checked_values.iter() {
                        let value_type = { value.get_info().type_id.borrow() }.clone();
                        // Verify all elements have the same type (homogeneous array)
                        if let (Some(type_id), Some(value_type)) = (&type_id, value_type) {
                            if *type_id != value_type {
                                return Err(TypeCheckError::TypeMismatch(
                                    TypeMismatch {
                                        expected: type_id.clone(),
                                        actual: value_type,
                                    },
                                    value.position(),
                                ));
                            }
                        }
                    }
                }

                // Create the final array type from the element type
                Ok(Array::Literal {
                    values: checked_values,
                    info: TypeInformation {
                        type_id: type_id.map_or(
                            // Empty array or elements with unknown types
                            Rc::new(RefCell::new(None)),
                            |type_id| {
                                // Wrap element type in Array type
                                Rc::new(RefCell::new(
                                    type_id
                                        .borrow()
                                        .clone()
                                        .map(|type_id| Type::Array(Box::new(type_id))),
                                ))
                            },
                        ),
                        context,
                    },
                    position,
                })
            }
            Array::Default {
                initial_value,
                length,
                position,
                ..
            } => {
                // Type check the default value that will fill the array
                let initial_value = initial_value.check(ctx)?;

                // Type check the length expression (should eventually be enforced as integer)
                let length = length.check(ctx)?;

                // Get the type of the default value to determine array element type
                let type_id = { initial_value.get_info().type_id.borrow() }.clone();

                Ok(Array::Default {
                    initial_value: Box::new(initial_value),
                    length,
                    info: TypeInformation {
                        type_id: type_id.map_or(
                            // Default value has unknown type
                            Rc::new(RefCell::new(None)),
                            // Create array type from default value type
                            |type_id| Rc::new(RefCell::new(Some(Type::Array(Box::new(type_id))))),
                        ),
                        context,
                    },
                    position,
                })
            }
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            Array::Literal {
                values, position, ..
            } => Array::Literal {
                values: values.iter().map(TypeCheckable::revert).collect(),
                info: (),
                position: position.clone(),
            },
            Array::Default {
                initial_value,
                length,
                position,
                ..
            } => Array::Default {
                initial_value: Box::new(TypeCheckable::revert(initial_value.as_ref())),
                length: TypeCheckable::revert(length),
                info: (),
                position: position.clone(),
            },
        }
    }
}

impl TypedConstruct for Array<TypeInformation> {
    type Validated = Array<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            Array::Default {
                initial_value,
                length,
                info,
                position,
            } => Ok(Array::Default {
                initial_value: Box::new(initial_value.validate()?),
                length: length.validate()?,
                info: info.validate(&position)?,
                position,
            }),
            Array::Literal {
                values,
                info,
                position,
            } => {
                let mut validated_values = vec![];
                for value in values {
                    validated_values.push(value.validate()?);
                }

                Ok(Array::Literal {
                    values: validated_values,
                    info: info.validate(&position)?,
                    position,
                })
            }
        }
    }

    fn update_type(&mut self, type_id: Type) -> TypeResult<()> {
        let inner_type = match self {
            Array::Literal { info, .. } => info.type_id.borrow().clone(),
            Array::Default { info, .. } => info.type_id.borrow().clone(),
        };

        if let Some(inner_type) = inner_type {
            if inner_type != type_id {
                return Err(TypeCheckError::TypeMismatch(
                    TypeMismatch {
                        expected: type_id,
                        actual: inner_type,
                    },
                    self.position(),
                ));
            }
        }

        match self {
            Array::Literal { info, .. } => info.type_id = Rc::new(RefCell::new(Some(type_id))),
            Array::Default { info, .. } => info.type_id = Rc::new(RefCell::new(Some(type_id))),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{Array, Expression, Num},
        typechecker::{
            context::Context,
            error::{TypeCheckError, TypeMismatch},
            types::Type,
            TypeCheckable,
        },
    };

    #[test]
    fn test_empty_array_literal() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Literal {
            values: vec![],
            info: (),
            position: Span::default(),
        };

        let arr = arr.check(&mut ctx)?;

        assert_eq!(arr.get_info().type_id, Rc::new(RefCell::new(None)));

        Ok(())
    }

    #[test]
    fn test_single_element_array_literal() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Literal {
            values: vec![Expression::Num(Num::Integer(42, (), Span::default()))],
            info: (),
            position: Span::default(),
        };

        let arr = arr.check(&mut ctx)?;

        assert_eq!(
            arr.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Array(Box::new(Type::Integer)))))
        );
        Ok(())
    }

    #[test]
    fn test_multiple_element_array_literal_match() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Literal {
            values: vec![
                Expression::Num(Num::FloatingPoint(42.0, (), Span::default())),
                Expression::Num(Num::FloatingPoint(1337.0, (), Span::default())),
            ],
            info: (),
            position: Span::default(),
        };

        let arr = arr.check(&mut ctx)?;

        assert_eq!(
            arr.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Array(Box::new(
                Type::FloatingPoint
            )))))
        );
        Ok(())
    }

    #[test]
    fn test_multiple_element_array_literal_mismatch() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Literal {
            values: vec![
                Expression::Num(Num::Integer(42, (), Span::default())),
                Expression::Num(Num::FloatingPoint(1337.0, (), Span::default())),
            ],
            info: (),
            position: Span::default(),
        };

        let res = arr.check(&mut ctx);

        assert_eq!(
            res,
            Err(TypeCheckError::TypeMismatch(
                TypeMismatch {
                    expected: Type::Integer,
                    actual: Type::FloatingPoint
                },
                Span::default()
            ))
        );
        Ok(())
    }

    #[test]
    fn test_simple_default_array() -> Result<()> {
        let mut ctx = Context::default();

        let arr = Array::Default {
            initial_value: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
            length: Num::Integer(10, (), Span::default()),
            info: (),
            position: Span::default(),
        };

        let arr = arr.check(&mut ctx)?;

        assert_eq!(
            arr.get_info().type_id,
            Rc::new(RefCell::new(Some(Type::Array(Box::new(Type::Integer)))))
        );

        Ok(())
    }
}
