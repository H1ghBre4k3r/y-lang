use std::{cell::RefCell, rc::Rc, collections::HashSet, sync::Mutex};

use crate::typechecker::{TypeValidationError, ValidatedTypeInformation};
use crate::{
    parser::ast::{Expression, Id, Lambda, LambdaParameter},
    typechecker::{
        context::Context,
        error::{RedefinedConstant, TypeCheckError, TypeMismatch},
        types::Type,
        TypeCheckable, TypeInformation, TypeResult, TypedConstruct,
    },
};

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Global storage for lambda capture information, keyed by lambda position
static LAMBDA_CAPTURES: Lazy<Mutex<HashMap<String, CaptureInfo>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Store capture information globally
pub fn store_lambda_captures(lambda_id: String, captures: CaptureInfo) {
    LAMBDA_CAPTURES.lock().unwrap().insert(lambda_id, captures);
}

/// Retrieve capture information globally
pub fn get_lambda_captures(lambda_id: &str) -> Option<CaptureInfo> {
    LAMBDA_CAPTURES.lock().unwrap().get(lambda_id).cloned()
}

/// Information about captures in a lambda expression
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CaptureInfo {
    /// Names of variables captured from outer scopes with their types
    pub captures: Vec<(String, Type)>,
}

impl CaptureInfo {
    fn new() -> Self {
        CaptureInfo {
            captures: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.captures.is_empty()
    }
}

/// Extended type information for lambda expressions that includes capture analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidatedLambdaTypeInformation {
    pub type_id: Type,
    #[serde(skip)]
    pub context: Context,
    pub captures: CaptureInfo,
}

impl ValidatedLambdaTypeInformation {
    pub fn new(type_id: Type, context: Context, captures: CaptureInfo) -> Self {
        ValidatedLambdaTypeInformation {
            type_id,
            context,
            captures,
        }
    }
}

/// Visitor to collect all identifier references in an expression
struct IdentifierCollector {
    identifiers: HashSet<String>,
}

impl IdentifierCollector {
    fn new() -> Self {
        IdentifierCollector {
            identifiers: HashSet::new(),
        }
    }

    fn visit_expression(&mut self, expr: &Expression<TypeInformation>) {
        match expr {
            Expression::Id(id) => {
                self.identifiers.insert(id.name.clone());
            }
            Expression::Binary(binary) => {
                self.visit_expression(&binary.left);
                self.visit_expression(&binary.right);
            }
            Expression::Prefix(prefix) => {
                match prefix {
                    crate::parser::ast::Prefix::Negation { expr, .. } => {
                        self.visit_expression(expr);
                    }
                    crate::parser::ast::Prefix::Minus { expr, .. } => {
                        self.visit_expression(expr);
                    }
                }
            }
            Expression::Postfix(postfix) => {
                match postfix {
                    crate::parser::ast::Postfix::Call { expr, args, .. } => {
                        self.visit_expression(expr);
                        for arg in args {
                            self.visit_expression(arg);
                        }
                    }
                    crate::parser::ast::Postfix::Index { expr, index, .. } => {
                        self.visit_expression(expr);
                        self.visit_expression(index);
                    }
                    crate::parser::ast::Postfix::PropertyAccess { expr, .. } => {
                        self.visit_expression(expr);
                    }
                }
            }
            Expression::If(if_expr) => {
                self.visit_expression(&if_expr.condition);
                // For now, just visit the condition. Block traversal can be added later.
            }
            Expression::Lambda(lambda) => {
                // For nested lambdas, we still want to collect identifiers
                // but we need to be careful about their parameter scopes
                self.visit_expression(&lambda.expression);
            }
            // For now, skip complex traversal of blocks, structs, arrays
            // We can add these incrementally as needed
            Expression::Block(_) => {}
            Expression::StructInitialisation(_) => {}
            Expression::Array(_) => {}
            // Leaf nodes - no further traversal needed
            Expression::Num(_) | Expression::Bool(_) | Expression::Character(_) | Expression::AstString(_) => {}
            // Other expression types
            Expression::Function(_) => {}
            Expression::Parens(inner) => {
                self.visit_expression(inner);
            }
        }
    }

