use std::collections::HashMap;

use Instruction::*;
use InstructionOperand::*;
use InstructionSize::*;
use Reg::*;

use crate::{
    asm::{Instruction, InstructionOperand, InstructionSize, Reg},
    ast::{
        Assignment, BinaryOp, Block, Boolean, Definition, Expression, FnCall, Ident, If, Integer,
        Intrinsic, Statement,
    },
    loader::Module,
    typechecker::TypeInfo,
};

#[derive(Debug, Clone, Copy)]
pub struct Variable {
    offset: usize,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub value: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
struct Parameter {
    name: String,
    info: TypeInfo,
    source: InstructionOperand,
}

type Parameters = Vec<Parameter>;

type VariableMap = HashMap<String, Variable>;

type ConstantsMap = HashMap<String, Constant>;

type FunctionMap = HashMap<String, Function>;

type ExternSymbols = Vec<String>;

#[derive(Clone, Debug, Default)]
pub struct Scope {
    params: Parameters,
    pub statements: Vec<Statement<TypeInfo>>,
    pub variables: VariableMap,
    pub constants: ConstantsMap,
    pub functions: FunctionMap,
    pub instructions: Vec<Instruction>,
    pub externals: ExternSymbols,
    var_count: usize,
    pub stack_offset: usize,
    level: usize,
    level_count: usize,
    new_stack_frame: bool,
    module: Option<Module<TypeInfo>>,
}

impl Scope {
    pub fn from_statements(
        statements: Vec<Statement<TypeInfo>>,
        level: usize,
        new_stack_frame: bool,
        module: Option<Module<TypeInfo>>,
    ) -> Self {
        Self {
            statements,
            level,
            params: vec![],
            variables: HashMap::default(),
            constants: HashMap::default(),
            functions: HashMap::default(),
            instructions: vec![],
            externals: vec![],
            var_count: 0,
            stack_offset: 0,
            level_count: level,
            new_stack_frame,
            module,
        }
    }

    fn var(&mut self, name: &str) -> String {
        let var_name = format!("{name}_{}_{}", self.level, self.var_count);
        self.var_count += 1;
        var_name
    }

    fn level(&mut self) -> usize {
        self.level_count += 1;
        self.level_count
    }

    pub fn add_param(&mut self, name: impl ToString, info: TypeInfo, source: InstructionOperand) {
        self.params.push(Parameter {
            name: name.to_string(),
            info,
            source,
        });
    }

    pub fn compile(&mut self) {
        let statements = self.statements.clone();

        for Parameter { name, info, source } in &self.params {
            self.stack_offset += info.var_size();

            let variable = Variable {
                offset: self.stack_offset,
            };
            self.variables.insert(name.to_owned(), variable);
            self.instructions
                .push(Comment(format!("{name} = {source}")));

            self.instructions.push(Mov(
                Memory(
                    InstructionSize::from(info.clone()),
                    format!("{}-{}", Rbp, self.stack_offset),
                ),
                source.to_owned(),
            ));
        }

        for node in statements {
            self.compile_statement(&node);
        }

        let mut instructions = if self.new_stack_frame {
            vec![
                Comment("Save old stack pointer".to_owned()),
                Push(Rbp),
                Mov(Register(Rbp), Register(Rsp)),
                Comment(
                    "Adjust stack pointer by the amount of space allocated in this stack frame"
                        .to_owned(),
                ),
                Sub(
                    Register(Rsp),
                    Immediate(((self.stack_offset as i64 / 16) + 1) * 16),
                ),
            ]
        } else {
            vec![]
        };

        instructions.append(&mut self.instructions);
        self.instructions = instructions;

        if self.new_stack_frame {
            self.instructions.push(Comment(
                "Adjust stack pointer to fit the previous one".to_owned(),
            ));
            self.instructions.push(Add(
                Register(Rsp),
                Immediate(((self.stack_offset as i64 / 16) + 1) * 16),
            ));
            self.instructions.push(Pop(Rbp));
        }
    }

