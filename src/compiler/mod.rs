mod scope;
mod ystd;

use std::{error::Error, fs::File, io::prelude::*, path::PathBuf, process::Command};

use Instruction::*;
use InstructionOperand::*;
use InstructionSize::*;
use Reg::*;

use log::{error, info};

use crate::{
    asm::{Instruction, InstructionOperand, InstructionSize, Reg, EXIT_SYSCALL},
    ast::Ast,
    loader::{Module, Modules},
    typechecker::TypeInfo,
};

use self::{
    scope::{Constant, Scope},
    ystd::INT_TO_STR,
};
pub struct Compiler {
    scope: Scope,
    modules: Modules<TypeInfo>,
}

impl Compiler {
    pub fn from_ast(ast: Ast<TypeInfo>, modules: Modules<TypeInfo>) -> Self {
        Self {
            scope: Scope::from_statements(ast.nodes(), 0, true, Option::None),
            modules,
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
            Literal(INT_TO_STR.to_owned()),
        ]
    }

    fn write_data_from_standard_library(&mut self, file: &mut File) -> Result<(), Box<dyn Error>> {
        file.write_all("\tint_to_str_val: times 64 db 0\n\n".as_bytes())?;

        Ok(())
    }

    fn write_data_from_scope(
        &mut self,
        file: &mut File,
        scope: &Scope,
    ) -> Result<(), Box<dyn Error>> {
        file.write_all("section .data\n".as_bytes())?;
        for Constant { value, name } in scope.constants.values() {
            // write the name of the string constant
            file.write_all(format!("\t{name} db ").as_bytes())?;

            // split string into lines
            let string = &value;
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

        Ok(())
    }

    fn write_data_section(&mut self, file: &mut File) -> Result<(), Box<dyn Error>> {
        self.write_data_from_scope(file, &self.scope.clone())?;
        self.write_data_from_standard_library(file)?;
        Ok(())
    }

    fn write_global_entry(&self, file: &mut File) -> Result<(), Box<dyn Error>> {
        #[cfg(target_os = "macos")]
        file.write_all("\tglobal _main\n".as_bytes())?;

        #[cfg(target_os = "linux")]
        file.write_all("\tglobal main\n".as_bytes())?;

        file.write_all("\tglobal str_len\n".as_bytes())?;
        file.write_all("\tglobal int_to_str\n".as_bytes())?;

        Ok(())
    }

    fn write_external_symbols(
        &mut self,
        file: &mut File,
        scope: &Scope,
    ) -> Result<(), Box<dyn Error>> {
        for external in &scope.externals {
            file.write_all(format!("extern {external}\n").as_bytes())?;
        }

        Ok(())
    }

    fn write_functions(&mut self, file: &mut File, scope: &Scope) -> Result<(), Box<dyn Error>> {
        file.write_all("\nsection .text\n".as_bytes())?;

        for (identifier, function) in &scope.functions {
            file.write_all(format!("{}", Label(identifier.to_owned())).as_bytes())?;

            for instruction in &function.instructions {
                file.write_all(format!("{instruction}\n").as_bytes())?;
            }
        }

        Ok(())
    }

    fn write_prelude(&mut self, file: &mut File) -> Result<(), Box<dyn Error>> {
        let prelude = Self::prelude();
        for instruction in &prelude {
            file.write_all(format!("{instruction}\n").as_bytes())?;
        }

        Ok(())
    }

    fn write_text_section(&mut self, file: &mut File, scope: &Scope) -> Result<(), Box<dyn Error>> {
        self.write_global_entry(file)?;

        self.write_external_symbols(file, scope)?;

        self.write_functions(file, scope)?;
        self.write_prelude(file)?;

        #[cfg(target_os = "macos")]
        let mut instructions = vec![Label("_main".to_owned())];

        #[cfg(target_os = "linux")]
        let mut instructions = vec![Label("main".to_owned())];

        instructions.append(&mut self.scope.instructions.clone());

        for instruction in &instructions {
            file.write_all(format!("{instruction}\n").as_bytes())?;
        }

        Ok(())
    }

    fn write_exit(&self, file: &mut File) -> Result<(), Box<dyn Error>> {
        file.write_all(format!("{}\n", Label("exit".to_owned())).as_bytes())?;
        file.write_all(format!("{}\n", Mov(Register(Rax), EXIT_SYSCALL)).as_bytes())?;
        file.write_all(format!("{}\n", Mov(Register(Rdi), Immediate(0))).as_bytes())?;
        file.write_all(format!("{Syscall}\n").as_bytes())?;

        Ok(())
    }

    fn write_code(&mut self, target: PathBuf) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(format!("{}.asm", target.to_string_lossy()))?;

        file.write_all("default rel\n\n".as_bytes())?;

        self.write_data_section(&mut file)?;
        self.write_text_section(&mut file, &self.scope.clone())?;

        self.write_exit(&mut file)?;
        Ok(())
    }

