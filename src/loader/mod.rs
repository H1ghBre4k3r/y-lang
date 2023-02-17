mod loaderror;

use std::{collections::HashMap, error::Error, path::PathBuf};

use pest::iterators::Pair;

use crate::{
    ast::{Ast, Rule, Statement, YParser},
    typechecker::{extract_function_declarations, TypeScope},
};

use self::loaderror::FileLoadError;

fn is_fn_declaration(pair: &Pair<Rule>) -> bool {
    match pair.as_rule() {
        Rule::definition => {
            let mut inner = pair.clone().into_inner();

            let Some(fn_def) = inner.nth(1) else {
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
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct Module<T> {
    pub name: String,
    pub ast: Ast<T>,
    pub exports: TypeScope,
    pub is_wildcard: bool,
}

pub type Modules<T> = HashMap<String, Module<T>>;

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

        let pairs = YParser::parse_program(&file_content);

        let fns = pairs
            .clone()
            .filter_map(|pair| {
                if is_fn_declaration(&pair) {
                    Some(pair)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let ast = Ast::from_program(fns.clone(), &file);

        let function_declarations = extract_function_declarations(&ast)?;

        // let ast = Ast::from_program(pairs.collect(), &file);

        modules.insert(
            import.path.to_owned(),
            Module {
                name: path.to_owned(),
                ast,
                exports: function_declarations,
                is_wildcard,
            },
        );
    }

    Ok(modules)
}