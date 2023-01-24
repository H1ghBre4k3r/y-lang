mod error;

use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc, str::FromStr};

use crate::ast::{
    Assignment, Ast, BinaryOp, BinaryVerb, Block, Declaration, Expression, FnCall, FnDef, Ident,
    If, Intrinsic, Statement,
};

use self::error::TypeError;

type TypecheckResult = Result<VariableType, TypeError>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum VariableType {
    Void,
    Bool,
    Str,
    Int,
    // TODO: Maybe just dont use
    Any,
    Func {
        params: Vec<VariableType>,
        scope: Scope,
        return_value: Box<VariableType>,
    },
}

struct VariableParseError(String);

impl FromStr for VariableType {
    type Err = VariableParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "void" => Ok(Self::Void),
            "bool" => Ok(Self::Bool),
            "str" => Ok(Self::Str),
            "int" => Ok(Self::Int),
            _ => Err(VariableParseError(format!("Invalid type '{}'", s))),
        }
    }
}

impl Display for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use VariableType::*;

        f.write_str(match self {
            Void => "void",
            Bool => "bool",
            Int => "int",
            Str => "str",
            Any => "any",
            Func { .. } => todo!(),
        })
    }
}

type ScopeFrame = HashMap<String, VariableType>;

type ScopeFrameReference = Rc<RefCell<ScopeFrame>>;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct Scope {
    scope_stack: Vec<ScopeFrameReference>,
}

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

        return None;
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
                            "Could not assign variable '{}' with type '{}' a value of type '{}'",
                            name, old_type, value
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
            scope: Scope::default(),
            return_value: Box::new(VariableType::Void),
        },
    );

    scope
}

pub fn check_ast(ast: &Ast) -> Result<(), TypeError> {
    let nodes = ast.nodes();

    let mut scope = setup_scope();

    for node in nodes {
        check_statement(&node, &mut scope)?;
    }

    Ok(())
}

fn check_statement(statement: &Statement, scope: &mut Scope) -> TypecheckResult {
    Ok(match &statement {
        Statement::Expression(expression) => check_expression(None, &expression, scope)?,
        Statement::Intrinsic(intrinsic) => check_intrinsic(&intrinsic, scope)?,
    })
}

fn check_intrinsic(intrinsic: &Intrinsic, scope: &mut Scope) -> TypecheckResult {
    match &intrinsic {
        Intrinsic::Declaration(declaration) => check_declaration(declaration, scope),
        Intrinsic::Assignment(assignment) => check_assignment(assignment, scope),
    }
}

fn check_if(if_statement: &If, scope: &mut Scope) -> TypecheckResult {
    let condition_type = check_expression(None, &if_statement.condition, scope)?;

    if condition_type != VariableType::Bool {
        return Err(TypeError {
            message: format!("Invalid tye of condition '{:?}'", condition_type),
            position: if_statement.condition.position(),
        });
    }

    let if_return_type = check_block(&if_statement.if_block, scope)?;

    if let Some(else_block) = &if_statement.else_block {
        let else_return_type = check_block(else_block, scope)?;

        if if_return_type != else_return_type {
            return Err(TypeError {
                message: format!(
                    "Return type mismatch of if-else. Got '{}' and '{}'",
                    if_return_type, else_return_type
                ),
                position: if_statement.position,
            });
        }
    }

    Ok(if_return_type)
}

fn check_block(block: &Block, scope: &mut Scope) -> TypecheckResult {
    scope.push();

    let mut last_return = VariableType::Void;

    for statement in &block.block {
        last_return = check_statement(statement, scope)?;
    }

    scope.pop();

    Ok(last_return)
}

fn check_declaration(declaration: &Declaration, scope: &mut Scope) -> TypecheckResult {
    let declaration_type = check_expression(Some(&declaration.ident), &declaration.value, scope)?;

    scope.set(&declaration.ident.value, declaration_type);

    Ok(VariableType::Void)
}

fn check_assignment(assignment: &Assignment, scope: &mut Scope) -> TypecheckResult {
    let ident = &assignment.ident;

    if !scope.contains(&ident.value) {
        return Err(TypeError {
            message: format!("Undefined identifier '{}'", ident.value),
            position: ident.position,
        });
    }

    let assignment_type = check_expression(Some(ident), &assignment.value, scope)?;

    scope.update(&ident.value, assignment_type, &assignment.position)?;

    Ok(VariableType::Void)
}

