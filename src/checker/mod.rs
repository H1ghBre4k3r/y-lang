use std::collections::HashMap;

use crate::ast::{Ast, AstNode, BinaryVerb};

#[derive(Debug, Clone, PartialEq, Eq)]
enum AssignmentType {
    Void,
    Bool,
    Str,
    Int,
    Func {
        params: Vec<AssignmentType>,
        return_value: Box<AssignmentType>,
    },
}

// TODO: This should be a vector of scopes
type Scope = HashMap<String, AssignmentType>;

fn setup_scope() -> Scope {
    let mut scope = Scope::new();

    scope.insert(
        "print".to_owned(),
        AssignmentType::Func {
            params: vec![],
            return_value: Box::new(AssignmentType::Void),
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
        AstNode::Assignment { .. } => check_assignment(&statement, scope),
        AstNode::If { .. } => check_if(&statement, scope),
        _ => {
            check_expression(&statement, scope);
        }
    }
}

fn check_if(if_statement: &AstNode, scope: &mut Scope) {
    let AstNode::If { condition, if_block, else_block } = if_statement else {
        unreachable!("Invalid if statement: '{:?}'", if_statement);
    };

    let condition_type = check_expression(condition.as_ref(), scope);

    if condition_type != AssignmentType::Bool {
        panic!("Invalid type of condition: '{:?}'", condition_type);
    }

    check_block(if_block.as_ref(), scope);

    if let Some(else_block) = else_block {
        check_block(else_block.as_ref(), scope);
    }
}

fn check_block(block: &AstNode, scope: &mut Scope) {
    let AstNode::Block(nodes) = block else {
        unreachable!("Invalid block statement: '{:?}", block);
    };

    for node in nodes {
        check_statement(&node, scope);
    }
}

fn check_assignment(assignment: &AstNode, scope: &mut Scope) {
    let AstNode::Assignment { ident, value } = assignment else {
        unreachable!("Invalid assignment: '{:?}'", assignment);
    };

    let AstNode::Ident(ident) = ident.as_ref() else {
        unreachable!("Invalid identifier: '{:?}'", ident);
    };

    if scope.contains_key(ident) {
        // TODO: Should this just overwrite the value?
        panic!("'{}' as already been defined!", ident);
    }

    let assignment_type = check_expression(value.as_ref(), scope);

    scope.insert(ident.to_owned(), assignment_type);
}

fn check_expression(expression: &AstNode, scope: &mut Scope) -> AssignmentType {
    match expression {
        AstNode::BinaryOp { .. } => check_binary_operation(expression, scope),
        AstNode::Integer(..) => AssignmentType::Int,
        AstNode::Str(..) => AssignmentType::Str,
        AstNode::Ident(..) => check_identifier(expression, scope),
        AstNode::FnCall { .. } => check_fn_call(expression, scope),
        _ => unreachable!("Invalid expression: '{:?}'", expression),
    }
}

fn check_identifier(identifier: &AstNode, scope: &mut Scope) -> AssignmentType {
    let AstNode::Ident(ident) = identifier else {
        unreachable!("Invalid identifier: '{:?}'", identifier);
    };

    match scope.get(ident) {
        Some(identifier_type) => identifier_type.clone(),
        None => panic!("Identifier '{}' not defined!", ident),
    }
}

fn check_fn_call(fn_call: &AstNode, scope: &mut Scope) -> AssignmentType {
    // TODO: actually type check function call
    let AstNode::FnCall { ident, params } = fn_call else {
        unreachable!("Invalid function call: '{:?}'", fn_call);
    };

    let AstNode::Ident(ident) = ident.as_ref() else {
        unreachable!("Invalid identifier: '{:?}'", ident);
    };

    if !scope.contains_key(ident) {
        // TODO: Should this just overwrite the value?
        panic!("Function '{}' not defined!", ident);
    }

    AssignmentType::Void
}

fn check_binary_operation(binary_operation: &AstNode, scope: &mut Scope) -> AssignmentType {
    let AstNode::BinaryOp { verb, lhs, rhs } = binary_operation else {
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
            return AssignmentType::Bool;
        }
        BinaryVerb::Plus | BinaryVerb::Minus | BinaryVerb::Times => {
            if l_type != AssignmentType::Int {
                panic!(
                    "Left value of numeric binary operation has to be of type Int. Found '{:?}'",
                    l_type
                );
            } else if r_type != AssignmentType::Int {
                panic!(
                    "Right value of numeric binary operation has to be of type Int. Found '{:?}'",
                    r_type
                );
            }

            return AssignmentType::Int;
        }
    }
}