    fn collect_from(expr: &Expression<TypeInformation>) -> HashSet<String> {
        let mut collector = IdentifierCollector::new();
        collector.visit_expression(expr);
        collector.identifiers
    }
}

/// Analyze captures for a lambda expression
fn analyze_captures(
    lambda_expr: &Expression<TypeInformation>,
    parameters: &[LambdaParameter<TypeInformation>],
    ctx: &mut Context,
) -> CaptureInfo {
    // Collect all identifiers referenced in the lambda body
    let referenced_identifiers = IdentifierCollector::collect_from(lambda_expr);

    // Filter out lambda parameters
    let param_names: HashSet<String> = parameters
        .iter()
        .map(|param| param.name.name.clone())
        .collect();

    // Find captures: identifiers that are referenced but not parameters
    let mut captures = Vec::new();
    for identifier in referenced_identifiers {
        if !param_names.contains(&identifier) {
            // This identifier is potentially captured from outer scope
            // Try to resolve its type in the current context
            if let Some(type_ref) = ctx.scope.resolve_name(&identifier) {
                if let Some(var_type) = type_ref.borrow().clone() {
                    captures.push((identifier, var_type));
                }
            }
        }
    }

    // Sort captures for deterministic order
    captures.sort_by(|a, b| a.0.cmp(&b.0));

    CaptureInfo { captures }
}

impl TypeCheckable for Lambda<()> {
    type Typed = Lambda<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let Lambda {
            parameters,
            expression,
            position,
            ..
        } = self;

        let context = ctx.clone();

        ctx.scope.enter_scope();

        let mut checked_parameters = vec![];

        for param in parameters.into_iter() {
            checked_parameters.push(param.check(ctx)?);
        }

        let checked_expression = expression.check(ctx)?;

        ctx.scope.exit_scope();

        // Infer function type from parameters and return expression
        let mut param_types = vec![];
        for param in &checked_parameters {
            // For now, if parameter types are unknown, we can't fully infer the lambda type
            // This will be resolved during type propagation
            let param_type = param.info.type_id.borrow().clone().unwrap_or(Type::Unknown);
            param_types.push(param_type);
        }

        let return_type = checked_expression
            .get_info()
            .type_id
            .borrow()
            .clone()
            .unwrap_or(Type::Unknown);

        // Create function type if we have concrete types, otherwise leave as None for later inference
        let function_type = if param_types.iter().any(|t| matches!(t, Type::Unknown))
            || matches!(return_type, Type::Unknown)
        {
            None
        } else {
            Some(Type::Function {
                params: param_types,
                return_value: Box::new(return_type),
            })
        };

        Ok(Lambda {
            parameters: checked_parameters,
            expression: Box::new(checked_expression),
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(function_type)),
                context,
            },
            position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let Lambda {
            parameters,
            expression,
            position,
            ..
        } = this;

