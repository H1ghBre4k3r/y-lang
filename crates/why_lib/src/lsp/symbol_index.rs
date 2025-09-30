use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use tower_lsp_server::lsp_types::{Position, Range, SymbolKind, Uri};

use crate::lexer::Span;
use crate::typechecker::TypeInformation;

/// Unique identifier for a symbol
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SymbolId(pub u64);

/// Represents a symbol definition with its location and metadata
#[derive(Debug, Clone)]
pub struct Definition {
    pub symbol_id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub selection_range: Range, // Range of just the symbol name
    pub type_info: Option<String>,
    pub parent: Option<SymbolId>,
    pub uri: Uri,
}

/// Represents a reference to a symbol
#[derive(Debug, Clone)]
pub struct Reference {
    pub symbol_id: SymbolId,
    pub range: Range,
    pub uri: Uri,
    pub is_definition: bool,
}

/// Main symbol database for storing definitions and references
#[derive(Debug, Default)]
pub struct SymbolIndex {
    /// Maps symbol IDs to their definitions
    definitions: DashMap<SymbolId, Definition>,
    /// Maps symbol IDs to all their references
    references: DashMap<SymbolId, Vec<Reference>>,
    /// Maps symbol names to symbol IDs for quick lookup
    by_name: DashMap<String, Vec<SymbolId>>,
    /// Maps positions to symbol IDs for go-to-definition
    by_position: DashMap<(Uri, u32, u32), SymbolId>,
    /// Next available symbol ID
    next_id: std::sync::atomic::AtomicU64,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a new unique symbol ID
    pub fn next_symbol_id(&self) -> SymbolId {
        SymbolId(self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }

    /// Add a symbol definition to the index
    pub fn add_definition(&self, definition: Definition) {
        let symbol_id = definition.symbol_id;
        let name = definition.name.clone();
        let uri = definition.uri.clone();
        let range = definition.range;

        // Store the definition
        self.definitions.insert(symbol_id, definition);

        // Index by name
        self.by_name.entry(name).or_insert_with(Vec::new).push(symbol_id);

        // Index by the start position only (more efficient than mapping every character)
        self.by_position.insert((uri.clone(), range.start.line, range.start.character), symbol_id);
    }

    /// Add a reference to a symbol
    pub fn add_reference(&self, reference: Reference) {
        let symbol_id = reference.symbol_id;
        self.references.entry(symbol_id).or_insert_with(Vec::new).push(reference);
    }

    /// Get definition by symbol ID
    pub fn get_definition(&self, symbol_id: &SymbolId) -> Option<Definition> {
        self.definitions.get(symbol_id).map(|def| def.clone())
    }

    /// Get all references for a symbol
    pub fn get_references(&self, symbol_id: &SymbolId) -> Vec<Reference> {
        self.references.get(symbol_id).map(|refs| refs.clone()).unwrap_or_default()
    }

    /// Find symbol at a specific position
    pub fn symbol_at_position(&self, uri: &Uri, position: &Position) -> Option<SymbolId> {
        // Check if position falls within any symbol's range
        for entry in self.definitions.iter() {
            let definition = entry.value();
            if definition.uri == *uri && position_in_range(position, &definition.range) {
                return Some(*entry.key());
            }
        }
        None
    }

    /// Find symbols by name (supports partial matching)
    pub fn find_symbols_by_name(&self, name: &str) -> Vec<SymbolId> {
        self.by_name.get(name).map(|ids| ids.clone()).unwrap_or_default()
    }

    /// Get all definitions in a document
    pub fn get_document_symbols(&self, uri: &Uri) -> Vec<Definition> {
        self.definitions
            .iter()
            .filter(|entry| entry.value().uri == *uri)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Clear all symbols for a specific document (for incremental updates)
    pub fn clear_document(&self, uri: &Uri) {
        // Remove definitions for this document
        let mut to_remove = Vec::new();
        for entry in self.definitions.iter() {
            if entry.value().uri == *uri {
                to_remove.push(*entry.key());
            }
        }

        for symbol_id in &to_remove {
            self.definitions.remove(symbol_id);
            self.references.remove(symbol_id);
        }

        // Clean up by_name index
        self.by_name.retain(|_, symbol_ids| {
            symbol_ids.retain(|id| !to_remove.contains(id));
            !symbol_ids.is_empty()
        });

        // Clean up by_position index
        self.by_position.retain(|(doc_uri, _, _), symbol_id| {
            doc_uri != uri || !to_remove.contains(symbol_id)
        });
    }
}

/// Utility functions for converting between span and LSP types
pub mod span_utils {
    use super::*;

    /// Convert a Span to an LSP Range
    /// Note: This assumes the text uses UTF-8 encoding and converts to UTF-16 positions
    pub fn span_to_range(span: &Span) -> Range {
        Range {
            start: Position {
                line: span.start.0 as u32,
                character: span.start.1 as u32,
            },
            end: Position {
                line: span.end.0 as u32,
                character: span.end.1 as u32,
            },
        }
    }

    /// Determine symbol kind from type information and context
    pub fn determine_symbol_kind(type_info: &Option<TypeInformation>, name: &str) -> SymbolKind {
        match type_info {
            Some(info) => {
                if let Some(type_ref) = info.type_id.borrow().as_ref() {
                    match type_ref {
                        crate::typechecker::Type::Function { .. } => SymbolKind::FUNCTION,
                        crate::typechecker::Type::Struct(_, _) => SymbolKind::STRUCT,
                        crate::typechecker::Type::Lambda { .. } => SymbolKind::FUNCTION,
                        _ => {
                            // Use naming conventions as hints
                            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                                SymbolKind::CONSTANT
                            } else {
                                SymbolKind::VARIABLE
                            }
                        }
                    }
                } else {
                    // No type information available, use naming convention
                    if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        SymbolKind::CONSTANT
                    } else {
                        SymbolKind::VARIABLE
                    }
                }
            }
            None => SymbolKind::VARIABLE, // Default fallback
        }
    }
}

/// Check if a position falls within a range
fn position_in_range(position: &Position, range: &Range) -> bool {
    let pos_line = position.line;
    let pos_char = position.character;
    let start_line = range.start.line;
    let start_char = range.start.character;
    let end_line = range.end.line;
    let end_char = range.end.character;

    // Position is before the range
    if pos_line < start_line || (pos_line == start_line && pos_char < start_char) {
        return false;
    }

    // Position is after the range
    if pos_line > end_line || (pos_line == end_line && pos_char >= end_char) {
        return false;
    }

    // Position is within the range
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_index_basic_operations() {
        let index = SymbolIndex::new();
        let uri: Uri = "file:///test.why".parse().unwrap();

        let symbol_id = index.next_symbol_id();
        let definition = Definition {
            symbol_id,
            name: "test_function".to_string(),
            kind: SymbolKind::FUNCTION,
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 13 },
            },
            selection_range: Range {
                start: Position { line: 0, character: 3 },
                end: Position { line: 0, character: 16 },
            },
            type_info: Some("fn() -> i64".to_string()),
            parent: None,
            uri: uri.clone(),
        };

        index.add_definition(definition.clone());

        // Test retrieval
        assert!(index.get_definition(&symbol_id).is_some());
        assert_eq!(index.find_symbols_by_name("test_function"), vec![symbol_id]);

        // Test position lookup
        let pos = Position { line: 0, character: 5 };
        assert_eq!(index.symbol_at_position(&uri, &pos), Some(symbol_id));
    }
}