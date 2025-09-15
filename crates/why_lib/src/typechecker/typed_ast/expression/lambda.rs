use std::{cell::RefCell, collections::HashSet, rc::Rc};

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

/// Analyzes an expression to find free variables (variables that are not defined in the current scope)
fn find_free_variables(
    expr: &Expression<TypeInformation>,
    param_names: &HashSet<String>,
) -> Vec<(String, Type)> {
    let mut free_vars = HashSet::new();
    collect_free_variables(expr, param_names, &mut free_vars);
    free_vars.into_iter().collect()
}

/// Recursively collects free variables from an expression
fn collect_free_variables(
    expr: &Expression<TypeInformation>,
    param_names: &HashSet<String>,
    free_vars: &mut HashSet<(String, Type)>,
) {
    match expr {
        Expression::Id(id) => {
            // If this is not a parameter and has a known type, it's a free variable
            if !param_names.contains(&id.name) {
                if let Some(type_id) = id.info.type_id.borrow().as_ref() {
                    if type_id != &Type::Unknown {
                        free_vars.insert((id.name.clone(), type_id.clone()));
                    }
                }
            }
        }
        Expression::Lambda(lambda) => {
            // For nested lambdas, we need to exclude their parameters too
            let mut nested_param_names = param_names.clone();
            for param in &lambda.parameters {
                nested_param_names.insert(param.name.name.clone());
            }
            collect_free_variables(&lambda.expression, &nested_param_names, free_vars);
        }
        Expression::Parens(expr) => {
            collect_free_variables(expr, param_names, free_vars);
        }
        Expression::Binary(binary) => {
            collect_free_variables(&binary.left, param_names, free_vars);
            collect_free_variables(&binary.right, param_names, free_vars);
        }
        Expression::Postfix(postfix) => match postfix {
            crate::parser::ast::Postfix::Call { expr, args, .. } => {
                collect_free_variables(expr, param_names, free_vars);
                for arg in args {
                    collect_free_variables(arg, param_names, free_vars);
                }
            }
            crate::parser::ast::Postfix::Index { expr, index, .. } => {
                collect_free_variables(expr, param_names, free_vars);
                collect_free_variables(index, param_names, free_vars);
            }
            crate::parser::ast::Postfix::PropertyAccess { expr, .. } => {
                collect_free_variables(expr, param_names, free_vars);
            }
        },
        Expression::Prefix(prefix) => match prefix {
            crate::parser::ast::Prefix::Negation { expr, .. } => {
                collect_free_variables(expr, param_names, free_vars);
            }
            crate::parser::ast::Prefix::Minus { expr, .. } => {
                collect_free_variables(expr, param_names, free_vars);
            }
        },
        Expression::If(if_expr) => {
            collect_free_variables(&if_expr.condition, param_names, free_vars);
            // For blocks, we need to collect from each statement
            for stmt in &if_expr.then_block.statements {
                match stmt {
                    crate::parser::ast::Statement::Expression(expr) => {
                        collect_free_variables(expr, param_names, free_vars);
                    }
                    crate::parser::ast::Statement::YieldingExpression(expr) => {
                        collect_free_variables(expr, param_names, free_vars);
                    }
                    _ => {}
                }
            }
            // Check if else_block has any statements (i.e., it's not empty)
            if !if_expr.else_block.statements.is_empty() {
                for stmt in &if_expr.else_block.statements {
                    match stmt {
                        crate::parser::ast::Statement::Expression(expr) => {
                            collect_free_variables(expr, param_names, free_vars);
                        }
                        crate::parser::ast::Statement::YieldingExpression(expr) => {
                            collect_free_variables(expr, param_names, free_vars);
                        }
                        _ => {}
                    }
                }
            }
        }
        Expression::Block(block) => {
            for stmt in &block.statements {
                match stmt {
                    crate::parser::ast::Statement::Expression(expr) => {
                        collect_free_variables(expr, param_names, free_vars);
                    }
                    crate::parser::ast::Statement::YieldingExpression(expr) => {
                        collect_free_variables(expr, param_names, free_vars);
                    }
                    crate::parser::ast::Statement::Declaration(_) => {
                        // Local declarations don't affect free variables in outer scopes
                    }
                    _ => {
                        // Other statement types don't contain expressions to analyze
                    }
                }
            }
        }
        // Other expression types don't contain variables
        Expression::Num(_) => {}
        Expression::Bool(_) => {}
        Expression::Character(_) => {}
        Expression::AstString(_) => {}
        Expression::Function(_) => {}
        Expression::Array(array) => {
            match array {
                crate::parser::ast::Array::Literal { values, .. } => {
                    for value in values {
                        collect_free_variables(value, param_names, free_vars);
                    }
                }
                crate::parser::ast::Array::Default {
                    initial_value,
                    length,
                    ..
                } => {
                    collect_free_variables(initial_value, param_names, free_vars);
                    // length is a Num, which doesn't contain variables
                }
            }
        }
        Expression::StructInitialisation(struct_init) => {
            for field in &struct_init.fields {
                collect_free_variables(&field.value, param_names, free_vars);
            }
        }
    }
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
        let param_names: HashSet<String> = checked_parameters
            .iter()
            .map(|p| p.name.name.clone())
            .collect();

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

        // Analyze free variables for closure capture
        let free_variables = find_free_variables(&checked_expression, &param_names);

        // Create function type if we have concrete types, otherwise leave as None for later inference
        let function_type = if param_types.iter().any(|t| matches!(t, Type::Unknown))
            || matches!(return_type, Type::Unknown)
        {
            None
        } else if free_variables.is_empty() {
            // Non-capturing lambda - use regular function type
            Some(Type::Function {
                params: param_types,
                return_value: Box::new(return_type),
            })
        } else {
            // Capturing lambda - use closure type with capture information
            let closure_type = Type::Closure {
                params: param_types,
                return_value: Box::new(return_type),
                captures: free_variables,
            };
            eprintln!("Lambda creating closure type: {:?}", closure_type);
            Some(closure_type)
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

        // check if we have function or closure
        let (params, return_value, actual_type_id) = match type_id.clone() {
            Type::Function {
                params,
                return_value,
            } => {
                // For function types, check if we actually need a closure due to captures
                let param_names: HashSet<String> = self.parameters
                    .iter()
                    .map(|p| p.name.name.clone())
                    .collect();
                let free_variables = find_free_variables(&self.expression, &param_names);

                if !free_variables.is_empty() {
                    // Convert to closure type
                    let closure_type = Type::Closure {
                        params: params.clone(),
                        return_value: return_value.clone(),
                        captures: free_variables.clone(),
                    };
                    (params, return_value, closure_type)
                } else {
                    (params, return_value, type_id.clone())
                }
            },
            Type::Closure {
                params,
                return_value,
                ..
            } => (params, return_value, type_id.clone()),
            _ => return err,
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

        self.info.type_id = Rc::new(RefCell::new(Some(actual_type_id)));

        Ok(())
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError> {
        let Lambda {
            parameters,
            expression,
            info,
            position,
        } = self;

        let mut validated_parameters = vec![];
        for param in parameters {
            validated_parameters.push(param.validate()?);
        }

        Ok(Lambda {
            parameters: validated_parameters,
            expression: Box::new(expression.validate()?),
            info: info.validate(&position)?,
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
