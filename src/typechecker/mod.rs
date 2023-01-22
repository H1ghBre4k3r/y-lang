mod error;

use std::{collections::HashMap, fmt::Display, str::FromStr};

use crate::ast::{Ast, AstNode, BinaryVerb};

use self::error::TypeError;

type TypecheckResult = Result<VariableType, TypeError>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum VariableType {
    Void,
    Bool,
    Str,
    Int,
    Func {
        params: Vec<VariableType>,
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
            Func { .. } => todo!(),
        })
    }
}

#[derive(Default, Debug)]
struct Scope {
    scope_stack: Vec<HashMap<String, VariableType>>,
}

impl Scope {
    /// Find a value/reference in this scope by iterating over the scopes from back to front.
    pub fn find(&self, name: &str) -> Option<VariableType> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in scopes {
            if let Some(variable) = scope.get(name) {
                return Some(variable.clone());
            }
        }

        return None;
    }

    /// Check, if a variable with a given name is present.
    pub fn contains(&self, name: &str) -> bool {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in scopes {
            if scope.contains_key(name) {
                return true;
            }
        }

        false
    }

    /// Push a new scope frame.
    pub fn push(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Pop the last scope frame.
    pub fn pop(&mut self) {
        self.scope_stack.pop();
    }

    /// Create a new variable on the current scope.
    pub fn set(&mut self, name: &str, value: VariableType) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name.to_owned(), value);
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
            params: vec![],
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

fn check_statement(statement: &AstNode, scope: &mut Scope) -> TypecheckResult {
    Ok(match &statement {
        AstNode::Declaration { .. } => check_declaration(&statement, scope)?,
        AstNode::Assignment { .. } => check_assignment(&statement, scope)?,
        AstNode::If { .. } => check_if(&statement, scope)?,
        _ => check_expression(&statement, scope)?,
    })
}

fn check_if(if_statement: &AstNode, scope: &mut Scope) -> TypecheckResult {
    let AstNode::If { condition, if_block, else_block, position } = if_statement else {
        return Err(TypeError {
            message: format!("Invalid if statement '{:?}'", if_statement),
            position: if_statement.position(),
        });
    };

    let condition_type = check_expression(condition.as_ref(), scope)?;

    if condition_type != VariableType::Bool {
        return Err(TypeError {
            message: format!("Invalid tye of condition '{:?}'", condition_type),
            position: position.to_owned(),
        });
    }

    let if_return_type = check_block(if_block.as_ref(), scope)?;

    if let Some(else_block) = else_block {
        let else_return_type = check_block(else_block.as_ref(), scope)?;

        if if_return_type != else_return_type {
            return Err(TypeError {
                message: format!(
                    "Return type mismatch of if-else. Got '{}' and '{}'",
                    if_return_type, else_return_type
                ),
                position: position.to_owned(),
            });
        }
    }

    Ok(if_return_type)
}

fn check_block(block: &AstNode, scope: &mut Scope) -> TypecheckResult {
    scope.push();

    let AstNode::Block { block: nodes, .. } = block else {
        return Err(TypeError {
            message: format!("Invalid block '{:?}'", block),
            position: block.position(),
        });
    };

    let mut last_return = VariableType::Void;

    for node in nodes {
        last_return = check_statement(&node, scope)?;
    }

    scope.pop();

    Ok(last_return)
}

fn check_declaration(declaration: &AstNode, scope: &mut Scope) -> TypecheckResult {
    let AstNode::Declaration { ident, value,..} = declaration else {
        return Err(TypeError {
            message: format!("Invalid declaration '{:?}'", declaration),
            position: declaration.position(),
        });
    };

    let AstNode::Ident { value: ident, ..} = ident.as_ref() else {
        return Err(TypeError {
            message: format!("Invalid identifier '{:?}'", ident),
            position: ident.position(),
        });
    };

    let declaration_type = check_expression(value.as_ref(), scope)?;

    scope.set(ident, declaration_type);

    Ok(VariableType::Void)
}