fn check_expression(
    identifier: Option<&Ident>,
    expression: &Expression,
    scope: &mut Scope,
) -> TypecheckResult {
    match expression {
        Expression::If(if_statement) => check_if(if_statement, scope),
        Expression::BinaryOp(binary_op) => check_binary_operation(binary_op, scope),
        Expression::Integer(_) => Ok(VariableType::Int),
        Expression::Str(_) => Ok(VariableType::Str),
        Expression::Ident(ident) => check_identifier(ident, scope),
        Expression::FnCall(fn_call) => check_fn_call(fn_call, scope),
        Expression::FnDef(fn_def) => check_fn_def(identifier, fn_def, scope),
        Expression::Block(block) => check_block(block, scope),
    }
}

fn check_identifier(identifier: &Ident, scope: &mut Scope) -> TypecheckResult {
    match scope.find(&identifier.value) {
        Some(identifier_type) => Ok(identifier_type.clone()),
        None => {
            return Err(TypeError {
                message: format!("Undefined identifier '{}'", identifier.value),
                position: identifier.position,
            });
        }
    }
}

fn check_fn_def(identifier: Option<&Ident>, fn_def: &FnDef, scope: &mut Scope) -> TypecheckResult {
    let Ok(type_annotation) = fn_def.type_annotation.value.parse::<VariableType>() else {
        return Err(TypeError {
            message: format!("Unexpected type annotatiot '{}'", fn_def.type_annotation.value),
            position: fn_def.type_annotation.position
        })
    };

    scope.push();

    let mut params = vec![];

    for param in &fn_def.params {
        let Ok(param_type) = param.type_annotation.value.parse::<VariableType>() else {
            panic!()
        };

        scope.set(&param.ident.value, param_type.clone());
        params.push(param_type);
    }

    if let Some(ident) = identifier {
        scope.set(
            &ident.value,
            VariableType::Func {
                params: params.clone(),
                return_value: Box::new(type_annotation.clone()),
                scope: scope.clone(),
            },
        )
    }

    let return_value = check_block(&fn_def.block, scope)?;

    if return_value != type_annotation {
        return Err(TypeError {
            message: format!(
                "Expected return type of '{}' but got '{}'",
                type_annotation, return_value
            ),
            position: fn_def.position,
        });
    }

    let function_scope = scope.clone();

    scope.pop();

    return Ok(VariableType::Func {
        params,
        return_value: Box::new(return_value),
        scope: function_scope,
    });
}

fn check_fn_call(fn_call: &FnCall, scope: &mut Scope) -> TypecheckResult {
    scope.push();

    let ident = &fn_call.ident.value;

    let Some(fn_def) = scope.find(ident) else {
        return Err(TypeError {
            message: format!("Call to undefined function '{}'", ident),
            position: fn_call.position,
        });
    };

    let VariableType::Func { params, return_value, .. } = fn_def else {
        return Err(TypeError {
            message: format!("Trying to call an invalid function '{}'", ident),
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

    for (i, param) in params.iter().enumerate() {
        let call_param_type = check_expression(None, &fn_call.params[i], scope)?;
        if param != &call_param_type && param != &VariableType::Any {
            return Err(TypeError {
                message: format!(
                    "Invalid type of parameter! Expected '{}' but got '{}'",
                    param, call_param_type
                ),
                position: fn_call.params[i].position(),
            });
        }
    }

    scope.pop();

    Ok(*return_value)
}

fn check_binary_operation(binary_operation: &BinaryOp, scope: &mut Scope) -> TypecheckResult {
    let position = binary_operation.position;

    let lhs = &binary_operation.lhs;
    let rhs = &binary_operation.rhs;

    let l_type = check_expression(None, lhs, scope)?;
    let r_type = check_expression(None, rhs, scope)?;

    match binary_operation.verb {
        BinaryVerb::Equal => {
            if l_type != r_type {
                return Err(TypeError {
                    message: format!(
                        "Left and right value of binary operation do not match! ('{}' and '{}')",
                        l_type, r_type
                    ),
                    position,
                });
            }
            return Ok(VariableType::Bool);
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
            return Ok(VariableType::Bool);
        }
        BinaryVerb::Plus | BinaryVerb::Minus | BinaryVerb::Times => {
            if l_type != VariableType::Int {
                return Err(TypeError {
                    message: format!(
                        "Left value of numeric binary operation has to be of type Int. Found '{}'",
                        l_type
                    ),
                    position: lhs.position(),
                });
            } else if r_type != VariableType::Int {
                return Err(TypeError {
                    message: format!(
                        "Right value of numeric binary operation has to be of type Int. Found '{}'",
                        r_type
                    ),
                    position: rhs.position(),
                });
            }

            return Ok(VariableType::Int);
        }
    }
}
