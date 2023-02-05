mod scope;
mod ystd;

use std::{error::Error, fs::File, io::prelude::*, process::Command};

use Instruction::*;
use InstructionOperand::*;
use InstructionSize::*;
use Reg::*;

use log::info;

use crate::{
    asm::{Instruction, InstructionOperand, InstructionSize, Reg},
    ast::Ast,
};

use self::{scope::Scope, ystd::int_to_str};
pub struct Compiler {
    scope: Scope,
}

impl Compiler {
    pub fn from_ast(ast: Ast) -> Self {
        Self {
            scope: Scope::from_statements(ast.nodes(), 0),
        }
    }

    fn prelude() -> Vec<Instruction> {
        vec![
            Label("str_len".to_owned()),
            Xor(Register(Rax), Register(Rax)),
            Label(".str_len_loop".to_owned()),
            Cmp(Memory(Byte, format!("{Rdi}+{Rax}")), Immediate(0)),
            Je(".str_len_end".to_owned()),
            Inc(Rax),
            Jmp(".str_len_loop".to_owned()),
            Label(".str_len_end".to_owned()),
            Ret,
            Label("print".to_owned()),
            Mov(Register(Rdi), Immediate(1)),
            Mov(Register(Rax), Immediate(0x2000004)),
            Syscall,
            Ret,
            Literal(int_to_str.to_owned()),
        ]
    }

    fn write_data_section(&mut self, file: &mut File) -> Result<(), Box<dyn Error>> {
        file.write_all("section .data\n".as_bytes())?;
        for k in &self.scope.constants {
            // write the name of the string constant
            file.write_all(format!("\t{} db ", k.0).as_bytes())?;

            // split string into lines
            let string = &k.1.value;
            let mut parts = string.split('\n').peekable();

            while let Some(part) = parts.next() {
                file.write_all(format!("\"{part}\", ").as_bytes())?;
                // if this is not the last line, we append a CRLF
                if parts.peek().is_some() {
                    file.write_all("0xa, 0xd, ".as_bytes())?;
                }
            }
            file.write_all("0\n".as_bytes())?;
        }

        file.write_all("\tint_to_str_val: times 64 db 0\n".as_bytes())?;

        Ok(())
    }

    fn write_text_section(&mut self, file: &mut File) -> Result<(), Box<dyn Error>> {
        file.write_all("\nsection .text\n".as_bytes())?;
        file.write_all("\tglobal _main\n".as_bytes())?;

        let prelude = Self::prelude();
        for instruction in &prelude {
            file.write_all(format!("{instruction}\n").as_bytes())?;
        }

        for (identifier, function) in &self.scope.functions {
            file.write_all(format!("{}", Label(identifier.to_owned())).as_bytes())?;

            for instruction in &function.instructions {
                file.write_all(format!("{instruction}\n").as_bytes())?;
            }
        }

        let mut instructions = vec![Label("_main".to_owned())];
        instructions.append(&mut self.scope.instructions.clone());

        for instruction in &instructions {
            file.write_all(format!("{instruction}\n").as_bytes())?;
        }

        Ok(())
    }

    fn write_exit(&self, file: &mut File) -> Result<(), Box<dyn Error>> {
        file.write_all("\nexit:\n".as_bytes())?;
        file.write_all("\tmov rax, 0x2000001\n".as_bytes())?;
        file.write_all("\tmov rdi, 0\n".as_bytes())?;
        file.write_all("\tsyscall\n".as_bytes())?;

        Ok(())
    }

    fn write_code(&mut self, target: &impl ToString) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(format!("{}.asm", target.to_string()))?;

        file.write_all("default rel\n\n".as_bytes())?;

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
                &target.to_string(),
                &format!("{}.o", target.to_string()),
            ])
            .output()?;
        Ok(())
    }

    pub fn compile(&mut self, target: impl ToString) -> Result<(), Box<dyn Error>> {
        info!("Generating code...");

        self.scope.compile();

        self.write_code(&target)?;
        self.compile_nasm(&target)?;
        self.link_program(&target)?;

        Ok(())
    }
}
