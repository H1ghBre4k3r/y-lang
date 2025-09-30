//! Language Server Protocol (LSP) support for Y Lang
//!
//! This module provides the core infrastructure for LSP features including:
//! - Symbol indexing and resolution
//! - Position utilities for UTF-16/UTF-8 conversion
//! - Caching system for analysis results
//!
//! The LSP implementation is designed to integrate with the existing
//! compiler pipeline (grammar → parser → typechecker) while adding
//! the necessary data structures for IDE features.

pub mod position;
//pub mod symbol_collector;
pub mod symbol_collector_simple;
pub mod symbol_index;
pub mod type_checker_ext;

pub use position::PositionUtils;
//pub use symbol_collector::{SymbolCollectable, SymbolCollectingContext};
pub use symbol_collector_simple::SimpleSymbolCollector;
pub use symbol_index::{Definition, Reference, SymbolId, SymbolIndex};
pub use type_checker_ext::TypeCheckerSymbolExt;