        Lambda {
            parameters: parameters.iter().map(TypeCheckable::revert).collect(),
            expression: Box::new(TypeCheckable::revert(expression.as_ref())),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for Lambda<TypeInformation> {
    type Validated = Lambda<ValidatedTypeInformation>;

    fn update_type(&mut self, type_id: Type) -> Result<(), TypeCheckError> {
        let err = Err(TypeCheckError::TypeMismatch(
            TypeMismatch {
                expected: Type::Function {
                    params: vec![Type::Unknown; self.parameters.len()],
                    return_value: Box::new(Type::Unknown),
                },
                actual: type_id.clone(),
            },
            self.position.clone(),
        ));

        // check, if we have function
        let Type::Function {
            params,
            return_value,
        } = type_id.clone()
        else {
            return err;
        };

        if let Some(current_type) = self.info.type_id.borrow().as_ref() {
            if *current_type == type_id {
                return Ok(());
            }

            // Check if current type can be refined by the new type
            // This allows lambdas with Unknown parameter types to be updated
            if let Type::Function {
                params: current_params,
                return_value: current_return,
            } = current_type
            {
                // If current lambda has Unknown parameter types, allow refinement
                let can_refine = current_params.iter().any(|p| matches!(p, Type::Unknown))
                    || matches!(current_return.as_ref(), Type::Unknown);

                if can_refine {
                    // Allow the update to proceed - fall through to the update logic below
                } else {
                    // Types are concrete and don't match - this is an error
                    return Err(TypeCheckError::TypeMismatch(
                        TypeMismatch {
                            expected: current_type.clone(),
                            actual: type_id,
                        },
                        self.position.clone(),
                    ));
                }
            } else {
                // Current type is not a function - this is an error
                return Err(TypeCheckError::TypeMismatch(
                    TypeMismatch {
                        expected: current_type.clone(),
                        actual: type_id,
                    },
                    self.position.clone(),
                ));
            }
        }

        // check for correct arity
        if params.len() != self.parameters.len() {
            return err;
        }

        // clone context to mess nothing up
        let mut ctx = self.info.context.clone();

        ctx.scope.enter_scope();

        // enter all parameters with their respective types into the scope
        for (i, t) in params.iter().enumerate() {
            let name = &self.parameters[i].name.name;
            let position = &self.parameters[i].name.position;

            if ctx
                .scope
                .add_variable(
                    name,
                    Expression::Id(Id {
                        name: name.clone(),
                        info: TypeInformation {
                            type_id: Rc::new(RefCell::new(Some(t.clone()))),
                            context: ctx.clone(),
                        },
                        position: position.clone(),
                    }),
                    false,
                )
                .is_err()
            {
                return Err(TypeCheckError::RedefinedConstant(
                    RedefinedConstant {
                        constant_name: name.to_string(),
                    },
                    position.clone(),
                ));
            }
        }

        // check (the reverted) expression
        let expr =
            <Expression<()> as TypeCheckable>::revert(self.expression.as_ref()).check(&mut ctx)?;

        // check, if return types match
        if let Some(expr_type) = expr.get_info().type_id.borrow_mut().as_ref() {
            if *expr_type != *return_value {
                return Err(TypeCheckError::TypeMismatch(
                    TypeMismatch {
                        expected: expr_type.clone(),
                        actual: *return_value.clone(),
                    },
                    expr.position(),
                ));
            }
        }

        // update types of parameters accordingly
        for (i, t) in params.iter().enumerate() {
            self.parameters[i].update_type(t.to_owned())?;
        }

        // update our expression as well
        self.expression = Box::new(expr);

        self.info.type_id = Rc::new(RefCell::new(Some(type_id)));

        Ok(())
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Lambda {
            parameters,
            expression,
            info,
            position,
        } = self;

        // Perform capture analysis before consuming parameters
        let mut ctx = info.context.clone();
        let captures = analyze_captures(&expression, &parameters, &mut ctx);

        // Generate a unique lambda ID from position for capture storage
        let lambda_id = format!("lambda_{}_{}_{}_{}",
            position.start.0, position.start.1, position.end.0, position.end.1);

        // Store capture information globally
        store_lambda_captures(lambda_id.clone(), captures.clone());

        // For debugging - print capture info
        if !captures.is_empty() {
            eprintln!("Lambda {} captures: {:?}", lambda_id, captures.captures);
        }

        let mut validated_parameters = vec![];
        for param in parameters {
            validated_parameters.push(param.validate()?);
        }

        let validated_info = info.validate(&position)?;

        Ok(Lambda {
            parameters: validated_parameters,
            expression: Box::new(expression.validate()?),
            info: validated_info,
            position,
        })
    }
}

impl TypeCheckable for LambdaParameter<()> {
    type Typed = LambdaParameter<TypeInformation>;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed> {
        let LambdaParameter {
            name,
            position: param_position,
            ..
        } = self;

        let Id {
            name,
            position: id_position,
            ..
        } = name;

        let type_id = Rc::new(RefCell::new(None));

        let id = Id {
            name,
            info: TypeInformation {
                type_id: type_id.clone(),
                context: ctx.clone(),
            },
            position: id_position,
        };

        if ctx
            .scope
            .add_variable(&id.name, Expression::Id(id.clone()), false)
            .is_err()
        {
            return Err(TypeCheckError::RedefinedConstant(
                RedefinedConstant {
                    constant_name: id.name,
                },
                param_position,
            ));
        }

        Ok(LambdaParameter {
            name: id,
            info: TypeInformation {
                type_id,
                context: ctx.clone(),
            },
            position: param_position,
        })
    }

    fn revert(this: &Self::Typed) -> Self {
        let LambdaParameter { name, position, .. } = this;

        LambdaParameter {
            name: TypeCheckable::revert(name),
            info: (),
            position: position.clone(),
        }
    }
}

impl TypedConstruct for LambdaParameter<TypeInformation> {
    type Validated = LambdaParameter<ValidatedTypeInformation>;

    fn update_type(&mut self, type_id: Type) -> std::result::Result<(), TypeCheckError> {
        *self.info.type_id.borrow_mut() = Some(type_id);

        Ok(())
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let LambdaParameter {
            name,
            info,
            position,
        } = self;

        Ok(LambdaParameter {
            name: name.validate()?,
            info: info.validate(&position)?,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, error::Error, rc::Rc};

    use anyhow::Result;

    use crate::{
        lexer::Span,
        parser::ast::{Expression, Id, Initialisation, Lambda, LambdaParameter, Num, TypeName},
        typechecker::{context::Context, types::Type, TypeCheckable, TypeInformation},
    };

    #[test]
    fn test_parameter_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let param = LambdaParameter {
            name: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            info: (),
            position: Span::default(),
        };

        let param = param.check(&mut ctx)?;

        assert_eq!(
            param,
            LambdaParameter {
                name: Id {
                    name: "foo".into(),
                    info: TypeInformation {
                        type_id: Rc::new(RefCell::new(None)),
                        context: Context::default(),
                    },
                    position: Span::default(),
                },
                info: TypeInformation {
                    type_id: Rc::new(RefCell::new(None)),
                    context: Context::default(),
                },
                position: Span::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_parameter_in_scope_insertion() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let param = LambdaParameter {
            name: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            info: (),
            position: Span::default(),
        };

        param.check(&mut ctx)?;

        assert_eq!(
            ctx.scope.resolve_name("foo"),
            Some(Rc::new(RefCell::new(None)))
        );

        Ok(())
    }

    #[test]
    fn test_lambda_type_inference() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::default();

        let lambda = Lambda {
            parameters: vec![],
            expression: Box::new(Expression::Num(Num::Integer(42, (), Span::default()))),
            info: (),
            position: Span::default(),
        };

        let lambda = lambda.check(&mut ctx)?;

        let expected = Lambda {
            parameters: vec![],
            expression: Box::new(Expression::Num(Num::Integer(
                42,
                TypeInformation {
                    type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                    context: Context::default(),
                },
                Span::default(),
            ))),
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Function {
                    params: vec![],
                    return_value: Box::new(Type::Integer),
                }))),
                context: Context::default(),
            },
            position: Span::default(),
        };

        eprintln!("{lambda:#?}");
        eprintln!("{expected:#?}");

        assert_eq!(lambda, expected);

        Ok(())
    }

