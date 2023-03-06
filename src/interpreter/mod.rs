use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    ast::{
        Assignment, Ast, BinaryExpr, BinaryOp, Block, Boolean, Call, CompilerDirective, Definition,
        Expression, FnDef, Ident, If, Import, Integer, Intrinsic, PostfixExpr, PostfixOp,
        PrefixExpr, PrefixOp, Statement, Str,
    },
    loader::Modules,
    typechecker::TypeInfo,
};

pub struct Interpreter {
    ast: Ast<TypeInfo>,
    modules: Modules<TypeInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VariableValue {
    Void,
    Bool(bool),
    Str(String),
    Int(i64),
    Func {
        params: Vec<String>,
        block: Block<TypeInfo>,
        scope: Scope,
    },
}

impl Display for VariableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_representation = match self {
            Self::Void => "void".to_owned(),
            Self::Bool(value) => format!("{value}"),
            Self::Str(value) => value.to_string(),
            Self::Int(value) => format!("{value}"),
            Self::Func { params, .. } => format!("{params:?} {{ .. }}"),
        };
        f.write_str(&str_representation)
    }
}

type ScopeFrame = HashMap<String, VariableValue>;

type ScopeFrameReference = Rc<RefCell<ScopeFrame>>;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct Scope {
    scope_stack: Vec<ScopeFrameReference>,
}

impl Scope {
    /// Find a value/reference in this scope by iterating over the scopes from back to front.
    pub fn find(&self, name: &str) -> Option<VariableValue> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in scopes {
            if let Some(variable) = scope.borrow().get(name) {
                return Some(variable.clone());
            }
        }

        None
    }

    /// Push a new scope frame.
    pub fn push(&mut self) {
        self.scope_stack.push(Rc::new(RefCell::new(HashMap::new())));
    }

    /// Pop the last scope frame.
    pub fn pop(&mut self) {
        self.scope_stack.pop();
    }

    /// Create a new variable on the current scope.
    pub fn set(&mut self, name: &str, value: VariableValue) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.borrow_mut().insert(name.to_owned(), value);
        }
    }

    /// Update a value of an already present variable.
    pub fn update(&mut self, name: &str, value: VariableValue) {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();

        for scope in &mut scopes {
            let mut scope = scope.borrow_mut();
            if scope.contains_key(name) {
                scope.insert(name.to_owned(), value);

                break;
            }
        }

        scopes.reverse();
        self.scope_stack = scopes;
    }

    pub fn flatten(&self) -> HashMap<String, VariableValue> {
        let mut entries = HashMap::default();

        for scope in &self.scope_stack {
            let scope = scope.borrow();

            for (key, value) in scope.iter() {
                entries.insert(key.to_owned(), value.to_owned());
            }
        }

        entries
    }
}

impl Interpreter {
    pub fn from_ast(ast: Ast<TypeInfo>, modules: Modules<TypeInfo>) -> Self {
        Self { ast, modules }
    }

    pub fn run(&self) {
        let nodes = self.ast.nodes();

        // TODO: Maybe move this into struct as field
        let mut scope = Scope::default();
        scope.push();

        for node in nodes {
            self.run_statement(&node, &mut scope);
        }
    }

    fn run_statement(&self, statement: &Statement<TypeInfo>, scope: &mut Scope) -> VariableValue {
        match &statement {
            Statement::Expression(expression) => self.run_expression(expression, scope),
            Statement::Intrinsic(intrinsic) => self.run_intrinsic(intrinsic, scope),
            Statement::Import(import) => self.run_import(import, scope),
            Statement::CompilerDirective(compiler_directive) => {
                self.run_compiler_directive(compiler_directive, scope)
            }
        }
    }

    fn run_compiler_directive(
        &self,
        CompilerDirective { statement, .. }: &CompilerDirective<TypeInfo>,
        scope: &mut Scope,
    ) -> VariableValue {
        if let Some(statement) = statement {
            return self.run_statement(statement, scope);
        };

        VariableValue::Void
    }

