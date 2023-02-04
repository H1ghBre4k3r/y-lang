use std::collections::HashMap;

use Instruction::*;
use InstructionOperand::*;
use InstructionSize::*;
use Reg::*;

use crate::{
    asm::{Instruction, InstructionOperand, InstructionSize, Reg},
    ast::{BinaryVerb, Declaration, Expression, FnCall, Intrinsic, Statement},
};

#[derive(Debug, Clone, Copy)]
pub struct Variable {
    offset: usize,
}

pub struct Constant {
    pub value: String,
    pub name: String,
}

pub struct Function {
    name: String,
    instructions: Vec<Instruction>,
}

type VariableMap = HashMap<String, Variable>;

type ConstantsMap = HashMap<String, Constant>;

type FunctionMap = HashMap<String, Function>;

pub struct Scope {
    pub statements: Vec<Statement>,
    pub variables: VariableMap,
    pub constants: ConstantsMap,
    pub functions: FunctionMap,
    pub instructions: Vec<Instruction>,
    var_count: usize,
    pub stack_offset: usize,
    level: usize,
}

impl Scope {
    pub fn from_statements(statements: Vec<Statement>, level: usize) -> Self {
        Self {
            statements,
            level,
            variables: HashMap::default(),
            constants: HashMap::default(),
            functions: HashMap::default(),
            instructions: vec![],
            var_count: 0,
            stack_offset: 0,
        }
    }

    fn var(&mut self, name: &str) -> String {
        let var_name = format!("{name}_{}_{}", self.level, self.var_count);
        self.var_count += 1;
        var_name
    }

    pub fn compile(&mut self) {
        let statements = self.statements.clone();
        for node in statements {
            self.compile_statement(&node);
        }
    }