    fn compile_nasm(&mut self, target: PathBuf) -> Result<(), Box<dyn Error>> {
        info!("Compiling '{}.asm'...", target.to_string_lossy());

        #[cfg(target_os = "macos")]
        let output = Command::new("nasm")
            .args([
                "-f",
                "macho64",
                &format!("{}.asm", target.to_string_lossy()),
            ])
            .output()?;

        #[cfg(target_os = "linux")]
        let output = Command::new("nasm")
            .args(["-f", "elf64", &format!("{}.asm", target.to_string_lossy())])
            .output()?;

        let stderr = std::str::from_utf8(&output.stderr)?;

        if !stderr.is_empty() {
            error!("{stderr}");
        }

        Ok(())
    }

    fn link_program(&mut self, target: PathBuf, files: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
        info!("Linking program...");

        let mut args = Vec::<String>::new();

        #[cfg(target_os = "macos")]
        {
            args.extend(["-arch", "x86_64"].map(|s| s.to_string()));
        }

        args.push("-o".to_string());

        let target = target.to_string_lossy();
        args.push(target.to_string());

        let target = format!("{target}.o");
        args.push(target);

        let mut files = files
            .iter()
            .map(|file| format!("{}.o", file.to_string_lossy().as_ref()))
            .collect::<Vec<_>>();

        args.append(&mut files);

        let output = Command::new("cc").args(args.as_slice()).output()?;

        let stderr = std::str::from_utf8(&output.stderr)?;

        if !stderr.is_empty() {
            error!("{stderr}");
        }

        Ok(())
    }

    fn compile_module(
        &mut self,
        module: &Module<TypeInfo>,
        folder: PathBuf,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let mut scope = Scope::from_statements(module.ast.nodes(), 0, true, Some(module.clone()));
        scope.compile();

        let mut output = folder;
        output.push(module.name.clone());

        let mut file = File::create(format!("{}.asm", output.to_string_lossy()))?;

        file.write_all("default rel\n\n".as_bytes())?;

        for export in module.exports.flatten().keys() {
            file.write_all(format!("global {}\n", module.resolve(export)).as_bytes())?;
        }

        self.write_external_symbols(&mut file, &scope)?;

        self.write_data_from_scope(&mut file, &scope)?;
        self.write_functions(&mut file, &scope)?;

        self.compile_nasm(output.clone())?;

        Ok(output)
    }

    pub fn compile_program(&mut self, target: PathBuf) -> Result<(), Box<dyn Error>> {
        info!("Generating code...");

        self.scope.compile();

        let mut folder = target.clone();
        folder.pop();

        let modules = self.modules.clone();

        let mut others = vec![];

        for module in modules.values() {
            others.push(self.compile_module(module, folder.clone())?);
        }

        self.write_code(target.clone())?;
        self.compile_nasm(target.clone())?;
        self.link_program(target, others)?;

        Ok(())
    }
}
