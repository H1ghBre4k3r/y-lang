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