    fn compile_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expression) => self.compile_expression(expression),
            Statement::Intrinsic(intrinsic) => self.compile_intrinsic(intrinsic),
        }
    }

    fn compile_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::If(if_statement) => {
                let condition = &if_statement.condition;

                self.compile_expression(condition);

                let if_block = &if_statement.if_block;

                let if_label = self.var("if");
                let else_label = format!(".{if_label}_else");
                let end_label = format!(".{if_label}_end");

                self.instructions.push(Cmp(Register(Rax), Immediate(0)));
                self.instructions
                    .push(Je(if if_statement.else_block.is_some() {
                        else_label.clone()
                    } else {
                        end_label.clone()
                    }));

                // TODO: Do some stack offset opimizations
                // i.e.: Only increment stack offset by the larger amount and not both
                for statement in &if_block.block {
                    self.compile_statement(statement);
                }

                if let Some(else_block) = &if_statement.else_block {
                    self.instructions.push(Jmp(end_label.clone()));
                    self.instructions.push(Label(else_label));
                    for statement in &else_block.block {
                        self.compile_statement(statement);
                    }
                }

                self.instructions.push(Label(end_label));
            }
            Expression::BinaryOp(binary_operation) => {
                let lhs = &binary_operation.lhs;
                let rhs = &binary_operation.rhs;
                // Compile the seconds expression. (RTL evaluation)
                // This will store the result of this expression in RAX
                self.compile_expression(rhs);
                // Save value on stack
                self.instructions.push(Push(Rax));

                // Evaluate second expression
                self.compile_expression(lhs);

                // Get value from first expression
                self.instructions.push(Pop(Rcx));

                self.instructions.push(Comment(format!(
                    "{:?} {} {:?}",
                    lhs, binary_operation.verb, rhs
                )));

                match &binary_operation.verb {
                    BinaryVerb::Plus => self.instructions.push(Add(Register(Rax), Register(Rcx))),
                    BinaryVerb::Minus => self.instructions.push(Sub(Register(Rax), Register(Rcx))),
                    BinaryVerb::Times => todo!(),
                    BinaryVerb::GreaterThan => {
                        self.instructions.push(Cmp(Register(Rax), Register(Rcx)));
                        self.instructions.push(Setg(Register(Al)));
                        // self.instructions.push(Xor(Register(Rax), Register(Rax)));
                        self.instructions.push(Movzx(Register(Eax), Register(Al)));
                    }
                    BinaryVerb::LessThan => {
                        self.instructions.push(Cmp(Register(Rax), Register(Rcx)));
                        self.instructions.push(Setl(Register(Al)));
                        // self.instructions.push(Xor(Register(Rax), Register(Rax)));
                        self.instructions.push(Movzx(Register(Eax), Register(Al)));
                    }
                    BinaryVerb::Equal => {
                        self.instructions.push(Cmp(Register(Rax), Register(Rcx)));
                        self.instructions.push(Sete(Register(Al)));
                        // self.instructions.push(Xor(Register(Rax), Register(Rax)));
                        self.instructions.push(Movzx(Register(Eax), Register(Al)));
                    }
                };
            }
            Expression::FnCall(fn_call) => self.compile_fn_call(fn_call),
            Expression::Integer(integer) => {
                let value = integer.value;
                self.instructions.push(Comment(format!("LOAD {value}")));
                self.instructions.push(Mov(Register(Rax), Immediate(value)));
            }
            Expression::Boolean(_) => todo!(),
            Expression::Ident(identifier) => {
                let identifier = &identifier.value;
                let variable = self
                    .variables
                    .get(identifier)
                    .expect("Variable not defined");
                let offset = variable.offset;
                self.instructions
                    .push(Comment(format!("LOAD {identifier}")));
                self.instructions
                    .push(Mov(Register(Rax), Memory(Qword, format!("{Rbp}-{offset}"))));
            }
            Expression::Str(_) => todo!(),
            Expression::FnDef(_) => todo!(),
            Expression::Block(_) => todo!(),
        }
    }

    fn compile_intrinsic(&mut self, intrinsic: &Intrinsic) {
        match intrinsic {
            Intrinsic::Declaration(declaration) => self.compile_declaration(declaration),
            Intrinsic::Assignment(_) => todo!(),
        }
    }

    fn compile_declaration(&mut self, declaration: &Declaration) {
        let name = &declaration.ident.value;

        match &declaration.value {
            Expression::Str(string) => {
                self.add_string_constant(Some(name.to_owned()), &string.value.to_owned());
            }
            Expression::Integer(integer) => {
                self.stack_offset += std::mem::size_of::<i64>();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions
                    .push(Comment(format!("{} = {}", name, integer.value)));

                self.instructions.push(Mov(
                    Memory(Qword, format!("{}-{}", Rbp, self.stack_offset)),
                    Immediate(integer.value),
                ));
            }
            Expression::Boolean(boolean) => {
                self.stack_offset += std::mem::size_of::<i64>();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions
                    .push(Comment(format!("{} = {}", name, boolean.value)));

                self.instructions.push(Mov(
                    Memory(Qword, format!("{}-{}", Rbp, self.stack_offset)),
                    Immediate(if boolean.value { 1 } else { 0 }),
                ));
            }
            Expression::If(if_statement) => {
                self.compile_expression(&Expression::If(if_statement.to_owned()));

                self.stack_offset += std::mem::size_of::<i64>();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions.push(Comment(format!(
                    "if {:?} then {:?} else {:?} ",
                    if_statement.condition, if_statement.if_block, if_statement.else_block
                )));

                self.instructions.push(Mov(
                    Memory(Qword, format!("{}-{}", Rbp, self.stack_offset)),
                    Register(Rax),
                ));
            }
            Expression::BinaryOp(binary_operation) => {
                self.compile_expression(&Expression::BinaryOp(binary_operation.to_owned()));

                self.stack_offset += std::mem::size_of::<i64>();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions.push(Comment(format!(
                    "{} = {:?} {} {:?}",
                    name, binary_operation.lhs, binary_operation.verb, binary_operation.rhs
                )));

                self.instructions.push(Mov(
                    Memory(Qword, format!("{}-{}", Rbp, self.stack_offset)),
                    Register(Rax),
                ));
            }
            Expression::FnCall(_) => todo!(),
            Expression::Ident(_) => todo!(),
            Expression::FnDef(fn_definition) => {}
            Expression::Block(_) => todo!(),
        };
    }

    fn compile_fn_call(&mut self, fn_call: &FnCall) {
        let name = fn_call.ident.value.to_owned();

        if name.as_str() == "print" {
            let param = fn_call.params[0].to_owned();
            match param {
                Expression::If(_) => todo!(),
                Expression::BinaryOp(_) => todo!(),
                Expression::FnCall(_) => todo!(),
                Expression::Integer(_) => todo!(),
                Expression::Boolean(_) => todo!(),
                Expression::Ident(ident) => {
                    let value = &ident.value;
                    if let Some(constant) = self.constants.get(value) {
                        self.instructions.append(&mut vec![
                            Lea(Register(Rsi), Identifier(constant.name.to_owned())),
                            Mov(Register(Rdi), Register(Rsi)),
                            Call("str_len".to_owned()),
                            Mov(Register(Rdx), Register(Rax)),
                            Mov(Register(Rdi), Identifier("print".to_owned())),
                            Call("rdi".to_owned()),
                        ]);
                        return;
                    };

                    #[allow(clippy::redundant_pattern_matching)]
                    if let Some(_) = self.variables.get(value) {
                        // TODO: this is another variable (e.g., integer or boolean)
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
                Expression::FnDef(_) => todo!(),
                Expression::Block(_) => todo!(),
            };
        }
    }

    fn add_string_constant(&mut self, name: Option<String>, value: &str) -> String {
        let var_name = name.unwrap_or_else(|| self.var("c"));
        let con = Constant {
            name: var_name.to_owned(),
            value: value.to_owned(),
        };
        self.constants.insert(var_name.to_owned(), con);
        var_name
    }
}
