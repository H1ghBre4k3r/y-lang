use parser::ast::TopLevelStatement;

pub mod lexer;
pub mod optimizer;
pub mod parser;
pub mod typechecker;

type Ast<T> = Vec<TopLevelStatement<T>>;
