use std::{collections::HashMap, error::Error, fs::File, io::prelude::*, process::Command};

use log::info;
use Instruction::*;
use LoadSource::*;
use Reg::*;

use crate::{
    asm::{Instruction, LoadSource, Reg},
    ast::{Ast, Declaration, Expression, FnCall, Intrinsic, Statement},
};

struct Variable {
    value: String,
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
}

impl Compiler {
    pub fn from_ast(ast: Ast) -> Self {
        Self {
            ast,
            variables: HashMap::default(),
            constants: HashMap::default(),
            instructions: Self::prelude(),
        }
    }

    fn prelude() -> Vec<Instruction> {
        vec![
            Label("print".to_owned()),
            Mov(RDI, Immediate(1)),
            Mov(RAX, Immediate(0x2000004)),
            Syscall,
            Ret,
            Label("_main".to_owned()),
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

        for instruction in &self.instructions {
            file.write(format!("{}\n", instruction).as_bytes())?;
        }

        Ok(())
    }

    fn write_exit(file: &mut File) -> Result<(), Box<dyn Error>> {
        file.write("exit:\n".as_bytes())?;
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

        Self::write_exit(&mut file)?;
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
            Expression::BinaryOp(_) => todo!(),
            Expression::FnCall(fn_call) => self.compile_fn_call(fn_call),
            Expression::Integer(_) => todo!(),
            Expression::Ident(identifier) => todo!(),
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
                self.constants.insert(
                    name.to_owned(),
                    Constant {
                        name: name.to_owned(),
                        value: string.value.clone(),
                    },
                );
            }
            _ => unimplemented!(),
        }
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
                Expression::Ident(ident) => {
                    let value = &ident.value;
                    self.instructions.append(&mut vec![
                        Lea(RSI, Identifier(value.to_owned())),
                        Mov(
                            RDX,
                            Immediate(self.constants.get(value).unwrap().value.len() as i64),
                        ),
                        Call("print".to_owned()),
                    ])
                }
                Expression::Str(_) => todo!(),
                Expression::FnDef(_) => todo!(),
                Expression::Block(_) => todo!(),
            };
        }
    }
}
