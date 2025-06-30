use std::fs;

use parser::ast::TopLevelStatement;
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
        hasher.update(self.path.as_bytes());
        let result = hasher.finalize();
        format!("{result:x}")
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
