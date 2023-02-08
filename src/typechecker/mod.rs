mod error;

use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc, str::FromStr};

use crate::ast::{
    Assignment, Ast, BinaryOp, BinaryVerb, Block, Boolean, Definition, Expression, FnCall, FnDef,
    Ident, If, Integer, Intrinsic, Position, Statement, Str, Type,
};

use self::error::TypeError;

type TResult<T> = Result<T, TypeError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableType {
    Void,
    Bool,
    Str,
    Int,
    // TODO: Maybe just dont use
    Any,
    Func {
        params: Vec<VariableType>,
        return_value: Box<VariableType>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    _type: VariableType,
}

pub struct VariableParseError(String);

impl FromStr for VariableType {
    type Err = VariableParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "void" => Ok(Self::Void),
            "bool" => Ok(Self::Bool),
            "str" => Ok(Self::Str),
            "int" => Ok(Self::Int),
            _ => Err(VariableParseError(format!("Invalid type '{s}'"))),
        }
    }
}

impl Display for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use VariableType::*;

        let value = &match self {
            Void => "void".to_owned(),
            Bool => "bool".to_owned(),
            Int => "int".to_owned(),
            Str => "str".to_owned(),
            Any => "any".to_owned(),
            Func {
                params,
                return_value,
                ..
            } => format!("{params:?} -> {return_value:?}"),
        };

        f.write_str(value)
    }
}

type ScopeFrame = HashMap<String, VariableType>;

type ScopeFrameReference = Rc<RefCell<ScopeFrame>>;

#[derive(Default, Debug, Clone)]
pub struct Scope {
    scope_stack: Vec<ScopeFrameReference>,
}

impl PartialEq for Scope {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Scope {}

impl Scope {
    /// Find a value/reference in this scope by iterating over the scopes from back to front.
    pub fn find(&self, name: &str) -> Option<VariableType> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in scopes {
            if let Some(variable) = scope.borrow().get(name) {
                return Some(variable.clone());
            }
        }

        None
    }

    pub fn is_in_current_scope(&self, name: &str) -> bool {
        let scopes = self.scope_stack.clone();
        if let Some(last) = scopes.last() {
            return last.borrow().contains_key(name);
        }
        false
    }

    /// Check, if a variable with a given name is present.
    pub fn contains(&self, name: &str) -> bool {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in &scopes {
            if scope.borrow().contains_key(name) {
                return true;
            }
        }

        false
    }

    /// Push a new scope frame.
    pub fn push(&mut self) {
        self.scope_stack.push(Rc::new(RefCell::new(HashMap::new())))
    }

    /// Pop the last scope frame.
    pub fn pop(&mut self) {
        self.scope_stack.pop();
    }

    /// Create a new variable on the current scope.
    pub fn set(&mut self, name: &str, value: VariableType) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.borrow_mut().insert(name.to_owned(), value);
        }
    }

    /// Update a value of an already present variable.
    pub fn update(
        &mut self,
        name: &str,
        value: VariableType,
        position: &(usize, usize),
    ) -> Result<(), TypeError> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();

        for scope in &mut scopes {
            let mut scope = scope.borrow_mut();
            if let Some(old_type) = scope.get(name) {
                if *old_type != value {
                    return Err(TypeError {
                        message: format!(
                            "Could not assign variable '{name}' with type '{old_type}' a value of type '{value}'"
                        ),
                        position: position.to_owned(),
                    });
                }
                scope.insert(name.to_owned(), value);

                break;
            }
        }

        scopes.reverse();
        self.scope_stack = scopes;

        Ok(())
    }
}

fn setup_scope() -> Scope {
    let mut scope = Scope::default();

    scope.push();

    scope.set(
        "print",
        VariableType::Func {
            params: vec![VariableType::Any],
            return_value: Box::new(VariableType::Void),
        },
    );

    scope.set(
        "printi",
        VariableType::Func {
            params: vec![VariableType::Int],
            return_value: Box::new(VariableType::Void),
        },
    );

    scope
}

pub struct Typechecker {
    ast: Ast<()>,
}

impl Typechecker {
    pub fn from_ast(ast: Ast<()>) -> Self {
        Self { ast }
    }

