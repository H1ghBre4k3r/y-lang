use std::collections::{HashMap, HashSet};

use Instruction::*;
use InstructionOperand::*;
use Reg::*;

use crate::{
    asm::{Instruction, InstructionOperand, InstructionSize, Reg},
    ast::{
        Array, Assignment, BinaryOp, Block, Boolean, Call, Character, CompilerDirective,
        Definition, Expression, Ident, If, InlineAssembly, Integer, Intrinsic, PostfixExpr,
        PostfixOp, Statement, WhileLoop,
    },
    loader::Module,
    typechecker::{TypeInfo, VariableType},
};

#[derive(Debug, Clone)]
pub struct Variable {
    offset: usize,
    _type: VariableType,
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

type ExternSymbols = HashSet<String>;

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
            externals: HashSet::default(),
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
            match info._type.clone() {
                VariableType::Void => {
                    unimplemented!("Parameters of type void are currently not supported")
                }
                // for basic types, we can just copy the value from the register into the stack
                VariableType::Bool
                | VariableType::Str
                | VariableType::Int
                | VariableType::Char
                | VariableType::Any
                | VariableType::Unknown
                | VariableType::Func { .. }
                | VariableType::ArraySlice(_)
                | VariableType::Reference(_) => {
                    self.stack_offset += info.var_size();

                    let variable = Variable {
                        offset: self.stack_offset,
                        _type: info._type.clone(),
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
                // for arrays on the other hand, we need to copy each element from the calling
                // function into our own stack
                VariableType::TupleArray { item_type, size } => {
                    self.stack_offset += item_type.size() * size;
                    let variable = Variable {
                        offset: self.stack_offset,
                        _type: info._type.clone(),
                    };
                    self.variables.insert(name.to_owned(), variable);
                    self.instructions
                        .push(Comment(format!("{name} = {source}")));
                    for i in 0..size {
                        self.instructions.push(Mov(
                            Register(Rax.to_sized(info)),
                            Memory(
                                InstructionSize::from(info.clone()),
                                format!("{}+{}", source, i as i64 * item_type.size() as i64),
                            ),
                        ));
                        self.instructions.push(Mov(
                            Memory(
                                InstructionSize::from(info.clone()),
                                format!(
                                    "{}-{}",
                                    Rbp,
                                    self.stack_offset as i64 - i as i64 * item_type.size() as i64
                                ),
                            ),
                            Register(Rax.to_sized(info)),
                        ));
                    }
                }
            }
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
            Statement::CompilerDirective(compiler_directive) => {
                self.compiler_compiler_directive(compiler_directive)
            }
            Statement::InlineAssembly(inline_assembly) => {
                self.compile_inline_assembly(inline_assembly)
            }
        }
    }

    fn compile_inline_assembly(&mut self, inline_assembly: &InlineAssembly<TypeInfo>) {
        let statements = &inline_assembly.statements;

        for statement in statements {
            self.instructions.push(Raw(statement.to_owned()));
        }
    }

    fn compiler_compiler_directive(
        &mut self,
        CompilerDirective { statement, .. }: &CompilerDirective<TypeInfo>,
    ) {
        if let Some(statement) = statement {
            self.compile_statement(statement);
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
            Expression::Binary(binary_expression) => {
                let lhs = &binary_expression.lhs;
                let rhs = &binary_expression.rhs;
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
                    lhs, binary_expression.op, rhs
                )));

                let info = lhs.info().min(&rhs.info());

