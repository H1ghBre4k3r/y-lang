use std::collections::HashMap;

use log::error;

use crate::ast::{Ast, AstNode, BinaryVerb};

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
    pub fn update(&mut self, name: &str, value: VariableType, position: &(usize, usize)) {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();

        for scope in &mut scopes {
            if let Some(old_type) = scope.get(name) {
                if *old_type != value {
                    error!(
                        "Could not assign variable '{}' with type '{:?}' a value of type '{:?}' at {}:{}",
                        name, old_type, value, position.0, position.1
                    );
                    std::process::exit(-1);
                }
                scope.insert(name.to_owned(), value);

                break;
            }
        }

        scopes.reverse();
        self.scope_stack = scopes;
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

pub fn check_ast(ast: &Ast) {
    let nodes = ast.nodes();

    let mut scope = setup_scope();

    for node in nodes {
        check_statement(&node, &mut scope);
    }
}

fn check_statement(statement: &AstNode, scope: &mut Scope) {
    match &statement {
        AstNode::Declaration { .. } => check_declaration(&statement, scope),
        AstNode::Assignment { .. } => check_assignment(&statement, scope),
        AstNode::If { .. } => check_if(&statement, scope),
        _ => {
            check_expression(&statement, scope);
        }
    }
}

fn check_if(if_statement: &AstNode, scope: &mut Scope) {
    let AstNode::If { condition, if_block, else_block, position } = if_statement else {
        let position = if_statement.position();
        error!("Invalid if statement '{:?}' at {}:{}", if_statement, position.0, position.1);
        std::process::exit(-1);
    };

    let condition_type = check_expression(condition.as_ref(), scope);

    if condition_type != VariableType::Bool {
        error!(
            "Invalid type of condition '{:?}' at {}:{}",
            condition_type, position.0, position.1
        );
        std::process::exit(-1);
    }

    check_block(if_block.as_ref(), scope);

    if let Some(else_block) = else_block {
        check_block(else_block.as_ref(), scope);
    }
}

fn check_block(block: &AstNode, scope: &mut Scope) {
    scope.push();

    let AstNode::Block { block: nodes, .. } = block else {
        let position = block.position();
        error!("Invalid block statement '{:?}' at {}:{}", block, position.0, position.1);
        std::process::exit(-1);
    };

    for node in nodes {
        check_statement(&node, scope);
    }

    scope.pop();
}

fn check_declaration(declaration: &AstNode, scope: &mut Scope) {
    let AstNode::Declaration { ident, value,..} = declaration else {
        let position = declaration.position();
        error!("Invalid declaration '{:?}' at {}:{}", declaration, position.0, position.1);
        std::process::exit(-1);
    };

    let AstNode::Ident { value: ident, ..} = ident.as_ref() else {
        let position = ident.position();
        error!("Invalid identifier '{:?}' at {}:{}", ident, position.0, position.1);
        std::process::exit(-1);
    };

    let declaration_type = check_expression(value.as_ref(), scope);

    scope.set(ident, declaration_type);
}

fn check_assignment(assignment: &AstNode, scope: &mut Scope) {
    let AstNode::Assignment { ident, value, position: assignment_position } = assignment else {
        let position = assignment.position();
        error!("Invalid assignment '{:?}' at {}:{}", assignment, position.0, position.1);
        std::process::exit(-1);
    };

    let AstNode::Ident { value: ident, position} = ident.as_ref() else {
        let position = ident.position();
        error!("Invalid identifier '{:?}' at {}:{}", ident, position.0, position.1);
        std::process::exit(-1);
    };

    if !scope.contains(ident) {
        error!(
            "Undefined identifier '{}' at {}:{}",
            ident, position.0, position.1
        );
        std::process::exit(-1);
    }

    let assignment_type = check_expression(value.as_ref(), scope);

    scope.update(ident, assignment_type, assignment_position);
}

fn check_expression(expression: &AstNode, scope: &mut Scope) -> VariableType {
    let position = expression.position();

    match expression {
        AstNode::BinaryOp { .. } => check_binary_operation(expression, scope),
        AstNode::Integer { .. } => VariableType::Int,
        AstNode::Str { .. } => VariableType::Str,
        AstNode::Ident { .. } => check_identifier(expression, scope),
        AstNode::FnCall { .. } => check_fn_call(expression, scope),
        _ => {
            error!(
                "Invalid expression '{:?}' at {}:{}",
                expression, position.0, position.1
            );
            std::process::exit(-1);
        }
    }
}

fn check_identifier(identifier: &AstNode, scope: &mut Scope) -> VariableType {
    let AstNode::Ident {value: ident, position } = identifier else {
        let position = identifier.position();
        error!("Invalid identifier '{:?}' at {}:{}", identifier, position.0, position.1);
        std::process::exit(-1);
    };

    match scope.find(ident) {
        Some(identifier_type) => identifier_type.clone(),
        None => {
            error!(
                "Undefined identifier '{}' at {}:{}",
                ident, position.0, position.1
            );
            std::process::exit(-1);
        }
    }
}

fn check_fn_call(fn_call: &AstNode, scope: &mut Scope) -> VariableType {
    scope.push();

    // TODO: actually type check function call
    let AstNode::FnCall { ident, params: _params, position: fn_call_position } = fn_call else {
        let position = fn_call.position();
        error!("Invalid function call '{:?}' at {}:{}", fn_call, position.0, position.1);
        std::process::exit(-1);
    };

    let AstNode::Ident { value: ident,.. } = ident.as_ref() else {
        let position = ident.position();
        error!("Invalid identifier '{:?}' at {}:{}", ident, position.0, position.1);
        std::process::exit(-1);
    };

    if !scope.contains(ident) {
        // TODO: Should this just overwrite the value?
        error!(
            "Call to undefined function '{}' at {}:{}",
            ident, fn_call_position.0, fn_call_position.1
        );
        std::process::exit(-1);
    }

    scope.pop();

    VariableType::Void
}

fn check_binary_operation(binary_operation: &AstNode, scope: &mut Scope) -> VariableType {
    let AstNode::BinaryOp { verb, lhs, rhs, position } = binary_operation else {
        let position = binary_operation.position();
        error!("Invalid binary operation: '{:?}' at {}:{}", binary_operation,  position.0, position.1);
        std::process::exit(-1);
    };

    let l_type = check_expression(lhs.as_ref(), scope);
    let r_type = check_expression(rhs.as_ref(), scope);

    match verb {
        BinaryVerb::Equal | BinaryVerb::LessThan | BinaryVerb::GreaterThan => {
            if l_type != r_type {
                error!(
                    "Left and right value of binary operation do not match! ('{:?}' and '{:?}') at {}:{}",
                    l_type, r_type, position.0, position.1
                );
                std::process::exit(-1);
            }
            return VariableType::Bool;
        }
        BinaryVerb::Plus | BinaryVerb::Minus | BinaryVerb::Times => {
            if l_type != VariableType::Int {
                let position = lhs.position();
                error!(
                    "Left value of numeric binary operation has to be of type Int. Found '{:?}' at {}:{}",
                    l_type, position.0, position.1
                );
                std::process::exit(-1);
            } else if r_type != VariableType::Int {
                let position = rhs.position();
                error!(
                    "Right value of numeric binary operation has to be of type Int. Found '{:?}' at {}:{}",
                    r_type, position.0, position.1
                );
                std::process::exit(-1);
            }

            return VariableType::Int;
        }
    }
}