    pub fn check(&self) -> Result<Ast<TypeInfo>, TypeError> {
        let nodes = self.ast.nodes();

        let mut scope = setup_scope();

        let mut statements = vec![];

        for node in nodes {
            statements.push(Self::check_statement(&node, &mut scope)?);
        }

        Ok(Ast::from_nodes(statements))
    }

    fn check_statement(
        statement: &Statement<()>,
        scope: &mut Scope,
    ) -> TResult<Statement<TypeInfo>> {
        Ok(match &statement {
            Statement::Expression(expression) => {
                Statement::Expression(Self::check_expression(None, expression, scope)?)
            }
            Statement::Intrinsic(intrinsic) => {
                Statement::Intrinsic(Self::check_intrinsic(intrinsic, scope)?)
            }
        })
    }

    fn check_intrinsic(
        intrinsic: &Intrinsic<()>,
        scope: &mut Scope,
    ) -> TResult<Intrinsic<TypeInfo>> {
        Ok(match &intrinsic {
            Intrinsic::Definition(definition) => {
                Intrinsic::Definition(Self::check_definition(definition, scope)?)
            }
            Intrinsic::Assignment(assignment) => {
                Intrinsic::Assignment(Self::check_assignment(assignment, scope)?)
            }
        })
    }

    fn check_if(if_statement: &If<()>, scope: &mut Scope) -> TResult<If<TypeInfo>> {
        let condition = Self::check_expression(None, &if_statement.condition, scope)?;
        let condition_info = condition.info();
        let condition_type = condition_info._type;

        if condition_type != VariableType::Bool {
            return Err(TypeError {
                message: format!("Invalid tye of condition '{condition_type:?}'"),
                position: if_statement.condition.position(),
            });
        }

        let if_block = Self::check_block(&if_statement.if_block, scope)?;
        let if_block_type = if_block.info._type.clone();

        let mut new_if = If {
            condition: Box::new(condition),
            if_block,
            else_block: None,
            position: if_statement.position,
            info: TypeInfo {
                _type: if_block_type.clone(),
            },
        };

        if let Some(else_block) = &if_statement.else_block {
            let else_block = Self::check_block(else_block, scope)?;
            let else_block_type = else_block.info._type.clone();

            if if_block_type != else_block_type {
                return Err(TypeError {
                    message: format!(
                        "Return type mismatch of if-else. Got '{if_block_type}' and '{else_block_type}'"
                    ),
                    position: if_statement.position,
                });
            }

            new_if.else_block = Some(else_block);
        }

        Ok(new_if)
    }

    fn check_block(block: &Block<()>, scope: &mut Scope) -> TResult<Block<TypeInfo>> {
        scope.push();

        let mut new_block = Block {
            position: block.position,
            block: vec![],
            info: TypeInfo {
                _type: VariableType::Void,
            },
        };

        for statement in &block.block {
            let statement = Self::check_statement(statement, scope)?;
            new_block.info._type = statement.info()._type;
            new_block.block.push(statement);
        }

        scope.pop();

        Ok(new_block)
    }

    fn check_definition(
        definition: &Definition<()>,
        scope: &mut Scope,
    ) -> TResult<Definition<TypeInfo>> {
        let definition_rhs =
            Self::check_expression(Some(&definition.ident), &definition.value, scope)?;

        scope.set(&definition.ident.value, definition_rhs.info()._type);

        let ident = &definition.ident;

        Ok(Definition {
            ident: Ident {
                position: ident.position,
                value: ident.value.clone(),
                info: definition_rhs.info(),
            },
            value: definition_rhs,
            position: definition.position,
            info: TypeInfo {
                _type: VariableType::Void,
            },
        })
    }

    fn check_assignment(
        assignment: &Assignment<()>,
        scope: &mut Scope,
    ) -> TResult<Assignment<TypeInfo>> {
        let ident = &assignment.ident;

        if !scope.contains(&ident.value) {
            return Err(TypeError {
                message: format!("Undefined identifier '{}'", ident.value),
                position: ident.position,
            });
        }

        if !scope.is_in_current_scope(&ident.value) {
            return Err(TypeError {
                message: format!(
                    "Variable '{}' can not be modified, because it is not defined in current scope",
                    ident.value
                ),
                position: ident.position,
            });
        }

        let assignment_rhs = Self::check_expression(Some(ident), &assignment.value, scope)?;

        scope.update(
            &ident.value,
            assignment_rhs.info()._type,
            &assignment.position,
        )?;

        Ok(Assignment {
            ident: Ident {
                position: ident.position,
                value: ident.value.clone(),
                info: assignment_rhs.info(),
            },
            value: assignment_rhs,
            position: assignment.position,
            info: TypeInfo {
                _type: VariableType::Void,
            },
        })
    }

