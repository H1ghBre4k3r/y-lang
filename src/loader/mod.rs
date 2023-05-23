mod loaderror;

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    error::Error,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use log::error;
use pest::iterators::Pair;

use crate::{
    ast::{Ast, Import, Rule, Statement, YParser},
    typechecker::{extract_exports, TypeInfo, TypeScope, Typechecker},
};

use self::loaderror::FileLoadError;

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

    /// The absolute path of this module in the file system.
    pub file_path: PathBuf,
    pub ast: Ast<T>,

    /// A TypeScope containing all exported members of this module.
    pub exports: TypeScope,

    /// A list of imported module. The first item in each tuple is the path under which imported module
    /// is specified in this module, the second item specifies the absolute path of the imported
    /// module in the file system. This is used to convert absolute modules to relative imports.
    pub imports: Vec<(String, String)>,
}

pub type Modules<T> = HashMap<String, Module<T>>;

impl<T> Module<T> {
    /// Resolve a variable name from this module.
    pub fn resolve(&self, var_name: &impl ToString) -> String {
        format!("{}_{}", self.name, var_name.to_string())
    }

    /// Convert the modules currently stored with their absolute path to modules stored with a
    /// relative path (relative to _this_ module). This is needed to determine the correct module
    /// to import while typechecking.
    pub fn convert_imports_to_local_names(&self, modules: &Modules<()>) -> Modules<()> {
        let mut local_modules = Modules::default();

        for (import_path, real_path) in &self.imports {
            local_modules.insert(
                import_path.to_owned(),
                modules.get(real_path).unwrap().to_owned(),
            );
        }
        local_modules
    }
}

impl Module<()> {
    pub fn type_check(
        &self,
        other_modules: &Modules<()>,
    ) -> Result<Module<TypeInfo>, Box<dyn Error>> {
        let modules = self.convert_imports_to_local_names(other_modules);

        let Module {
            name,
            file_path,
            exports,
            imports,
            ast,
        } = self;

        let typechecker = Typechecker::from_ast(ast.clone(), modules);
        let ast = match typechecker.check() {
            Ok(ast) => ast,
            Err(type_error) => {
                error!("{}", type_error);
                std::process::exit(-1);
            }
        };

        Ok(Module {
            ast,
            name: name.clone(),
            exports: exports.clone(),
            imports: imports.clone(),
            file_path: file_path.clone(),
        })
    }
}

pub fn load_module(mut file: PathBuf) -> Result<Module<()>, Box<dyn Error>> {
    let file_content = std::fs::read_to_string(&file)
        .unwrap_or_else(|_| panic!("Could not read file: '{}'", file.to_string_lossy()));

    let pairs = match YParser::parse_program(&file.to_string_lossy(), &file_content) {
        Ok(pairs) => pairs,
        Err(parse_error) => {
            error!("{parse_error}");
            std::process::exit(-1);
        }
    };

    let ast = Ast::from_program(pairs.collect(), &file.to_string_lossy());

    file.pop();

    let folder = file.to_string_lossy();

    let exports = extract_exports(&ast)?;

    let imports = extract_imports(&ast)
        .iter()
        .map(|import_path| {
            (
                import_path.to_owned(),
                convert_to_path(&folder, import_path),
            )
        })
        .collect();

    Ok(Module {
        name: "_".to_owned(),
        ast,
        file_path: file,
        exports,
        imports,
    })
}

pub fn load_modules(
    ast: &Ast<()>,
    mut file: PathBuf,
    mut modules: Modules<()>,
) -> Result<Modules<()>, Box<dyn Error>> {
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

    for import in &imports {
        let file = convert_to_path(&folder, &import.path);

        let mut folder = PathBuf::from(&file);
        folder.pop();
        let folder = folder.to_string_lossy();

        if modules.contains_key(&file) {
            continue;
        }

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

        let imports = extract_imports(&ast)
            .iter()
            .map(|import_path| {
                (
                    import_path.to_owned(),
                    convert_to_path(&folder, import_path),
                )
            })
            .collect();

        let file_path = PathBuf::from(file.clone());

        let mut s = DefaultHasher::new();
        file_content.hash(&mut s);
        let file_hash = s.finish();

        modules.insert(
            file,
            Module {
                name: format!(
                    "{}_{file_hash:x}",
                    file_path.file_stem().unwrap().to_string_lossy()
                ),
                ast: ast.clone(),
                file_path: file_path.clone(),
                exports,
                imports,
            },
        );

        modules = load_modules(&ast, file_path, modules)?;
    }

    Ok(modules)
}

fn convert_to_path(folder: &str, import_path: &str) -> String {
    let is_wildcard = import_path.ends_with("::*");

    let path = &import_path[0..if is_wildcard {
        import_path.len() - 3
    } else {
        import_path.len()
    }]
        .split("::")
        .map(|part| if part == "super" { ".." } else { part })
        .collect::<Vec<_>>()
        .join("/");

    let path = format!("{folder}/{path}.why");

    Path::new(&path)
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

pub fn extract_imports(ast: &Ast<()>) -> Vec<String> {
    ast.nodes()
        .iter()
        .filter_map(|statement| match statement {
            Statement::Import(Import { path, .. }) => Some(path.clone()),
            _ => None,
        })
        .collect()
}