    #[test]
    fn test_error_on_type_update() -> Result<()> {
        let mut ctx = Context::default();

        let init = Initialisation {
            id: Id {
                name: "foo".into(),
                info: (),
                position: Span::default(),
            },
            mutable: false,
            type_name: Some(crate::parser::ast::TypeName::Fn {
                params: vec![TypeName::Literal("i64".into(), Span::default())],
                return_type: Box::new(TypeName::Literal("i64".into(), Span::default())),
                position: Span::default(),
            }),
            value: Expression::Lambda(Lambda {
                parameters: vec![LambdaParameter {
                    name: Id {
                        name: "x".into(),
                        info: (),
                        position: Span::default(),
                    },
                    info: (),
                    position: Span::default(),
                }],
                expression: Box::new(Expression::Id(Id {
                    name: "x".into(),
                    info: (),
                    position: Span::default(),
                })),
                info: (),
                position: Span::default(),
            }),
            info: (),
            position: Span::default(),
        };

        init.check(&mut ctx)?;

        ctx.scope.update_variable(
            "foo",
            Type::Function {
                params: vec![Type::Integer],
                return_value: Box::new(Type::Integer),
            },
        )?;

        assert!(ctx
            .scope
            .update_variable(
                "foo",
                Type::Function {
                    params: vec![Type::FloatingPoint],
                    return_value: Box::new(Type::Integer),
                },
            )
            .is_err());
        Ok(())
    }
}