                match &binary_expression.op {
                    BinaryOp::Plus => self.instructions.push(Add(
                        Register(Rax.to_sized(&info)),
                        Register(Rcx.to_sized(&info)),
                    )),
                    BinaryOp::Minus => self.instructions.push(Sub(
                        Register(Rax.to_sized(&info)),
                        Register(Rcx.to_sized(&info)),
                    )),
                    BinaryOp::Times => self.instructions.push(Imul(
                        Register(Rax.to_sized(&info)),
                        Register(Rcx.to_sized(&info)),
                    )),
                    BinaryOp::DividedBy => {
                        self.instructions.push(Idiv(Register(Rcx.to_sized(&info))))
                    }
                    BinaryOp::GreaterThan => {
                        self.instructions.push(Cmp(
                            Register(Rax.to_sized(&info)),
                            Register(Rcx.to_sized(&info)),
                        ));
                        self.instructions.push(Setg(Register(Al)));
                        self.instructions.push(Movzx(Register(Eax), Register(Al)));
                    }
                    BinaryOp::LessThan => {
                        self.instructions.push(Cmp(
                            Register(Rax.to_sized(&info)),
                            Register(Rcx.to_sized(&info)),
                        ));
                        self.instructions.push(Setl(Register(Al)));
                        self.instructions.push(Movzx(Register(Eax), Register(Al)));
                    }
                    BinaryOp::Equal => {
                        self.instructions.push(Cmp(
                            Register(Rax.to_sized(&info)),
                            Register(Rcx.to_sized(&info)),
                        ));
                        self.instructions.push(Sete(Register(Al)));
                        self.instructions.push(Movzx(Register(Eax), Register(Al)));
                    }
                };
            }
            Expression::Prefix(_) => {
                unimplemented!("Compiling prefix expressions is not supported yet!")
            }
            Expression::Postfix(PostfixExpr {
                lhs,
                op: PostfixOp::Call(call),
                ..
            }) => match **lhs {
                Expression::Ident(ref ident) => self.compile_fn_call(ident, call),
                _ => unimplemented!(
                    "Compiling calls on non-identifier expressions is not supported yet!"
                ),
            },
            Expression::Postfix(PostfixExpr {
                lhs,
                op: PostfixOp::Indexing(indexing),
                ..
            }) => {
                self.compile_expression(&indexing.index);
                self.instructions.push(Push(Rax));

                self.compile_expression(lhs);

                self.instructions.push(Pop(Rcx));
                self.instructions.push(Mov(
                    Register(Rax.to_sized(&indexing.info)),
                    Memory(
                        InstructionSize::from(indexing.info.clone()),
                        format!("{Rax} + {Rcx} * {}", indexing.info.var_size()),
                    ),
                ))
            }
            Expression::Integer(integer) => {
                let value = integer.value;
                self.instructions.push(Comment(format!("LOAD {value}")));
                self.instructions
                    .push(Mov(Register(Rax.to_sized(&integer.info)), Immediate(value)));
            }
            Expression::Character(Character { value, info, .. }) => {
                self.instructions.push(Comment(format!("LOAD '{value}'")));
                self.instructions
                    .push(Mov(Register(Rax.to_sized(info)), Immediate(*value as i64)));
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
                    match variable._type {
                        VariableType::TupleArray { .. } => {
                            self.instructions.push(Mov(Register(Rax), Register(Rbp)));
                            self.instructions
                                .push(Sub(Register(Rax), Immediate(offset as i64)));
                        }
                        VariableType::Reference(_) => {
                            self.instructions.push(Mov(
                                Register(Rax.to_sized(info)),
                                Memory(
                                    InstructionSize::from(info.clone()),
                                    format!("{Rbp}-{offset}"),
                                ),
                            ));
                            self.instructions.push(Mov(
                                Register(Rax.to_sized(info)),
                                Memory(InstructionSize::from(info.clone()), format!("{Rax}")),
                            ));
                        }
                        _ => {
                            self.instructions.push(Mov(
                                Register(Rax.to_sized(info)),
                                Memory(
                                    InstructionSize::from(info.clone()),
                                    format!("{Rbp}-{offset}"),
                                ),
                            ));
                        }
                    }
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
                        2 => InstructionOperand::Register(Rdx.to_sized(info)),
                        3 => InstructionOperand::Register(Rcx.to_sized(info)),
                        4 => InstructionOperand::Register(R8.to_sized(info)),
                        5 => InstructionOperand::Register(R9.to_sized(info)),
                        _ => unimplemented!(
                            "More than 6 function parameters are currently not supported"
                        ),
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

                function_scope.externals.into_iter().for_each(|external| {
                    self.externals.insert(external);
                });

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

                scope.externals.into_iter().for_each(|external| {
                    self.externals.insert(external);
                });

                self.stack_offset = scope.stack_offset;
            }
            Expression::Array(array) => {
                self.instructions.push(Comment(format!(
                    "LOAD [{:?}; {:?}]",
                    array.initializer, array.size
                )));

                self.store_array_on_stack(array);

                self.instructions.push(Mov(Register(Rax), Register(Rbp)));
                self.instructions
                    .push(Sub(Register(Rax), Immediate(self.stack_offset as i64)));
            }
        }
    }

    fn store_array_on_stack(
        &mut self,
        Array {
            initializer, size, ..
        }: &Array<TypeInfo>,
    ) {
        self.compile_expression(initializer);

        // TODO: Maybe introduce an ASM loop for that
        for i in 0..size.value {
            self.instructions.push(Mov(
                Memory(
                    InstructionSize::from(initializer.info().clone()),
                    format!(
                        "{}-{}",
                        Rbp,
                        self.stack_offset as i64 - i * initializer.info().var_size() as i64
                    ),
                ),
                Register(Rax.to_sized(&initializer.info())),
            ));
        }
    }

    fn compile_intrinsic(&mut self, intrinsic: &Intrinsic<TypeInfo>) {
        match intrinsic {
            Intrinsic::Definition(definition) => self.compile_definition(definition),
            Intrinsic::Assignment(assignment) => self.compile_assignment(assignment),
            Intrinsic::WhileLoop(while_loop) => self.compile_while_loop(while_loop),
            // TODO: Maybe compile as "extern"
            Intrinsic::Declaration(_) => (),
        }
    }

    fn compile_while_loop(&mut self, while_loop: &WhileLoop<TypeInfo>) {
        let condition = &while_loop.condition;
        let block = &while_loop.block;

        let while_label = self.var("while");
        let end_label = format!(".{while_label}_end");

        self.instructions.push(Label(while_label.clone()));

        self.compile_expression(condition);

        self.instructions
            .push(Cmp(Register(Rax.to_sized(&condition.info())), Immediate(0)));
        self.instructions.push(Je(end_label.clone()));

        self.compile_expression(&Expression::Block(block.to_owned()));

        self.instructions.push(Jmp(while_label));
        self.instructions.push(Label(end_label));
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
                    _type: info._type.clone(),
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
            Expression::Character(Character { value, info, .. }) => {
                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                    _type: info._type.clone(),
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions
                    .push(Comment(format!("{name} = '{value}'")));

                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Immediate(*value as i64),
                ));
            }
            Expression::Boolean(Boolean { value, info, .. }) => {
                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                    _type: info._type.clone(),
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
                    _type: info._type.clone(),
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
            Expression::Binary(binary_expression) => {
                self.compile_expression(&Expression::Binary(binary_expression.to_owned()));

                let info = &binary_expression.info;
                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                    _type: info._type.clone(),
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions.push(Comment(format!(
                    "{} = {:?} {} {:?}",
                    name, binary_expression.lhs, binary_expression.op, binary_expression.rhs
                )));

                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Register(Rax.to_sized(info)),
                ));
            }
            Expression::Prefix(_) => {
                unimplemented!("Definitions cannot be generated from prefix expressions yet")
            }
            Expression::Postfix(PostfixExpr {
                op: PostfixOp::Call(call),
                info,
                ..
            }) => {
                self.compile_expression(&definition.value);

                match call.info._type.clone() {
                    VariableType::Void
                    | VariableType::Bool
                    | VariableType::Str
                    | VariableType::Int
                    | VariableType::Char
                    | VariableType::Any
                    | VariableType::Unknown
                    | VariableType::Func { .. }
                    | VariableType::ArraySlice(_)
                    | VariableType::Reference(_) => {
                        self.stack_offset += call.info.var_size();
                        let variable = Variable {
                            offset: self.stack_offset,
                            _type: info._type.clone(),
                        };
                        self.variables.insert(name.to_owned(), variable);

                        self.instructions
                            .push(Comment(format!("{name} = {:?}", definition.value)));

                        self.instructions.push(Mov(
                            Memory(
                                InstructionSize::from(call.info.clone()),
                                format!("{}-{}", Rbp, self.stack_offset),
                            ),
                            Register(Rax.to_sized(&call.info)),
                        ));
                    }
                    VariableType::TupleArray { item_type, size } => {
                        self.stack_offset += item_type.size() * size;
                        let variable = Variable {
                            offset: self.stack_offset,
                            _type: info._type.clone(),
                        };
                        self.variables.insert(name.to_owned(), variable);

                        self.instructions
                            .push(Comment(format!("{name} = {:?}", definition.value)));
                        for i in 0..size {
                            self.instructions.push(Mov(
                                Register(Rcx.to_sized(info)),
                                Memory(
                                    InstructionSize::from(info.clone()),
                                    format!("{}+{}", Rax, i as i64 * item_type.size() as i64),
                                ),
                            ));
                            self.instructions.push(Mov(
                                Memory(
                                    InstructionSize::from(info.clone()),
                                    format!(
                                        "{}-{}",
                                        Rbp,
                                        self.stack_offset as i64
                                            - i as i64 * item_type.size() as i64
                                    ),
                                ),
                                Register(Rcx.to_sized(info)),
                            ));
                        }
                    }
                }
            }
            Expression::Postfix(PostfixExpr {
                op: PostfixOp::Indexing(indexing),
                info,
                ..
            }) => {
                self.compile_expression(&definition.value);

                self.stack_offset += indexing.info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                    _type: info._type.clone(),
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions
                    .push(Comment(format!("{name} = {:?}", definition.value)));

                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(indexing.info.clone()),
                        format!("{}-{}", Rbp, self.stack_offset),
                    ),
                    Register(Rax.to_sized(&indexing.info)),
                ));
            }
            Expression::Ident(Ident { value, info, .. }) => {
                self.compile_expression(&definition.value);
                self.stack_offset += info.var_size();
                let variable = Variable {
                    offset: self.stack_offset,
                    _type: info._type.clone(),
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions.push(Comment(format!("{name} = {value}")));
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
                        2 => InstructionOperand::Register(Rdx.to_sized(info)),
                        3 => InstructionOperand::Register(Rcx.to_sized(info)),
                        4 => InstructionOperand::Register(R8.to_sized(info)),
                        5 => InstructionOperand::Register(R9.to_sized(info)),
                        _ => unimplemented!(
                            "More than 6 function parameters are currently not supported"
                        ),
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

                function_scope.externals.into_iter().for_each(|external| {
                    self.externals.insert(external);
                });

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
                    _type: info._type.clone(),
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
            Expression::Array(array) => {
                let info = &array.info;
                let size = &array.size;

                self.stack_offset += info.var_size() * size.value as usize;
                let variable = Variable {
                    offset: self.stack_offset,
                    _type: info._type.clone(),
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions.push(Comment(format!(
                    "{name} = [{:?}; {size:?}]",
                    array.initializer
                )));

                self.store_array_on_stack(array);
            }
        };
    }

    fn compile_assignment(&mut self, assignment: &Assignment<TypeInfo>) {
        let value = &assignment.value;
        self.compile_expression(value);

        let lhs = &assignment.lhs;

        match lhs {
            Expression::Postfix(PostfixExpr {
                op: PostfixOp::Indexing(indexing),
                lhs,
                ..
            }) => {
                self.instructions
                    .push(Comment(format!("{lhs:?} = {value:?}")));

                // rvalue -> stack
                self.instructions.push(Push(Rax));

                // index -> rax
                self.compile_expression(&indexing.index);

                // index -> stack
                self.instructions.push(Push(Rax));

                // lvalue -> rax
                self.compile_expression(lhs);

                // lvalue -> R8
                self.instructions.push(Mov(Register(R8), Register(Rax)));

                // index -> Rcx
                self.instructions.push(Pop(Rcx));

                // rvalue -> Rax
                self.instructions.push(Pop(Rax));

                // rvalue -> lvalue[index]
                self.instructions.push(Mov(
                    Memory(
                        InstructionSize::from(indexing.info.clone()),
                        format!("{R8} + {Rcx} * {}", indexing.info.var_size()),
                    ),
                    Register(Rax.to_sized(&indexing.info)),
                ));
            }
            Expression::Ident(identifier) => {
                let info = &identifier.info;
                let Some(variable) = self.variables.get(&identifier.value) else {
                    unreachable!();
                };

                match &variable._type {
                    // if we have a reference as an lvalue, we first need to load the address of it
                    VariableType::Reference(var_type) => {
                        let info = TypeInfo {
                            _type: var_type.as_ref().clone(),
                            source: None,
                        };
                        self.instructions
                            .push(Comment(format!("{} = {value:?}", identifier.value)));
                        self.instructions.push(Mov(
                            Register(Rcx),
                            Memory(
                                InstructionSize::from(info.clone()),
                                format!("{}-{}", Rbp, variable.offset),
                            ),
                        ));
                        self.instructions.push(Mov(
                            Memory(InstructionSize::from(info.clone()), format!("{}", Rcx)),
                            Register(Rax.to_sized(&info)),
                        ));
                    }
                    // in every other case, we can just store it on the stack
                    _ => {
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
            }
            _ => {}
        }
    }

    fn compile_fn_call(&mut self, ident: &Ident<TypeInfo>, call: &Call<TypeInfo>) {
        let mut name = ident.value.to_owned();

        self.instructions
            .push(Comment(format!("CALL {name} ({:?})", call.params)));

        if name.as_str() == "str_len" {
            let param = call.params[0].to_owned();
            match param {
                Expression::If(_)
                | Expression::Binary(_)
                | Expression::Prefix(_)
                | Expression::Postfix(_)
                | Expression::Block(_)
                | Expression::Integer(_)
                | Expression::Ident(_) => {
                    self.compile_expression(&param);
                    self.instructions.append(&mut vec![
                        Mov(Register(Rdi), Register(Rax)),
                        Call("str_len".to_owned()),
                    ])
                }
                _ => unreachable!(),
            }

            self.externals.insert("str_len".to_owned());
            return;
        } else if name.as_str() == "int_to_str" {
            let param = call.params[0].to_owned();
            match param {
                Expression::If(_)
                | Expression::Binary(_)
                | Expression::Prefix(_)
                | Expression::Postfix(_)
                | Expression::Block(_)
                | Expression::Integer(_)
                | Expression::Ident(_) => {
                    self.compile_expression(&param);
                    self.instructions.append(&mut vec![
                        Mov(Register(Rdi), Register(Rax)),
                        Call("int_to_str".to_owned()),
                    ])
                }
                _ => unreachable!(),
            }

            self.externals.insert("int_to_str".to_owned());
            return;
        }

        let VariableType::Func { params, .. } = &ident.info._type else {
            unreachable!("Trying to call a non-function expression");
        };

        for (index, param) in call.params.iter().enumerate() {
            // if the type of the parameter is a reference, we need to load the address of it
            if let VariableType::Reference(_) = params[index] {
                let Expression::Ident(Ident { value, info, .. }) = &call.params[index] else {
                    unimplemented!(
                        "Passing non-identifiers as references is currently not supported!"
                    );
                };

                let Some(Variable { offset, .. }) = self.variables.get(value) else {
                    unreachable!()
                };

                if let VariableType::Reference(_) = &info._type {
                    // if our parameter is a reference itself, it needs some extra cuddling
                    self.instructions.push(Mov(
                        Register(Rax),
                        Memory(
                            InstructionSize::from(info.clone()),
                            format!("{Rbp}-{offset}"),
                        ),
                    ));
                } else {
                    self.instructions.push(Mov(Register(Rax), Register(Rbp)));
                    self.instructions
                        .push(Sub(Register(Rax), Immediate(*offset as i64)));
                }
            } else {
                self.compile_expression(param);
            }

            self.instructions.push(Push(Rax));
        }

        if self.variables.get(&name).is_some() {
            // if we have a variable with this name, we need to load it first
            self.compile_expression(&Expression::Ident(ident.to_owned()));
            name = Rax.to_string();
        }

        for (index, _) in call.params.iter().enumerate() {
            match call.params.len() - (index + 1) {
                0 => self.instructions.push(Pop(Rdi)),
                1 => self.instructions.push(Pop(Rsi)),
                2 => self.instructions.push(Pop(Rdx)),
                3 => self.instructions.push(Pop(Rcx)),
                4 => self.instructions.push(Pop(R8)),
                5 => self.instructions.push(Pop(R9)),
                _ => unimplemented!("More than 6 function parameters are currently not supported"),
            }
        }

        match call.info.source() {
            Some(source) => {
                let fn_name = name.split("::").last().unwrap();
                let fn_name = source.resolve(&fn_name.to_string());
                self.externals.insert(fn_name.clone());
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
