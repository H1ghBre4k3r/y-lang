//! Inference context wrapper holding the current lexical `Scope` during
//! type checking. Cloned frequently (cheap) to snapshot scope state when
//! constructing nested AST nodes; equality is intentionally degenerate
//! (always true) because context identity is not semantically relevant
//! to type equality comparisons.
use super::scope::Scope;

#[derive(Debug, Clone)]
pub struct Context {
    pub scope: Scope,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            scope: Scope::new(),
        }
    }
}

impl<'a> From<&'a mut Context> for &'a Context {
    fn from(value: &'a mut Context) -> Self {
        value
    }
}

impl PartialEq for Context {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Context {}
