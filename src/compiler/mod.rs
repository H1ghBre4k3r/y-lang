use std::{collections::HashMap, error::Error, fs::File, io::prelude::*, process::Command};

use log::info;
use Instruction::*;
use InstructionOperand::*;
use InstructionSize::*;
use Reg::*;

use crate::{
    asm::{Instruction, InstructionOperand, InstructionSize, Reg},
    ast::{Ast, BinaryVerb, Declaration, Expression, FnCall, Intrinsic, Statement},
};

#[derive(Debug, Clone, Copy)]
struct Variable {
    offset: usize,
}

struct Constant {
    value: String,
    name: String,
}

type VariableMap = HashMap<String, Variable>;

type ConstantsMap = HashMap<String, Constant>;

pub struct Compiler {
    ast: Ast,
    variables: VariableMap,
    constants: ConstantsMap,
    instructions: Vec<Instruction>,
    var_count: usize,
    stack_offset: usize,
}

impl Compiler {
    pub fn from_ast(ast: Ast) -> Self {
        Self {
            ast,
            variables: HashMap::default(),
            constants: HashMap::default(),
            instructions: vec![],
            var_count: 0,
            stack_offset: 0,
        }
    }

    fn var(&mut self, name: &str) -> String {
        let var_name = format!("{}_{}", name, self.var_count);
        self.var_count += 1;
        var_name
    }

    fn prelude() -> Vec<Instruction> {
        vec![
            Label("str_len".to_owned()),
            Xor(Register(RAX), Register(RAX)),
            Label(".str_len_loop".to_owned()),
            Cmp(Memory(Byte, format!("{}+{}", RDI, RAX)), Immediate(0)),
            Je(".str_len_end".to_owned()),
            Inc(RAX),
            Jmp(".str_len_loop".to_owned()),
            Label(".str_len_end".to_owned()),
            Ret,
            Label("print".to_owned()),
            Mov(Register(RDI), Immediate(1)),
            Mov(Register(RAX), Immediate(0x2000004)),
            Syscall,
            Ret,
            // TODO: add somethign like "sub rsp STACK_OFFSET"
        ]
    }

    fn write_data_section(&mut self, file: &mut File) -> Result<(), Box<dyn Error>> {
        file.write(format!("section .data\n").as_bytes())?;

        for k in &self.constants {
            file.write(format!("\t{} db \"{}\", 0\n", k.0, k.1.value).as_bytes())?;
        }

        Ok(())
    }

    fn write_text_section(&mut self, file: &mut File) -> Result<(), Box<dyn Error>> {
        file.write(format!("\nsection .text\n").as_bytes())?;
        file.write(format!("\tglobal _main\n\n").as_bytes())?;

        let prelude = Self::prelude();
        for instruction in &prelude {
            file.write(format!("{}\n", instruction).as_bytes())?;
        }

        let mut instructions = vec![
            Label("_main".to_owned()),
            Comment("Save old stack pointer".to_owned()),
            Push(RBP),
            Mov(Register(RBP), Register(RSP)),
            Comment(
                "Adjust stack pointer by the amount of space allocated in this stack frame"
                    .to_owned(),
            ),
            Sub(
                Register(RSP),
                Immediate(((self.stack_offset as i64 / 16) + 1) * 16),
            ),
        ];
        instructions.append(&mut self.instructions.clone());

        for instruction in &instructions {
            file.write(format!("{}\n", instruction).as_bytes())?;
        }

        Ok(())
    }

    fn write_exit(&self, file: &mut File) -> Result<(), Box<dyn Error>> {
        file.write(
            format!(
                "{}\n",
                Add(
                    Register(RSP),
                    Immediate(((self.stack_offset as i64 / 16) + 1) * 16),
                )
            )
            .as_bytes(),
        )?;
        file.write(format!("{}\n", Pop(RBP)).as_bytes())?;
        file.write("\nexit:\n".as_bytes())?;
        file.write("\tmov rax, 0x2000001\n".as_bytes())?;
        file.write("\tmov rdi, 0\n".as_bytes())?;
        file.write("\tsyscall\n".as_bytes())?;

        Ok(())
    }

    fn write_code(&mut self, target: &impl ToString) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(format!("{}.asm", target.to_string()))?;

        file.write(format!("default rel\n\n").as_bytes())?;

        self.write_data_section(&mut file)?;
        self.write_text_section(&mut file)?;

