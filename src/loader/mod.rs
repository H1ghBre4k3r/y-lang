mod loaderror;

use std::{collections::HashMap, error::Error, path::PathBuf};

use log::error;
use pest::iterators::Pair;

use crate::{
    ast::{Ast, Rule, Statement, YParser},
    typechecker::{extract_exports, TypeScope},
};

use self::loaderror::FileLoadError;

// TODO: include imports aswell
fn should_be_exported(pair: &Pair<Rule>) -> bool {
    match pair.as_rule() {
        Rule::definition => {
            let mut inner = pair.clone().into_inner();
            let Some(expression) = inner.nth(1) else {
                return false;
            };

            let mut inner = expression.into_inner();
            let Some(fn_def) = inner.next() else {
                return false;
            };
            fn_def.as_rule() == Rule::fnDef
        }
        Rule::declaration => {
            let mut inner = pair.clone().into_inner();

            let Some(type_annotation) = inner.nth(1) else {
                return false;
            };

            let mut inner = type_annotation.into_inner();
            let Some(fn_type) = inner.next() else {
                return false;
            };

            fn_type.as_rule() == Rule::fnType
        }
        Rule::importDirective => true,
        Rule::compiler_directive => true,
        _ => false,
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Module<T> {
    pub name: String,
    pub ast: Ast<T>,
    pub exports: TypeScope,
}

pub type Modules<T> = HashMap<String, Module<T>>;

impl<T> Module<T> {
    pub fn resolve(&self, var_name: &impl ToString) -> String {
        format!("{}_{}", self.name, var_name.to_string())
    }
}

pub fn load_modules(ast: &Ast<()>, mut file: PathBuf) -> Result<Modules<()>, Box<dyn Error>> {
    let nodes = ast.nodes();

    let imports = nodes
        .iter()
        .filter_map(|elem| match elem {
            Statement::Import(import) => Some(import.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    file.pop();

    let folder = file.to_string_lossy();

    let mut modules = HashMap::default();

    for import in &imports {
        let is_wildcard = import.path.ends_with("::*");

        let path = &import.path[0..if is_wildcard {
            import.path.len() - 3
        } else {
            import.path.len()
        }];

        let file = format!("{folder}/{path}.why");

        let Ok(file_content) = std::fs::read_to_string(&file) else {
            return Err(Box::new(FileLoadError {
                message: format!("Could not load module: '{file}'"),
                position: import.position.clone()
            }));
        };

        let pairs = match YParser::parse_program(&file, &file_content) {
            Ok(pairs) => pairs,
            Err(parse_error) => {
                error!("{parse_error}");
                std::process::exit(-1);
            }
        };

        let fns = pairs
            .clone()
            .filter_map(|pair| {
                if should_be_exported(&pair) {
                    Some(pair)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let ast = Ast::from_program(fns.clone(), &file);

        let exports = extract_exports(&ast)?;

        modules.insert(
            import.path.to_owned(),
            Module {
                name: path.to_owned(),
                ast,
                exports,
            },
        );
    }

    Ok(modules)
}