fn check_assignment(assignment: &AstNode, scope: &mut Scope) -> TypecheckResult {
    let AstNode::Assignment { ident, value, position: assignment_position } = assignment else {
        return Err(TypeError {
            message: format!("Invalid assignment '{:?}'", assignment),
            position: assignment.position(),
        });
    };

    let AstNode::Ident { value: ident, position} = ident.as_ref() else {
        return Err(TypeError {
            message: format!("Invalid identifier '{:?}'", ident),
            position: ident.position(),
        });
    };

    if !scope.contains(ident) {
        return Err(TypeError {
            message: format!("Undefined identifier '{}'", ident),
            position: position.to_owned(),
        });
    }

    let assignment_type = check_expression(value.as_ref(), scope)?;

    scope.update(ident, assignment_type, assignment_position)?;

    Ok(VariableType::Void)
}

fn check_expression(expression: &AstNode, scope: &mut Scope) -> TypecheckResult {
    let position = expression.position();

    match expression {
        AstNode::BinaryOp { .. } => check_binary_operation(expression, scope),
        AstNode::Integer { .. } => Ok(VariableType::Int),
        AstNode::Str { .. } => Ok(VariableType::Str),
        AstNode::Ident { .. } => check_identifier(expression, scope),
        AstNode::FnCall { .. } => check_fn_call(expression, scope),
        AstNode::FnDef { .. } => check_fn_def(expression, scope),
        _ => {
            return Err(TypeError {
                message: format!("Invalid expression '{:?}'", expression),
                position,
            });
        }
    }
}

fn check_identifier(identifier: &AstNode, scope: &mut Scope) -> TypecheckResult {
    let AstNode::Ident {value: ident, position } = identifier else {
        return Err(TypeError {
            message: format!("Invalid identifier '{:?}'", identifier),
            position: identifier.position(),
        });
    };

    match scope.find(ident) {
        Some(identifier_type) => Ok(identifier_type.clone()),
        None => {
            return Err(TypeError {
                message: format!("Undefined identifier '{}'", ident),
                position: position.to_owned(),
            });
        }
    }
}

fn check_fn_def(fn_def: &AstNode, scope: &mut Scope) -> TypecheckResult {
    let AstNode::FnDef { params, type_annotation, block, position } = fn_def else {
        return Err(TypeError {
            message: format!("Invalid function definition '{:?}'", fn_def),
            position: fn_def.position(),
        })
    };

    check_block(block, scope)?;

    todo!();
}

fn check_fn_call(fn_call: &AstNode, scope: &mut Scope) -> TypecheckResult {
    scope.push();

    // TODO: actually type check function call
    let AstNode::FnCall { ident, params: _params, position: fn_call_position } = fn_call else {
        return Err(TypeError {
            message: format!("Invalid function call '{:?}'", fn_call),
            position: fn_call.position(),
        })
    };

    let AstNode::Ident { value: ident,.. } = ident.as_ref() else {
        return Err(TypeError {
            message: format!("Invalid identifier '{:?}'", ident),
            position: ident.position(),
        })
    };

    if !scope.contains(ident) {
        return Err(TypeError {
            message: format!("Call to undefined function '{}'", ident),
            position: fn_call_position.clone(),
        });
    }

    scope.pop();

    Ok(VariableType::Void)
}

fn check_binary_operation(binary_operation: &AstNode, scope: &mut Scope) -> TypecheckResult {
    let AstNode::BinaryOp { verb, lhs, rhs, position } = binary_operation else {
        return Err(TypeError {
            message: format!("Invalid binar operation '{:?}'", binary_operation),
            position: binary_operation.position()
        });
    };

    let l_type = check_expression(lhs.as_ref(), scope)?;
    let r_type = check_expression(rhs.as_ref(), scope)?;

    match verb {
        BinaryVerb::Equal | BinaryVerb::LessThan | BinaryVerb::GreaterThan => {
            if l_type != r_type {
                return Err(TypeError {
                    message: format!("Left and right value of binary operation do not match! ('{:?}' and '{:?}')", l_type, r_type),
                    position: position.clone()
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