    fn compile_statement(&mut self, statement: &Statement<TypeInfo>) {
        match statement {
            Statement::Expression(expression) => self.compile_expression(expression),
            Statement::Intrinsic(intrinsic) => self.compile_intrinsic(intrinsic),
            Statement::Import(_) => {}
        }
    }

    fn compile_expression(&mut self, expression: &Expression<TypeInfo>) {
        match expression {
            Expression::If(if_statement) => {
                let condition = &if_statement.condition;

                self.compile_expression(condition);

                let if_block = &if_statement.if_block;

                let if_label = self.var("if");
                let else_label = format!(".{if_label}_else");
                let end_label = format!(".{if_label}_end");

                self.instructions
                    .push(Cmp(Register(Rax.to_sized(&condition.info())), Immediate(0)));
                self.instructions
                    .push(Je(if if_statement.else_block.is_some() {
                        else_label.clone()
                    } else {
                        end_label.clone()
                    }));

                // TODO: Do some stack offset opimizations
                // i.e.: Only increment stack offset by the larger amount and not both
                self.compile_expression(&Expression::Block(if_block.to_owned()));

                if let Some(else_block) = &if_statement.else_block {
                    self.instructions.push(Jmp(end_label.clone()));
                    self.instructions.push(Label(else_label));
                    self.compile_expression(&Expression::Block(else_block.to_owned()));
                }

                self.instructions.push(Label(end_label));
            }
            Expression::Binary(binary_operation) => {
                let lhs = &binary_operation.lhs;
                let rhs = &binary_operation.rhs;
                // Compile the seconds expression. (RTL evaluation)
                // This will store the result of this expression in RAX
                self.compile_expression(rhs);
                // Save value on stack
                self.instructions.push(Push(Rax.to_sized(&rhs.info())));

                // Evaluate second expression
                self.compile_expression(lhs);

                // Get value from first expression
                self.instructions.push(Pop(Rcx.to_sized(&rhs.info())));

                self.instructions.push(Comment(format!(
                    "{:?} {} {:?}",
                    lhs, binary_operation.op, rhs
                )));

                match &binary_operation.op {
                    BinaryOp::Plus => self.instructions.push(Add(
                        Register(Rax.to_sized(&lhs.info())),
                        Register(Rcx.to_sized(&rhs.info())),
                    )),
                    BinaryOp::Minus => self.instructions.push(Sub(
                        Register(Rax.to_sized(&lhs.info())),
                        Register(Rcx.to_sized(&rhs.info())),
                    )),
                    BinaryOp::Times => self.instructions.push(Imul(
                        Register(Rax.to_sized(&lhs.info())),
                        Register(Rcx.to_sized(&rhs.info())),
                    )),
                    BinaryOp::DividedBy => self
                        .instructions
                        .push(Idiv(Register(Rcx.to_sized(&rhs.info())))),
                    BinaryOp::GreaterThan => {
                        self.instructions.push(Cmp(
                            Register(Rax.to_sized(&lhs.info())),
                            Register(Rcx.to_sized(&rhs.info())),
                        ));
                        self.instructions.push(Setg(Register(Al)));
                        self.instructions.push(Movzx(Register(Eax), Register(Al)));
                    }
                    BinaryOp::LessThan => {
                        self.instructions.push(Cmp(
                            Register(Rax.to_sized(&lhs.info())),
                            Register(Rcx.to_sized(&rhs.info())),
                        ));
                        self.instructions.push(Setl(Register(Al)));
                        self.instructions.push(Movzx(Register(Eax), Register(Al)));
                    }
                    BinaryOp::Equal => {
                        self.instructions.push(Cmp(
                            Register(Rax.to_sized(&lhs.info())),
                            Register(Rcx.to_sized(&rhs.info())),
                        ));
                        self.instructions.push(Sete(Register(Al)));
                        self.instructions.push(Movzx(Register(Eax), Register(Al)));
                    }
                };
            }
            Expression::FnCall(fn_call) => self.compile_fn_call(fn_call),
            Expression::Integer(integer) => {
                let value = integer.value;
                self.instructions.push(Comment(format!("LOAD {value}")));
                self.instructions
                    .push(Mov(Register(Rax.to_sized(&integer.info)), Immediate(value)));
            }
            Expression::Boolean(boolean) => {
                self.instructions.push(Comment(format!("LOAD {boolean:?}")));

                self.instructions.push(Mov(
                    Register(Rax.to_sized(&boolean.info)),
                    Immediate(i64::from(boolean.value)),
                ));
            }
            Expression::Ident(Ident {
                value,
                position,
                info,
            }) => {
                let identifier = value;
                self.instructions
                    .push(Comment(format!("LOAD {identifier}")));
                if let Some(variable) = self.variables.get(identifier) {
                    let offset = variable.offset;
                    self.instructions.push(Mov(
                        Register(Rax.to_sized(info)),
                        Memory(
                            InstructionSize::from(info.clone()),
                            format!("{Rbp}-{offset}"),
                        ),
                    ));
                } else if let Some(constant) = self.constants.get(identifier) {
                    self.instructions.push(Lea(
                        Register(Rax.to_sized(info)),
                        Identifier(constant.name.to_owned()),
                    ));
                } else if self.functions.get(identifier).is_some() {
                    self.instructions.push(Lea(
                        Register(Rax.to_sized(info)),
                        Identifier(identifier.to_owned()),
                    ));
                } else {
                    unreachable!(
                        "Could not find variable, constant or function '{identifier}' ({}:{})",
                        position.0, position.1
                    )
                }
            }
            Expression::Str(string) => {
                let value = &string.value;
                let var_name = self.add_string_constant(None, value);
                self.instructions.push(Lea(
                    Register(Rax.to_sized(&string.info)),
                    Identifier(var_name),
                ));
            }
            Expression::FnDef(fn_definition) => {
                // this stuff is basically useful for lambdas
                let statements = &fn_definition.block.block;
                let mut function_scope = Scope::from_statements(
                    statements.clone(),
                    self.level(),
                    true,
                    self.module.clone(),
                );

                for (index, param) in fn_definition.params.iter().enumerate() {
                    let identifier = &param.ident;

                    let info = &identifier.info;
                    let source = match index {
                        0 => InstructionOperand::Register(Rdi.to_sized(info)),
                        1 => InstructionOperand::Register(Rsi.to_sized(info)),
                        _ => todo!(),
                    };

                    function_scope.add_param(&identifier.value, info.clone(), source);
                }

                function_scope.compile();

                let mut instructions = function_scope.instructions.clone();
                instructions.push(Ret);

                for (identifier, constant) in &function_scope.constants {
                    self.constants
                        .insert(identifier.to_owned(), constant.to_owned());
                }

                let fn_name = self.var("fn");

                self.functions
                    .insert(fn_name.to_owned(), Function { instructions });

                self.instructions.push(Comment(format!("fn {fn_name}")));
                self.instructions.push(Lea(
                    Register(Rax.to_sized(&fn_definition.info)),
                    Identifier(fn_name),
                ));
            }
            Expression::Block(Block { block, .. }) => {
                let mut scope =
                    Scope::from_statements(block.clone(), self.level(), false, self.module.clone());

                for (key, value) in &self.variables {
                    scope.variables.insert(key.to_owned(), value.to_owned());
                }

                scope.stack_offset = self.stack_offset;
                scope.compile();

                let mut instructions = scope.instructions.clone();

                self.instructions.append(&mut instructions);

                for (identifier, constant) in &scope.constants {
                    self.constants
                        .insert(identifier.to_owned(), constant.to_owned());
                }

                self.stack_offset = scope.stack_offset;
            }
        }
    }

