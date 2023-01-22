use std::collections::HashMap;

use crate::ast::{Ast, AstNode, BinaryVerb};

pub struct Interpreter {
    ast: Ast,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum VariableType {
    Void,
    Bool(bool),
    Str(String),
    Int(i64),
    Func {
        name: String,
        return_value: Box<VariableType>,
    },
}

impl VariableType {
    pub fn as_str(&self) -> String {
        match self {
            Self::Void => "void".to_owned(),
            Self::Bool(value) => format!("{}", value),
            Self::Str(value) => format!("{}", value),
            Self::Int(value) => format!("{}", value),
            _ => unimplemented!(),
        }
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
            if scope.contains_key(name) {
                scope.insert(name.to_owned(), value);

                break;
            }
        }

        scopes.reverse();
        self.scope_stack = scopes;
    }
}

impl Interpreter {
    pub fn from_ast(ast: Ast) -> Self {
        Self { ast }
    }

    pub fn run(&self) {
        let nodes = self.ast.nodes();

        // TODO: Maybe move this into struct as field
        let mut scope = Scope::default();
        scope.push();

        for node in nodes {
            Self::run_statement(&node, &mut scope);
        }
    }

    fn run_statement(statement: &AstNode, scope: &mut Scope) {
        match &statement {
            AstNode::Declaration { .. } => Self::run_declaration(&statement, scope),
            AstNode::Assignment { .. } => Self::run_assignment(&statement, scope),
            AstNode::If { .. } => Self::run_if(statement, scope),
            _ => {
                Self::run_expression(statement, scope);
            }
        }
    }

    fn run_if(if_statement: &AstNode, scope: &mut Scope) {
        let AstNode::If { condition, if_block, else_block } = if_statement else {
            unreachable!()
        };

        let VariableType::Bool(condition) = Self::run_expression(condition.as_ref(), scope) else {
            unreachable!();
        };

        if condition {
            Self::run_block(if_block.as_ref(), scope);
        } else {
            if let Some(else_block) = else_block {
                Self::run_block(else_block.as_ref(), scope);
            }
        }
    }

    fn run_block(block: &AstNode, scope: &mut Scope) {
        scope.push();

        let AstNode::Block(nodes) = block else {
            unreachable!()
        };

        for node in nodes {
            Self::run_statement(&node, scope);
        }

        scope.pop();
    }

    fn run_declaration(declaration: &AstNode, scope: &mut Scope) {
        let AstNode::Declaration { ident, value } = declaration else {
            unreachable!()
        };

        let AstNode::Ident(ident) = ident.as_ref() else {
            unreachable!()
        };

        let value = Self::run_expression(value.as_ref(), scope);

        scope.set(ident, value);
    }

    fn run_assignment(assignment: &AstNode, scope: &mut Scope) {
        let AstNode::Assignment { ident, value } = assignment else {
            unreachable!()
        };

        let AstNode::Ident(ident) = ident.as_ref() else {
            unreachable!()
        };

        let value = Self::run_expression(value.as_ref(), scope);

        scope.update(ident, value);
    }

    fn run_expression(expression: &AstNode, scope: &mut Scope) -> VariableType {
        match expression {
            AstNode::Integer(value) => VariableType::Int(*value),
            AstNode::Str(value) => VariableType::Str(value.clone()),
            AstNode::Ident(value) => {
                let Some(value) = scope.find(value) else {
                    unreachable!()
                };

                value
            }
            AstNode::BinaryOp { .. } => Self::run_binary_operation(expression, scope),
            AstNode::FnCall { .. } => Self::run_fn_call(expression, scope),
            _ => unreachable!(),
        }
    }

    fn run_binary_operation(expression: &AstNode, scope: &mut Scope) -> VariableType {
        let AstNode::BinaryOp { verb, lhs, rhs } = expression else {
            unreachable!()
        };

        let lhs = Self::run_expression(lhs.as_ref(), scope);
        let rhs = Self::run_expression(rhs.as_ref(), scope);

        match verb {
            BinaryVerb::Equal => VariableType::Bool(lhs == rhs),
            BinaryVerb::GreaterThan => VariableType::Bool(lhs > rhs),
            BinaryVerb::LessThan => VariableType::Bool(lhs < rhs),
            BinaryVerb::Plus => {
                let (VariableType::Int(lhs), VariableType::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableType::Int(lhs + rhs)
            }
            BinaryVerb::Minus => {
                let (VariableType::Int(lhs), VariableType::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableType::Int(lhs - rhs)
            }
            BinaryVerb::Times => {
                let (VariableType::Int(lhs), VariableType::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableType::Int(lhs * rhs)
            }
        }
    }

    fn run_fn_call(fn_call: &AstNode, scope: &mut Scope) -> VariableType {
        let AstNode::FnCall { ident, params } = fn_call else{
            unreachable!()
        };

        let AstNode::Ident(ident) = ident.as_ref() else {
            unreachable!()
        };

        scope.push();

        match ident.as_str() {
            "print" => {
                for param in params {
                    match param {
                        AstNode::Ident(name) => {
                            let Some(value) = scope.find(name) else {
                                unreachable!();
                            };
                            print!("{}", value.as_str());
                        }
                        AstNode::Str(value) => print!("{}", value),
                        AstNode::BinaryOp { .. } => {
                            print!("{}", Self::run_binary_operation(param, scope).as_str())
                        }
                        AstNode::Integer(value) => print!("{}", value),
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!("Function '{}' not defined!", ident),
        }

        scope.pop();

        VariableType::Void
    }
}