    fn run_import(&self, import: &Import, scope: &mut Scope) -> VariableValue {
        let Import {
            path,
            position: (file, line, col),
        } = import;

        let Some(module) = self.modules.get(path) else {
            unreachable!("Could not load module '{}' ({}:{}:{})", path, file, line, col);
        };

        let mut import_scope = Scope::default();
        import_scope.push();

        let nodes = module.ast.nodes();

        for node in &nodes {
            self.run_statement(node, &mut import_scope);
        }

        let imports = import_scope.flatten();

        for (key, value) in imports {
            if module.is_wildcard {
                scope.set(&key, value);
            } else {
                scope.set(&format!("{path}::{key}"), value);
            }
        }

        VariableValue::Void
    }

    fn run_intrinsic(&self, intrinsic: &Intrinsic<TypeInfo>, scope: &mut Scope) -> VariableValue {
        match intrinsic {
            Intrinsic::Definition(definition) => self.run_definition(definition, scope),
            Intrinsic::Assignment(assignment) => self.run_assignment(assignment, scope),
            Intrinsic::Declaration(_) => VariableValue::Void,
        }
    }

    fn run_if(&self, if_statement: &If<TypeInfo>, scope: &mut Scope) -> VariableValue {
        let condition = &if_statement.condition;
        let VariableValue::Bool(condition) = self.run_expression(condition, scope) else {
            let position = condition.position();
            unreachable!(
                "Invalid type of condition '{:?}' at {}:{}",
                condition, position.0, position.1
            );
        };

        if condition {
            self.run_block(&if_statement.if_block, scope)
        } else {
            if let Some(else_block) = &if_statement.else_block {
                return self.run_block(else_block, scope);
            }
            VariableValue::Void
        }
    }

    fn run_block(&self, block: &Block<TypeInfo>, scope: &mut Scope) -> VariableValue {
        scope.push();

        let mut return_value = VariableValue::Void;

        for statement in &block.block {
            return_value = self.run_statement(statement, scope);
        }

        scope.pop();

        return_value
    }

    fn run_definition(
        &self,
        definition: &Definition<TypeInfo>,
        scope: &mut Scope,
    ) -> VariableValue {
        let value = self.run_expression(&definition.value, scope);

        scope.set(&definition.ident.value, value);
        VariableValue::Void
    }

    fn run_assignment(
        &self,
        assignment: &Assignment<TypeInfo>,
        scope: &mut Scope,
    ) -> VariableValue {
        let value = self.run_expression(&assignment.value, scope);

        scope.update(&assignment.ident.value, value);
        VariableValue::Void
    }

    fn run_expression(
        &self,
        expression: &Expression<TypeInfo>,
        scope: &mut Scope,
    ) -> VariableValue {
        match expression {
            Expression::If(if_statement) => self.run_if(if_statement, scope),
            Expression::Integer(Integer { value, .. }) => VariableValue::Int(*value),
            Expression::Str(Str { value, .. }) => VariableValue::Str(value.clone()),
            Expression::Boolean(Boolean { value, .. }) => VariableValue::Bool(*value),
            Expression::Ident(Ident { value, .. }) => {
                let Some(value) = scope.find(value) else {
                    unreachable!("Could not find identifier in scope: {}", value)
                };

                value
            }
            Expression::Binary(binary_expr) => self.run_binary_expression(binary_expr, scope),
            Expression::Prefix(prefix_expr) => self.run_prefix_expression(prefix_expr, scope),
            Expression::Postfix(postfix_expr) => self.run_postfix_expression(postfix_expr, scope),
            Expression::Block(block) => self.run_block(block, scope),
            Expression::FnDef(fn_def) => self.run_fn_def(fn_def, scope),
            Expression::Array(_) => todo!(),
        }
    }