    fn check_expression(
        identifier: Option<&Ident<()>>,
        expression: &Expression<()>,
        scope: &mut Scope,
    ) -> TResult<Expression<TypeInfo>> {
        Ok(match expression {
            Expression::If(if_statement) => Expression::If(Self::check_if(if_statement, scope)?),
            Expression::BinaryOp(binary_op) => {
                Expression::BinaryOp(Self::check_binary_operation(binary_op, scope)?)
            }
            Expression::Integer(Integer {
                value, position, ..
            }) => Expression::Integer(Integer {
                value: *value,
                position: *position,
                info: TypeInfo {
                    _type: VariableType::Int,
                },
            }),
            Expression::Str(Str {
                value, position, ..
            }) => Expression::Str(Str {
                value: value.to_owned(),
                position: *position,
                info: TypeInfo {
                    _type: VariableType::Str,
                },
            }),
            Expression::Boolean(Boolean {
                value, position, ..
            }) => Expression::Boolean(Boolean {
                value: *value,
                position: *position,
                info: TypeInfo {
                    _type: VariableType::Bool,
                },
            }),
            Expression::Ident(ident) => Expression::Ident(Self::check_identifier(ident, scope)?),
            Expression::FnCall(fn_call) => Expression::FnCall(Self::check_fn_call(fn_call, scope)?),
            Expression::FnDef(fn_def) => {
                Expression::FnDef(Self::check_fn_def(identifier, fn_def, scope)?)
            }
            Expression::Block(block) => Expression::Block(Self::check_block(block, scope)?),
        })
    }

    fn check_identifier(identifier: &Ident<()>, scope: &mut Scope) -> TResult<Ident<TypeInfo>> {
        match scope.find(&identifier.value) {
            Some(identifier_type) => Ok(Ident {
                value: identifier.value.clone(),
                position: identifier.position,
                info: TypeInfo {
                    _type: identifier_type,
                },
            }),
            None => {
                return Err(TypeError {
                    message: format!("Undefined identifier '{}'", identifier.value),
                    position: identifier.position,
                });
            }
        }
    }

    fn get_type_def(type_: &Type, position: Position) -> Result<VariableType, TypeError> {
        match type_ {
            Type::Literal(literal) => literal.parse().map_err(|_| TypeError {
                message: format!("Unexpected type annotatiot '{type_:?}'"),
                position,
            }),
            Type::Function {
                params,
                return_type,
            } => {
                let mut fn_params = vec![];
                for param in params {
                    fn_params.push(Self::get_type_def(param, position)?);
                }

                let return_type = Self::get_type_def(return_type, position)?;
                Ok(VariableType::Func {
                    return_value: Box::new(return_type),
                    params: fn_params,
                })
            }
        }
    }

    fn check_fn_def(
        identifier: Option<&Ident<()>>,
        fn_def: &FnDef<()>,
        scope: &mut Scope,
    ) -> TResult<FnDef<TypeInfo>> {
        let type_annotation = Self::get_type_def(
            &fn_def.type_annotation.value,
            fn_def.type_annotation.position,
        )?;
        scope.push();

        let mut params = vec![];

        for param in &fn_def.params {
            let param_type =
                Self::get_type_def(&param.type_annotation.value, param.type_annotation.position)?;

            scope.set(&param.ident.value, param_type.clone());
            params.push(param_type);
        }

        if let Some(ident) = identifier {
            scope.set(
                &ident.value,
                VariableType::Func {
                    params: params.clone(),
                    return_value: Box::new(type_annotation.clone()),
                },
            )
        }

        let block = Self::check_block(&fn_def.block, scope)?;

        if block.info._type != type_annotation {
            return Err(TypeError {
                message: format!(
                    "Expected return type of '{type_annotation}' but got '{}'",
                    block.info._type
                ),
                position: fn_def.position,
            });
        }

        scope.pop();

        Ok(FnDef {
            params: fn_def.params.clone(),
            type_annotation: fn_def.type_annotation.clone(),
            block: block.clone(),
            position: fn_def.position,
            info: TypeInfo {
                _type: VariableType::Func {
                    params,
                    return_value: Box::new(block.info._type),
                },
            },
        })
    }