        self.write_exit(&mut file)?;
        Ok(())
    }

    fn compile_nasm(&mut self, target: &impl ToString) -> Result<(), Box<dyn Error>> {
        info!("Compiling '{}.asm'...", target.to_string());

        #[cfg(target_os = "macos")]
        Command::new("nasm")
            .args(["-f", "macho64", &format!("{}.asm", target.to_string())])
            .output()?;

        Ok(())
    }

    fn link_program(&mut self, target: &impl ToString) -> Result<(), Box<dyn Error>> {
        info!("Linking program...");

        #[cfg(target_os = "macos")]
        Command::new("ld")
            .args([
                "-macos_version_min",
                "10.12.0",
                "-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib",
                "-lSystem",
                "-o",
                &format!("{}", target.to_string()),
                &format!("{}.o", target.to_string()),
            ])
            .output()?;
        Ok(())
    }

    pub fn compile(&mut self, target: impl ToString) -> Result<(), Box<dyn Error>> {
        info!("Generating code...");
        let nodes = self.ast.nodes();

        for node in &nodes {
            self.compile_statement(&node);
        }

        self.write_code(&target)?;
        self.compile_nasm(&target)?;
        self.link_program(&target)?;

        Ok(())
    }

    fn compile_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expression) => self.compile_expression(expression),
            Statement::Intrinsic(intrinsic) => self.compile_intrinsic(intrinsic),
        }
    }

    fn compile_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::If(_) => todo!(),
            Expression::BinaryOp(binary_operation) => {
                let lhs = &binary_operation.lhs;
                let rhs = &binary_operation.rhs;
                // Compile the seconds expression. (RTL evaluation)
                // This will store the result of this expression in RAX
                self.compile_expression(&rhs);
                // Move value from RAX to RCX
                self.instructions.push(Mov(Register(RCX), Register(RAX)));

                self.compile_expression(&lhs);

                self.instructions.push(Comment(format!(
                    "{:?} {} {:?}",
                    lhs, binary_operation.verb, rhs
                )));

                match &binary_operation.verb {
                    BinaryVerb::Plus => self.instructions.push(Add(Register(RAX), Register(RCX))),
                    BinaryVerb::Minus => self.instructions.push(Sub(Register(RAX), Register(RCX))),
                    BinaryVerb::Times => todo!(),
                    BinaryVerb::GreaterThan => todo!(),
                    BinaryVerb::LessThan => todo!(),
                    BinaryVerb::Equal => todo!(),
                };
            }
            Expression::FnCall(fn_call) => self.compile_fn_call(fn_call),
            Expression::Integer(integer) => {
                let value = integer.value;
                self.instructions.push(Comment(format!("LOAD {}", value)));
                self.instructions.push(Mov(Register(RAX), Immediate(value)));
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
                    .push(Comment(format!("LOAD {}", identifier)));
                self.instructions.push(Mov(
                    Register(RAX),
                    Memory(Qword, format!("{}-{}", RBP, offset)),
                ));
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
                    Memory(Qword, format!("{}-{}", RBP, self.stack_offset)),
                    Immediate(integer.value),
                ));
            }
            Expression::Boolean(boolean) => {
                self.stack_offset += std::mem::size_of::<bool>();
                let variable = Variable {
                    offset: self.stack_offset,
                };
                self.variables.insert(name.to_owned(), variable);

                self.instructions
                    .push(Comment(format!("{} = {}", name, boolean.value)));

                self.instructions.push(Mov(
                    Memory(Qword, format!("{}-{}", RBP, self.stack_offset)),
                    Immediate(if boolean.value { 1 } else { 0 }),
                ));
            }
            Expression::If(_) => todo!(),
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
                    Memory(Qword, format!("{}-{}", RBP, self.stack_offset)),
                    Register(RAX),
                ));
            }
            Expression::FnCall(_) => todo!(),
            Expression::Ident(_) => todo!(),
            Expression::FnDef(_) => todo!(),
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
                            Lea(Register(RSI), Identifier(constant.name.to_owned())),
                            Mov(Register(RDI), Register(RSI)),
                            Call("str_len".to_owned()),
                            Mov(Register(RDX), Register(RAX)),
                            Call("print".to_owned()),
                        ]);
                        return;
                    };

                    if let Some(variable) = self.variables.get(value) {
                        // TODO: this is another variable (e.g., integer or boolean)
                    }
                }
                Expression::Str(string) => {
                    let value = string.value;
                    let var_name = self.add_string_constant(None, &value.to_owned());
                    self.instructions.append(&mut vec![
                        Lea(Register(RSI), Identifier(var_name)),
                        Mov(Register(RDI), Register(RSI)),
                        Call("str_len".to_owned()),
                        Mov(Register(RDX), Register(RAX)),
                        Call("print".to_owned()),
                    ])
                }
                Expression::FnDef(_) => todo!(),
                Expression::Block(_) => todo!(),
            };
        }
    }

    fn add_string_constant(&mut self, name: Option<String>, value: &str) -> String {
        let var_name = name.unwrap_or_else(|| self.var("str_const"));
        let con = Constant {
            name: var_name.to_owned(),
            value: value.to_owned(),
        };
        self.constants.insert(var_name.to_owned(), con);
        var_name
    }
}