    fn run_binary_expression(
        &self,
        binary_expr: &BinaryExpr<TypeInfo>,
        scope: &mut Scope,
    ) -> VariableValue {
        let lhs = &binary_expr.lhs;
        let rhs = &binary_expr.rhs;

        let lhs = self.run_expression(lhs, scope);
        let rhs = self.run_expression(rhs, scope);

        match binary_expr.op {
            BinaryOp::Equal => VariableValue::Bool(lhs == rhs),
            BinaryOp::GreaterThan => {
                let (VariableValue::Int(lhs), VariableValue::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableValue::Bool(lhs > rhs)
            }
            BinaryOp::LessThan => {
                let (VariableValue::Int(lhs), VariableValue::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableValue::Bool(lhs < rhs)
            }
            BinaryOp::Plus => {
                let (VariableValue::Int(lhs), VariableValue::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableValue::Int(lhs + rhs)
            }
            BinaryOp::Minus => {
                let (VariableValue::Int(lhs), VariableValue::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableValue::Int(lhs - rhs)
            }
            BinaryOp::Times => {
                let (VariableValue::Int(lhs), VariableValue::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableValue::Int(lhs * rhs)
            }
            BinaryOp::DividedBy => {
                let (VariableValue::Int(lhs), VariableValue::Int(rhs)) = (lhs, rhs) else {
                    unreachable!();
                };
                VariableValue::Int(lhs / rhs)
            }
        }
    }

    fn run_prefix_expression(
        &self,
        prefix_expression: &PrefixExpr<TypeInfo>,
        _scope: &mut Scope,
    ) -> VariableValue {
        // TODO: Use this rhs
        // let rhs = self.run_expression(&prefix_expression.rhs, scope);

        match prefix_expression.op {
            PrefixOp::UnaryMinus => unimplemented!(),
            PrefixOp::Not => unimplemented!(),
        }
    }

    fn run_postfix_expression(
        &self,
        postfix_expression: &PostfixExpr<TypeInfo>,
        scope: &mut Scope,
    ) -> VariableValue {
        // FIXME: We really ought to evaluate built-in functions such as `print` properly
        // to a `VariableValue::Func` and then call it instead of requiring the
        // expression to be a literal identifier.
        // let lhs = self.run_expression(&postfix_expression.lhs, scope);

        match postfix_expression.op.clone() {
            PostfixOp::Call(call) => {
                let Expression::Ident(ident) = *postfix_expression.lhs.clone() else {
                    unimplemented!("Calling non-identifier expressions is not supported yet!");
                };
                self.run_fn_call(&ident.value, &call, scope)
            }
            PostfixOp::Indexing(_) => todo!(),
        }
    }

    fn run_fn_def(&self, fn_def: &FnDef<TypeInfo>, scope: &mut Scope) -> VariableValue {
        let mut params = vec![];

        for param in &fn_def.params {
            params.push(param.ident.value.clone());
        }

        VariableValue::Func {
            params,
            block: fn_def.block.clone(),
            scope: scope.clone(),
        }
    }

    fn run_fn_call(
        &self,
        fn_name: &str,
        fn_call: &Call<TypeInfo>,
        scope: &mut Scope,
    ) -> VariableValue {
        scope.push();

        let return_value = match fn_name {
            "syscall_4" => {
                let params = &fn_call.params;
                let syscall_type = self.run_expression(&params[0], scope);
                let syscall_param_1 = self.run_expression(&params[1], scope);
                let syscall_param_2 = self.run_expression(&params[2], scope);
                let _syscall_param_3 = self.run_expression(&params[3], scope);

                match (std::env::consts::OS, syscall_type) {
                    ("macos", VariableValue::Int(33554436)) | ("linux", VariableValue::Int(1)) => {
                        match syscall_param_1 {
                            VariableValue::Int(1) => {
                                print!("{syscall_param_2}")
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }

                VariableValue::Void
            }
            "str_len" => {
                let value = self.run_expression(&fn_call.params[0], scope);
                match value {
                    VariableValue::Str(value) => VariableValue::Int(value.len() as i64),
                    _ => unreachable!(),
                }
            }
            "int_to_str" => {
                let value = self.run_expression(&fn_call.params[0], scope);
                match value {
                    VariableValue::Int(value) => VariableValue::Str(format!("{value}")),
                    _ => unreachable!(),
                }
            }
            ident => {
                let Some(fn_def) = scope.find(ident) else {
                    unreachable!();
                };

                let VariableValue::Func { params, block, scope: mut fn_scope } = fn_def else {
                    unreachable!();
                };

                fn_scope.push();

                for (i, param) in fn_call.params.iter().enumerate() {
                    let param_name = &params[i];
                    let param_value = self.run_expression(param, scope);

                    fn_scope.set(param_name, param_value);
                }

                let return_value = self.run_block(&block, &mut fn_scope);

                fn_scope.pop();
                return_value
            }
        };

        scope.pop();

        return_value
    }
}
