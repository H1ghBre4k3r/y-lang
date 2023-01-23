use std::collections::HashMap;

use crate::ast::{
    Assignment, Ast, BinaryOp, BinaryVerb, Block, Declaration, Expression, FnCall, Ident, If,
    Integer, Intrinsic, Statement, Str,
};

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
        println!("{} {}", name, value.as_str());
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

    fn run_statement(statement: &Statement, scope: &mut Scope) {
        match &statement {
            Statement::Expression(expression) => {
                Self::run_expression(expression, scope);
            }
            Statement::Intrinsic(intrinsic) => {
                Self::run_intrinsic(intrinsic, scope);
            }
        }
    }

    fn run_intrinsic(intrinsic: &Intrinsic, scope: &mut Scope) {
        match intrinsic {
            Intrinsic::If(if_statement) => Self::run_if(if_statement, scope),
            Intrinsic::Declaration(declaration) => Self::run_declaration(declaration, scope),
            Intrinsic::Assignment(assignment) => Self::run_assignment(assignment, scope),
        }
    }

    fn run_if(if_statement: &If, scope: &mut Scope) {
        let condition = &if_statement.condition;
        let VariableType::Bool(condition) = Self::run_expression(condition, scope) else {
            let position = condition.position();
            unreachable!(
                "Invalid type of condition '{:?}' at {}:{}",
                condition, position.0, position.1
            );
        };

        if condition {
            Self::run_block(&if_statement.if_block, scope);
        } else {
            if let Some(else_block) = &if_statement.else_block {
                Self::run_block(else_block, scope);
            }
        }
    }

    fn run_block(block: &Block, scope: &mut Scope) {
        scope.push();

        for statement in &block.block {
            Self::run_statement(statement, scope);
        }

        scope.pop();
    }

    fn run_declaration(declaration: &Declaration, scope: &mut Scope) {
        let value = Self::run_expression(&declaration.value, scope);

        scope.set(&declaration.ident.value, value);
    }

    fn run_assignment(assignment: &Assignment, scope: &mut Scope) {
        let value = Self::run_expression(&assignment.value, scope);

        scope.update(&assignment.ident.value, value);
    }

    fn run_expression(expression: &Expression, scope: &mut Scope) -> VariableType {
        match expression {
            Expression::Integer(Integer { value, .. }) => VariableType::Int(*value),
            Expression::Str(Str { value, .. }) => VariableType::Str(value.clone()),
            Expression::Ident(Ident { value, .. }) => {
                let Some(value) = scope.find(value) else {
                    unreachable!()
                };

                value
            }
            Expression::BinaryOp(binary_operation) => {
                Self::run_binary_operation(binary_operation, scope)
            }
            Expression::FnCall(fn_call) => Self::run_fn_call(fn_call, scope),
            Expression::FnDef(_) => todo!(),
        }
    }

    fn run_binary_operation(binary_operation: &BinaryOp, scope: &mut Scope) -> VariableType {
        let lhs = &binary_operation.lhs;
        let rhs = &binary_operation.rhs;

        let lhs = Self::run_expression(lhs, scope);
        let rhs = Self::run_expression(rhs, scope);

        match binary_operation.verb {
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

    fn run_fn_call(fn_call: &FnCall, scope: &mut Scope) -> VariableType {
        scope.push();

        let ident = &fn_call.ident;

        match ident.value.as_str() {
            "print" => {
                for param in &fn_call.params {
                    match param {
                        Expression::Ident(Ident { value: name, .. }) => {
                            let Some(value) = scope.find(&name) else {
                                unreachable!();
                            };
                            print!("{}", value.as_str());
                        }
                        Expression::Str(Str { value, .. }) => print!("{}", value),
                        Expression::BinaryOp(binary_operation) => {
                            print!(
                                "{}",
                                Self::run_binary_operation(&binary_operation, scope).as_str()
                            )
                        }
                        Expression::Integer(Integer { value, .. }) => print!("{}", value),
                        Expression::FnCall(_) => todo!(),
                        Expression::FnDef(_) => todo!(),
                    }
                }
            }
            _ => unreachable!(
                "Call to undefined function '{}' at {}:{}",
                ident.value, fn_call.position.0, fn_call.position.1
            ),
        }

        scope.pop();

        VariableType::Void
    }
}