    fn check_fn_call(fn_call: &FnCall<()>, scope: &mut Scope) -> TResult<FnCall<TypeInfo>> {
        scope.push();

        let ident = &fn_call.ident.value;

        let Some(fn_def) = scope.find(ident) else {
            return Err(TypeError {
                message: format!("Call to undefined function '{ident}'"),
                position: fn_call.position,
            });
        };

        let VariableType::Func { params, return_value, .. } = fn_def.clone() else {
            return Err(TypeError {
                message: format!("Trying to call an invalid function '{ident}'"),
                position: fn_call.position,
            });
        };

        if params.len() != fn_call.params.len() {
            return Err(TypeError {
                message: format!(
                    "Invalid amount of parameters! Expected {} but got {}",
                    params.len(),
                    fn_call.params.len()
                ),
                position: fn_call.position,
            });
        }

        let mut new_params = vec![];

        for (i, param) in params.iter().enumerate() {
            let call_param = Self::check_expression(None, &fn_call.params[i], scope)?;
            let call_param_type = call_param.info()._type;

            if param != &call_param_type && param != &VariableType::Any {
                return Err(TypeError {
                    message: format!(
                        "Invalid type of parameter! Expected '{param}' but got '{call_param_type}'"
                    ),
                    position: fn_call.params[i].position(),
                });
            }

            new_params.push(call_param);
        }

        scope.pop();

        Ok(FnCall {
            ident: Ident {
                value: fn_call.ident.value.clone(),
                position: fn_call.ident.position,
                info: TypeInfo { _type: fn_def },
            },
            params: new_params,
            position: fn_call.position,
            info: TypeInfo {
                _type: *return_value,
            },
        })
    }

    fn check_binary_operation(
        binary_operation: &BinaryOp<()>,
        scope: &mut Scope,
    ) -> TResult<BinaryOp<TypeInfo>> {
        let position = binary_operation.position;

        let lhs = &binary_operation.lhs;
        let rhs = &binary_operation.rhs;

        let lhs = Self::check_expression(None, lhs, scope)?;
        let l_type = lhs.info()._type;

        let rhs = Self::check_expression(None, rhs, scope)?;
        let r_type = rhs.info()._type;

        match binary_operation.verb {
            BinaryVerb::Equal => {
                if l_type != r_type {
                    return Err(TypeError {
                        message: format!(
                        "Left and right value of binary operation do not match! ('{l_type}' and '{r_type}')"
                    ),
                        position,
                    });
                }
                Ok(BinaryOp {
                    verb: binary_operation.verb,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    position: binary_operation.position,
                    info: TypeInfo {
                        _type: VariableType::Bool,
                    },
                })
            }
            BinaryVerb::LessThan | BinaryVerb::GreaterThan => {
                if l_type != VariableType::Int || r_type != VariableType::Int {
                    return Err(TypeError {
                        message: format!(
                            "Invalid types for binary operation '{}'. Got '{}' and '{}'",
                            binary_operation.verb, l_type, r_type
                        ),
                        position,
                    });
                }
                Ok(BinaryOp {
                    verb: binary_operation.verb,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    position: binary_operation.position,
                    info: TypeInfo {
                        _type: VariableType::Bool,
                    },
                })
            }
            BinaryVerb::Plus | BinaryVerb::Minus | BinaryVerb::Times => {
                if l_type != VariableType::Int {
                    return Err(TypeError {
                        message: format!(
                        "Left value of numeric binary operation has to be of type Int. Found '{l_type}'"
                    ),
                        position: lhs.position(),
                    });
                } else if r_type != VariableType::Int {
                    return Err(TypeError {
                        message: format!(
                        "Right value of numeric binary operation has to be of type Int. Found '{r_type}'"
                    ),
                        position: rhs.position(),
                    });
                }

                Ok(BinaryOp {
                    verb: binary_operation.verb,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    position: binary_operation.position,
                    info: TypeInfo {
                        _type: VariableType::Int,
                    },
                })
            }
        }
    }
}
