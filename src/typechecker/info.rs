use crate::loader::Module;

use super::variabletype::VariableType;

/// Struct containing type information about a certain expression.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    /// The concrete type of the expression associated with this type information.
    pub _type: VariableType,
    /// The module, where this expression (or at least the value of this expression) originates
    /// from.
    pub source: Option<Module<()>>,
}

impl TypeInfo {
    /// The size of the type of this expression in memory.
    pub fn var_size(&self) -> usize {
        self._type.size()
    }

    pub fn source(&self) -> Option<Module<()>> {
        self.source.clone()
    }
}
