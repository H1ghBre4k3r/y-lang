use std::sync::Arc;

use tower_lsp_server::lsp_types::Uri;

use crate::lsp::{SimpleSymbolCollector, SymbolIndex};
use crate::parser::ast::TopLevelStatement;
use crate::typechecker::{TypeChecker, TypeInformation, TypeResult};

/// Extension trait for TypeChecker to support symbol collection
pub trait TypeCheckerSymbolExt {
    /// Type check while collecting symbols for LSP features
    fn check_with_symbols(
        self,
        text: &str,
        uri: Uri,
    ) -> TypeResult<(Vec<TopLevelStatement<TypeInformation>>, Arc<SymbolIndex>)>;
}

impl TypeCheckerSymbolExt for TypeChecker {
    fn check_with_symbols(
        self,
        text: &str,
        uri: Uri,
    ) -> TypeResult<(Vec<TopLevelStatement<TypeInformation>>, Arc<SymbolIndex>)> {
        // Clone the context before consuming self
        let context = self.context.clone();

        // Perform the normal type checking first
        let checked_statements = self.check()?;

        // Create a simple symbol collector
        let collector = SimpleSymbolCollector::new(
            context,
            text,
            uri,
        );

        // Collect symbols from the checked AST
        collector.collect_from_statements(&checked_statements);

        let symbol_index = collector.get_symbol_index();

        Ok((checked_statements, symbol_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar;
    use crate::parser::parse_program;

    #[test]
    fn test_symbol_collection_basic() {
        let input = "
            fn main(): i64 {
                let x = 42;
                return x;
            }
        ";

        let program = grammar::parse(input).expect("Failed to parse");
        let parsed = parse_program(program, input);
        let typechecker = TypeChecker::new(parsed);
        let uri: Uri = "file:///test.why".parse().unwrap();

        let result = typechecker.check_with_symbols(input, uri);
        assert!(result.is_ok());

        let (_, symbol_index) = result.unwrap();

        // Check that we collected the main function and variable x
        let symbols_by_name = symbol_index.find_symbols_by_name("main");
        assert!(!symbols_by_name.is_empty(), "Should find main function");

        let symbols_by_name = symbol_index.find_symbols_by_name("x");
        assert!(!symbols_by_name.is_empty(), "Should find variable x");
    }

    #[test]
    fn test_symbol_collection_struct() {
        let input = "
            struct Point {
                x: i64;
                y: i64;
            }

            fn main(): i64 {
                let p = Point { x: 1, y: 2 };
                return p.x;
            }
        ";

        let program = grammar::parse(input).expect("Failed to parse");
        let parsed = parse_program(program, input);
        let typechecker = TypeChecker::new(parsed);
        let uri: Uri = "file:///test.why".parse().unwrap();

        let result = typechecker.check_with_symbols(input, uri);
        assert!(result.is_ok());

        let (_, symbol_index) = result.unwrap();

        // Check that we collected the struct and its fields
        let symbols_by_name = symbol_index.find_symbols_by_name("Point");
        assert!(!symbols_by_name.is_empty(), "Should find Point struct");

        let symbols_by_name = symbol_index.find_symbols_by_name("x");
        assert!(!symbols_by_name.is_empty(), "Should find field x");

        let symbols_by_name = symbol_index.find_symbols_by_name("y");
        assert!(!symbols_by_name.is_empty(), "Should find field y");
    }
}