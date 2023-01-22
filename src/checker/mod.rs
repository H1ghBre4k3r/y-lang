use std::collections::HashMap;

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
    pub fn update(&mut self, name: &str, value: VariableType) {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();

        for scope in &mut scopes {
            if let Some(old_type) = scope.get(name) {
                if *old_type != value {
                    panic!(
                        "Could not assign variable '{}' with type '{:?}' a value of type '{:?}'",
                        name, old_type, value
                    );
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
        unreachable!("Invalid if statement: '{:?}'", if_statement);
    };

    let condition_type = check_expression(condition.as_ref(), scope);

    if condition_type != VariableType::Bool {
        panic!("Invalid type of condition: '{:?}'", condition_type);
    }

    check_block(if_block.as_ref(), scope);

    if let Some(else_block) = else_block {
        check_block(else_block.as_ref(), scope);
    }
}

fn check_block(block: &AstNode, scope: &mut Scope) {
    scope.push();

    let AstNode::Block { block: nodes, position } = block else {
        unreachable!("Invalid block statement: '{:?}", block);
    };

    for node in nodes {
        check_statement(&node, scope);
    }

    scope.pop();
}

fn check_declaration(declaration: &AstNode, scope: &mut Scope) {
    let AstNode::Declaration { ident, value, position: declaration_position } = declaration else {
        unreachable!("Invalid declaration: '{:?}'", declaration);
    };

    let AstNode::Ident { value: ident, position: ident_position } = ident.as_ref() else {
        unreachable!("Invalid identifier: '{:?}'", ident);
    };

    let declaration_type = check_expression(value.as_ref(), scope);

    scope.set(ident, declaration_type);
}

fn check_assignment(assignment: &AstNode, scope: &mut Scope) {
    let AstNode::Assignment { ident, value, position: assignment_position } = assignment else {
        unreachable!("Invalid assignment: '{:?}'", assignment);
    };

    let AstNode::Ident { value: ident, position: ident_position} = ident.as_ref() else {
        unreachable!("Invalid identifier: '{:?}'", ident);
    };

    if !scope.contains(ident) {
        panic!("'{}' has not beed defined!", ident);
    }

    let assignment_type = check_expression(value.as_ref(), scope);

    scope.update(ident, assignment_type);
}

fn check_expression(expression: &AstNode, scope: &mut Scope) -> VariableType {
    match expression {
        AstNode::BinaryOp { .. } => check_binary_operation(expression, scope),
        AstNode::Integer { .. } => VariableType::Int,
        AstNode::Str { .. } => VariableType::Str,
        AstNode::Ident { .. } => check_identifier(expression, scope),
        AstNode::FnCall { .. } => check_fn_call(expression, scope),
        _ => unreachable!("Invalid expression: '{:?}'", expression),
    }
}

fn check_identifier(identifier: &AstNode, scope: &mut Scope) -> VariableType {
    let AstNode::Ident {value: ident, position } = identifier else {
        unreachable!("Invalid identifier: '{:?}'", identifier);
    };

    match scope.find(ident) {
        Some(identifier_type) => identifier_type.clone(),
        None => panic!("Identifier '{}' not defined!", ident),
    }
}

fn check_fn_call(fn_call: &AstNode, scope: &mut Scope) -> VariableType {
    scope.push();

    // TODO: actually type check function call
    let AstNode::FnCall { ident, params, position: fn_call_position } = fn_call else {
        unreachable!("Invalid function call: '{:?}'", fn_call);
    };

    let AstNode::Ident { value: ident, position: ident_position } = ident.as_ref() else {
        unreachable!("Invalid identifier: '{:?}'", ident);
    };

    if !scope.contains(ident) {
        // TODO: Should this just overwrite the value?
        panic!("Function '{}' not defined!", ident);
    }

    scope.pop();

    VariableType::Void
}

fn check_binary_operation(binary_operation: &AstNode, scope: &mut Scope) -> VariableType {
    let AstNode::BinaryOp { verb, lhs, rhs, position } = binary_operation else {
        unreachable!("Invalid binary operation: '{:?}'", binary_operation);
    };

    let l_type = check_expression(lhs.as_ref(), scope);
    let r_type = check_expression(rhs.as_ref(), scope);

    match verb {
        BinaryVerb::Equal | BinaryVerb::LessThan | BinaryVerb::GreaterThan => {
            if l_type != r_type {
                panic!(
                    "Left and right value of binary operation do not match! ('{:?}' and '{:?}')",
                    l_type, r_type
                );
            }
            return VariableType::Bool;
        }
        BinaryVerb::Plus | BinaryVerb::Minus | BinaryVerb::Times => {
            if l_type != VariableType::Int {
                panic!(
                    "Left value of numeric binary operation has to be of type Int. Found '{:?}'",
                    l_type
                );
            } else if r_type != VariableType::Int {
                panic!(
                    "Right value of numeric binary operation has to be of type Int. Found '{:?}'",
                    r_type
                );
            }

            return VariableType::Int;
        }
    }
}
