use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    process::{self, Command, Stdio},
};

use codegen::{CodeGen, CodegenContext, ScopeFrame};
use inkwell::{
    OptimizationLevel,
    context::Context,
    module::Module as LLVMModule,
    targets::{FileType, InitializationConfig, Target, TargetMachine},
};
use parser::ast::TopLevelStatement;
use sha2::{Digest, Sha256};
use typechecker::{TypeChecker, TypeInformation, ValidatedTypeInformation};

use crate::{grammar::Program, parser::parse_program};

pub mod codegen;
pub mod formatter;
pub mod grammar;
pub mod lexer;
pub mod lsp;
pub mod parser;
pub mod typechecker;

#[derive(Debug, Clone)]
pub struct Module<T> {
    path: String,
    pub input: String,
    pub inner: T,
}

impl<A> Module<A> {
    fn convert<B>(&self, inner: B) -> Module<B> {
        let Module { path, input, .. } = self;

        Module {
            path: path.clone(),
            input: input.clone(),
            inner,
        }
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.input);
        let result = hasher.finalize();
        format!("{result:x}")
    }

    pub fn file_path(&self) -> String {
        format!("out/{}.ll", self.hash())
    }

    pub fn assembly_file_path(&self) -> String {
        format!("out/{}.s", self.hash())
    }

    pub fn exists(&self) -> bool {
        matches!(std::fs::exists(self.file_path()), Ok(true))
    }

    pub fn compile(&self, out: &str) {
        let out = Command::new("clang")
            .arg(self.file_path())
            .arg("-o")
            .arg(out)
            .stderr(Stdio::inherit())
            .output();

        match out {
            Ok(std::process::Output { status, .. }) => {
                if !status.success() {
                    process::exit(status.code().unwrap_or(-1));
                }
            }
            Err(e) => {
                eprintln!("{e}");
                process::exit(-1);
            }
        }
    }
}

impl Module<()> {
    pub fn new(path: String) -> anyhow::Result<Self> {
        let input = fs::read_to_string(&path)?;
        Ok(Self {
            path,
            input,
            inner: (),
        })
    }

    pub fn lex(&self) -> Result<Module<Program>, Vec<rust_sitter::errors::ParseError>> {
        let program = grammar::parse(&self.input)?;

        Ok(self.convert(program))
    }
}

impl Module<Program> {
    pub fn parse(&self) -> anyhow::Result<Module<Vec<TopLevelStatement<()>>>> {
        let program = self.inner.clone();

        Ok(self.convert(parse_program(program, &self.input)))
    }
}

impl Module<Vec<TopLevelStatement<()>>> {
    pub fn check(&self) -> anyhow::Result<Module<Vec<TopLevelStatement<TypeInformation>>>> {
        let statements = self.inner.clone();
        let typechecker = TypeChecker::new(statements);

        Ok(self.convert(typechecker.check()?))
    }
}

impl Module<Vec<TopLevelStatement<TypeInformation>>> {
    pub fn validate(
        &self,
    ) -> anyhow::Result<Module<Vec<TopLevelStatement<ValidatedTypeInformation>>>> {
        Ok(self.convert(TypeChecker::validate(self.inner.clone())?))
    }
}

impl Module<Vec<TopLevelStatement<ValidatedTypeInformation>>> {
    pub fn codegen(&self) -> anyhow::Result<()> {
        let context = Context::create();
        let module = context.create_module(&self.hash());
        let builder = context.create_builder();

        let codegen_context = CodegenContext {
            context: &context,
            module,
            builder,
            types: RefCell::new(HashMap::default()),
            scopes: RefCell::new(vec![ScopeFrame::default()]),
        };

        // TODO: this _must_ include insertion of functions types, etc.
        // Otherwise, one can not reference functions which are later in the files
        let top_level_statements = &self.inner;
        for statement in top_level_statements {
            statement.codegen(&codegen_context);
        }

        self.emit_assembly_file(&codegen_context.module)?;

        codegen_context
            .module
            .print_to_file(self.file_path())
            .expect("Error while writing to file");

        Ok(())
    }

    fn emit_assembly_file(&self, module: &LLVMModule) -> anyhow::Result<()> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| anyhow::anyhow!("Failed to initialize native target: {}", e))?;

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| anyhow::anyhow!("Failed to create target from triple: {}", e))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                TargetMachine::get_host_cpu_name().to_str().unwrap(),
                TargetMachine::get_host_cpu_features().to_str().unwrap(),
                OptimizationLevel::None,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| anyhow::anyhow!("Failed to create target machine"))?;

        target_machine
            .write_to_file(
                module,
                FileType::Assembly,
                std::path::Path::new(&self.assembly_file_path()),
            )
            .map_err(|e| anyhow::anyhow!("Failed to write assembly file: {}", e))?;

        Ok(())
    }
}
