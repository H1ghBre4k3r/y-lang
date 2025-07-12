use std::{cell::RefCell, collections::HashMap, fs, process::Command};

use codegen::{CodeGen, CodegenContext};
use inkwell::context::Context;
use parser::ast::{Function, TopLevelStatement};
use sha2::{Digest, Sha256};
use typechecker::{TypeChecker, TypeInformation, ValidatedTypeInformation};

use crate::{grammar::Program, parser::parse_program};

pub mod codegen;
pub mod formatter;
pub mod grammar;
pub mod lexer;
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

    pub fn exists(&self) -> bool {
        matches!(std::fs::exists(self.file_path()), Ok(true))
    }

    pub fn compile(&self, out: &str) {
        if let Err(e) = Command::new("clang")
            .arg(self.file_path())
            .arg("-o")
            .arg(out)
            .output()
        {
            eprintln!("{e}")
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
    fn get_main(&self) -> Function<ValidatedTypeInformation> {
        self.inner
            .iter()
            .find_map(|statement| match statement {
                TopLevelStatement::Function(function) if function.id.name.as_str() == "main" => {
                    Some(function.clone())
                }
                _ => None,
            })
            .expect("At this point, there should be a valid main function")
    }

    pub fn codegen(&self) {
        let context = Context::create();
        let module = context.create_module(&self.hash());
        let builder = context.create_builder();

        let codegen_context = CodegenContext {
            context: &context,
            module,
            builder,
            types: RefCell::new(HashMap::default()),
        };
        let main_function = self.get_main();

        main_function.codegen(&codegen_context);

        // TODO: generate everything else as well

        codegen_context
            .module
            .print_to_file(self.file_path())
            .expect("Error while writing to fil");
    }
}
