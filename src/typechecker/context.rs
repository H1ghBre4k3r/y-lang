use super::scope::Scope;

#[derive(Debug, Clone)]
pub struct Context {
    scope: Scope,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            scope: Scope::new(),
        }
    }
}