    fn compile_intrinsic(&mut self, intrinsic: &Intrinsic<TypeInfo>) {
        match intrinsic {
            Intrinsic::Definition(definition) => self.compile_definition(definition),
            Intrinsic::Assignment(assignment) => self.compile_assignment(assignment),
            // TODO: Maybe compile as "extern"
            Intrinsic::Declaration(_) => (),
        }
    }

    fn compile_definition(&mut self, definition: &Definition<TypeInfo>) {
        let name = &definition.ident.value;

        match &definition.value {
            Expression::Str(string) => {
                self.add_string_constant(Some(name.to_owned()), &string.value.to_owned());
            }
            Expression::Integer(Integer { value, info, .. }) => {
                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions.push(Comment(format!("{name} = {value}")));

                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Immediate(*value),
                ));
            }
            Expression::Boolean(Boolean { value, info, .. }) => {
                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions.push(Comment(format!("{name} = {value}")));

                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Immediate(i64::from(*value)),
                ));
            }
            Expression::If(If {
                condition,
                if_block,
                else_block,
                info,
                ..
            }) => {
                self.compile_expression(&definition.value);

                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions.push(Comment(format!(
                    "if {condition:?} then {if_block:?} else {else_block:?} "
                )));

                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Register(Rax.to_sized(info)),
                ));
            }
            Expression::Binary(binary_operation) => {
                self.compile_expression(&Expression::Binary(binary_operation.to_owned()));

                let info = &binary_operation.info;
                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions.push(Comment(format!(
                    "{} = {:?} {} {:?}",
                    name, binary_operation.lhs, binary_operation.op, binary_operation.rhs
                )));

                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Register(Rax.to_sized(info)),
                ));
            }
            Expression::FnCall(FnCall { info, .. }) => {
                self.compile_expression(&definition.value);

                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions
                    .push(Comment(format!("{name} = {:?}", definition.value)));

                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Register(Rax.to_sized(info)),
                ));
            }
            Expression::Ident(Ident { value, info, .. }) => {
                self.compile_expression(&definition.value);
                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions
                    .push(Comment(format!("{name} = {value}",)));
                // TODO: This does not cover booleans or other non-aligned types
                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Register(Rax.to_sized(info)),
                ));
            }
            Expression::FnDef(fn_definition) => {
                let statements = &fn_definition.block.block;
                let mut function_scope = Scope::from_statements(
                    statements.clone(),
                    self.level(),
                    true,
                    self.module.clone(),
                );

                for (key, function) in &self.functions {
                    function_scope
                        .functions
                        .insert(key.to_owned(), function.to_owned());
                }

                for (index, param) in fn_definition.params.iter().enumerate() {
                    let identifier = &param.ident;

                    let info = &identifier.info;
                    let source = match index {
                        0 => InstructionOperand::Register(Rdi.to_sized(info)),
                        1 => InstructionOperand::Register(Rsi.to_sized(info)),
                        _ => todo!(),
                    };

                    function_scope.add_param(&identifier.value, info.clone(), source);
                }

                function_scope.compile();

                let mut instructions = function_scope.instructions.clone();
                instructions.push(Ret);

                for (identifier, constant) in &function_scope.constants {
                    self.constants
                        .insert(identifier.to_owned(), constant.to_owned());
                }

                let mut name = name.clone();

                if let Some(module) = &self.module {
                    name = module.resolve(&name);
                }

                // TODO: This does not allow for function definitions in functions
                self.functions.insert(name, Function { instructions });
            }
            Expression::Block(Block { block, info, .. }) => {
                self.compile_expression(&definition.value);

                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions
                    .push(Comment(format!("{name} = {block:?}")));

                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Register(Rax),
                ));
            }
        };
    }

    fn compile_assignment(&mut self, assignment: &Assignment<TypeInfo>) {
        let value = &assignment.value;
        self.compile_expression(value);

        let identifier = &assignment.ident;
        let info = &identifier.info;

        if let Some(variable) = self.variables.get(&identifier.value) {
            self.instructions
                .push(Comment(format!("{} = {value:?}", identifier.value)));
            self.instructions.push(Mov(
                Memory(
                    InstructionSize::from(info.clone()),
                    format!("{}-{}", Rbp, variable.offset),
                ),
                Register(Rax.to_sized(info)),
            ));
        }
    }

    fn compile_fn_call(&mut self, fn_call: &FnCall<TypeInfo>) {
        let mut name = fn_call.ident.value.to_owned();

        self.instructions
            .push(Comment(format!("CALL {name} ({:?})", fn_call.params)));

        if name.as_str() == "print" {
            let param = fn_call.params[0].to_owned();
            match param {
                Expression::Ident(Ident {
                    value, position, ..
                }) => {
                    if let Some(constant) = self.constants.get(&value) {
                        self.instructions.append(&mut vec![
                            Lea(Register(Rsi), Identifier(constant.name.to_owned())),
                            Mov(Register(Rdi), Register(Rsi)),
                            Call("str_len".to_owned()),
                            Mov(Register(Rdx), Register(Rax)),
                            Call("print".to_owned()),
                        ]);
                        return;
                    } else if let Some(variable) = self.variables.get(&value) {
                        self.instructions.append(&mut vec![
                            Mov(
                                Register(Rsi),
                                Memory(Qword, format!("{}-{}", Rbp, variable.offset)),
                            ),
                            Mov(Register(Rdi), Register(Rsi)),
                            Call("str_len".to_owned()),
                            Mov(Register(Rdx), Register(Rax)),
                            Call("print".to_owned()),
                        ]);
                    } else {
                        unreachable!(
                            "Could not find variable or constant '{value}' ({}:{})",
                            position.0, position.1
                        );
                    }
                }
                Expression::Str(string) => {
                    let value = string.value;
                    let var_name = self.add_string_constant(None, &value);
                    self.instructions.append(&mut vec![
                        Lea(Register(Rsi), Identifier(var_name)),
                        Mov(Register(Rdi), Register(Rsi)),
                        Call("str_len".to_owned()),
                        Mov(Register(Rdx), Register(Rax)),
                        Call("print".to_owned()),
                    ])
                }
                Expression::If(_) => todo!(),
                Expression::Binary(_) => todo!(),
                Expression::FnCall(_) => todo!(),
                Expression::Integer(_) => todo!(),
                Expression::Boolean(_) => todo!(),
                Expression::FnDef(_) => todo!(),
                Expression::Block(_) => todo!(),
            };
            return;
        } else if name.as_str() == "printi" {
            let param = fn_call.params[0].to_owned();
            match param {
                Expression::If(_)
                | Expression::Binary(_)
                | Expression::FnCall(_)
                | Expression::Block(_)
                | Expression::Integer(_)
                | Expression::Ident(_) => {
                    self.compile_expression(&param);
                    self.instructions.append(&mut vec![
                        Mov(Register(Rdi), Register(Rax)),
                        Call("int_to_str".to_owned()),
                        Lea(Register(Rsi), Identifier("int_to_str_val".to_owned())),
                        Mov(Register(Rdi), Register(Rsi)),
                        Call("str_len".to_owned()),
                        Mov(Register(Rdx), Register(Rax)),
                        Call("print".to_owned()),
                    ]);
                }
                _ => unreachable!(),
            };
            return;
        };

        for param in fn_call.params.iter() {
            self.compile_expression(param);
            self.instructions.push(Push(Rax));
        }

        if self.variables.get(&name).is_some() {
            // if we have a variable with this name, we need to load it first
            self.compile_expression(&Expression::Ident(fn_call.ident.to_owned()));
            name = Rax.to_string();
        }

        for (index, _) in fn_call.params.iter().enumerate() {
            match fn_call.params.len() - (index + 1) {
                0 => self.instructions.push(Pop(Rdi)),
                1 => self.instructions.push(Pop(Rsi)),
                _ => todo!(),
            }
        }

        match fn_call.info.source() {
            Some(source) => {
                let fn_name = name.split("::").last().unwrap();
                let fn_name = source.resolve(&fn_name.to_string());
                self.externals.push(fn_name.clone());
                self.instructions.push(Call(fn_name));
            }
            None => {
                let mut fn_name = name;
                if let Some(module) = &self.module {
                    fn_name = module.resolve(&fn_name);
                }
                self.instructions.push(Call(fn_name));
            }
        }
    }

    fn add_string_constant(&mut self, name: Option<String>, value: &str) -> String {
        let var_name = self.var(&name.clone().unwrap_or_else(|| "c".to_owned()));
        let con = Constant {
            name: var_name.to_owned(),
            value: value.to_owned(),
        };
        self.constants
            .insert(name.unwrap_or_else(|| var_name.clone()), con);
        var_name
    }
}
