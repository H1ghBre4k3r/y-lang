use std::{collections::HashMap, fmt::Display};

use crate::ast::{
    Assignment, Ast, BinaryOp, BinaryVerb, Block, Declaration, Expression, FnCall, FnDef, Ident,
    If, Integer, Intrinsic, Statement, Str,
};

pub struct Interpreter {
    ast: Ast,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum VariableValue {
    Void,
    Bool(bool),
    Str(String),
    Int(i64),
    Func { params: Vec<String>, block: Block },
}

impl Display for VariableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_representation = match self {
            Self::Void => "void".to_owned(),
            Self::Bool(value) => format!("{}", value),
            Self::Str(value) => format!("{}", value),
            Self::Int(value) => format!("{}", value),
            _ => unimplemented!(),
        };
        f.write_str(&str_representation)
    }
}

#[derive(Default, Debug)]
struct Scope {
    scope_stack: Vec<HashMap<String, VariableValue>>,
}

impl Scope {
    /// Find a value/reference in this scope by iterating over the scopes from back to front.
    pub fn find(&self, name: &str) -> Option<VariableValue> {
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
    pub fn set(&mut self, name: &str, value: VariableValue) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name.to_owned(), value);
        }
    }

    /// Update a value of an already present variable.
    pub fn update(&mut self, name: &str, value: VariableValue) {
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

    fn run_statement(statement: &Statement, scope: &mut Scope) -> VariableValue {
        match &statement {
            Statement::Expression(expression) => Self::run_expression(expression, scope),
            Statement::Intrinsic(intrinsic) => Self::run_intrinsic(intrinsic, scope),
        }
    }

    fn run_intrinsic(intrinsic: &Intrinsic, scope: &mut Scope) -> VariableValue {
        match intrinsic {
            Intrinsic::Declaration(declaration) => Self::run_declaration(declaration, scope),
            Intrinsic::Assignment(assignment) => Self::run_assignment(assignment, scope),
        }
    }

    fn run_if(if_statement: &If, scope: &mut Scope) -> VariableValue {
        let condition = &if_statement.condition;
        let VariableValue::Bool(condition) = Self::run_expression(condition, scope) else {
            let position = condition.position();
            unreachable!(
                "Invalid type of condition '{:?}' at {}:{}",
                condition, position.0, position.1
            );
        };

        if condition {
            return Self::run_block(&if_statement.if_block, scope);
        } else {
            if let Some(else_block) = &if_statement.else_block {
                return Self::run_block(else_block, scope);
            }
            return VariableValue::Void;
        }
    }

    fn run_block(block: &Block, scope: &mut Scope) -> VariableValue {
        scope.push();

        let mut return_value = VariableValue::Void;

        for statement in &block.block {
            return_value = Self::run_statement(statement, scope);
        }

        scope.pop();

        return_value
    }

    fn run_declaration(declaration: &Declaration, scope: &mut Scope) -> VariableValue {
        let value = Self::run_expression(&declaration.value, scope);

        scope.set(&declaration.ident.value, value);
        VariableValue::Void
    }

    fn run_assignment(assignment: &Assignment, scope: &mut Scope) -> VariableValue {
        let value = Self::run_expression(&assignment.value, scope);

        scope.update(&assignment.ident.value, value);
        VariableValue::Void
    }

    fn run_expression(expression: &Expression, scope: &mut Scope) -> VariableValue {
        match expression {
            Expression::If(if_statement) => Self::run_if(if_statement, scope),
            Expression::Integer(Integer { value, .. }) => VariableValue::Int(*value),
            Expression::Str(Str { value, .. }) => VariableValue::Str(value.clone()),
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
            Expression::Block(block) => Self::run_block(block, scope),
            Expression::FnDef(fn_def) => Self::run_fn_def(fn_def, scope),
        }
    }

    fn run_binary_operation(binary_operation: &BinaryOp, scope: &mut Scope) -> VariableValue {
        let lhs = &binary_operation.lhs;
        let rhs = &binary_operation.rhs;

        let lhs = Self::run_expression(lhs, scope);
        let rhs = Self::run_expression(rhs, scope);

        match binary_operation.verb {
            BinaryVerb::Equal => VariableValue::Bool(lhs == rhs),
            BinaryVerb::GreaterThan => VariableValue::Bool(lhs > rhs),
            BinaryVerb::LessThan => VariableValue::Bool(lhs < rhs),
            BinaryVerb::Plus => {
                let (VariableValue::Int(lhs), VariableValue::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableValue::Int(lhs + rhs)
            }
            BinaryVerb::Minus => {
                let (VariableValue::Int(lhs), VariableValue::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableValue::Int(lhs - rhs)
            }
            BinaryVerb::Times => {
                let (VariableValue::Int(lhs), VariableValue::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableValue::Int(lhs * rhs)
            }
        }
    }

    fn run_fn_def(fn_def: &FnDef, _scope: &mut Scope) -> VariableValue {
        let mut params = vec![];

        for param in &fn_def.params {
            params.push(param.ident.value.clone());
        }

        VariableValue::Func {
            params,
            block: fn_def.block.clone(),
        }
    }

    fn run_fn_call(fn_call: &FnCall, scope: &mut Scope) -> VariableValue {
        scope.push();

        let ident = &fn_call.ident;

        let return_value = match ident.value.as_str() {
            "print" => {
                for param in &fn_call.params {
                    match param {
                        Expression::Ident(Ident { value: name, .. }) => {
                            let Some(value) = scope.find(&name) else {
                                unreachable!();
                            };
                            print!("{}", value);
                        }
                        Expression::Str(Str { value, .. }) => print!("{}", value),
                        Expression::BinaryOp(binary_operation) => {
                            print!("{}", Self::run_binary_operation(&binary_operation, scope))
                        }
                        Expression::Integer(Integer { value, .. }) => print!("{}", value),
                        Expression::If(if_statement) => {
                            println!("{}", Self::run_if(if_statement, scope))
                        }
                        Expression::Block(block) => {
                            println!("{}", Self::run_block(block, scope))
                        }
                        Expression::FnCall(_) => todo!(),
                        Expression::FnDef(_) => todo!(),
                    }
                }
                VariableValue::Void
            }
            ident => {
                let Some(fn_def) = scope.find(ident) else {
                    unreachable!();
                };

                let VariableValue::Func { params, block } = fn_def else {
                    unreachable!();  
                };

                for (i, param) in fn_call.params.iter().enumerate() {
                    let param_name = &params[i];
                    let param_value = Self::run_expression(param, scope);

                    scope.set(&param_name, param_value);
                }

                Self::run_block(&block, scope)
            }
        };

        scope.pop();

        return_value
    }
}
