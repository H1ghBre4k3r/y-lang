//! # Numeric Literal Type Checking: Direct Type Assignment
//!
//! Numeric literals in Y have immediate, unambiguous types that support LLVM's
//! primitive type optimizations. This design avoids the complexity of numeric
//! type hierarchies found in other languages:
//!
//! - Integer literals map directly to LLVM's integer types (i64)
//! - Floating-point literals map to LLVM's float types (f64)
//! - No automatic promotion or coercion between numeric types
//! - Consistent memory layout and operation costs across platforms
//!
//! This explicit approach eliminates hidden conversion costs and ensures that
//! numeric operations have predictable performance characteristics.

use std::{cell::RefCell, rc::Rc};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::Num,
    typechecker::{
        context::Context, types::Type, TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

impl TypeCheckable for Num<()> {
    type Typed = Num<TypeInformation>;

    /// Numeric type assignment is immediate because literal syntax determines type.
    ///
    /// This deterministic approach eliminates the need for complex numeric type
    /// promotion rules, ensuring that arithmetic operations have known costs and
    /// LLVM can generate optimal machine code without runtime type checks.
    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        // Numeric literal type checking is deterministic - no inference or validation needed
        // Each numeric literal has a known type based on its syntactic form
        match self {
            // Integer literals (42, -10, 0) always get Integer type
            // No type inference is required since the type is unambiguous from the literal
            Num::Integer(val, _, position) => Ok(Num::Integer(
                val,
                TypeInformation {
                    // Assign concrete Integer type immediately - no Unknown phase needed
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: ctx.clone(),
                },
                position,
            )),
            // Floating-point literals (3.14, 0.5, -2.7) always get FloatingPoint type
            // Like integers, the type is determined directly from the literal syntax
            Num::FloatingPoint(val, _, position) => Ok(Num::FloatingPoint(
                val,
                TypeInformation {
                    // Assign concrete FloatingPoint type immediately - no inference needed
                    type_id: Rc::new(RefCell::new(Some(Type::FloatingPoint))),
                    context: ctx.clone(),
                },
                position,
            )),
        }
    }

    fn revert(this: &Self::Typed) -> Self {
        match this {
            Num::Integer(val, _, pos) => Num::Integer(*val, (), pos.clone()),
            Num::FloatingPoint(val, _, pos) => Num::FloatingPoint(*val, (), pos.clone()),
        }
    }
}

impl TypedConstruct for Num<TypeInformation> {
    type Validated = Num<ValidatedTypeInformation>;

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        match self {
            Num::Integer(val, info, position) => {
                Ok(Num::Integer(val, info.validate(&position)?, position))
            }
            Num::FloatingPoint(val, info, position) => {
                Ok(Num::FloatingPoint(val, info.validate(&position)?, position))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::Num,
        typechecker::{context::Context, types::Type, TypeCheckable},
    };

    #[test]
    fn test_check_integer() -> Result<(), Box<dyn Error>> {
        let Num::Integer(num, info, ..) =
            Num::Integer(42, (), Span::default()).check(&mut Context::default())?
        else {
            unreachable!()
        };

        assert_eq!(num, 42);
        assert_eq!(info.type_id, Rc::new(RefCell::new(Some(Type::Integer))));
        Ok(())
    }

    #[test]
    fn test_check_floatingpoint() -> Result<(), Box<dyn Error>> {
        let Num::FloatingPoint(num, info, ..) =
            Num::FloatingPoint(42.0, (), Span::default()).check(&mut Context::default())?
        else {
            unreachable!()
        };

        assert_eq!(num, 42.0);
        assert_eq!(
            info.type_id,
            Rc::new(RefCell::new(Some(Type::FloatingPoint)))
        );
        Ok(())
    }
}
