use crate::loader::Module;

use super::variabletype::VariableType;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    pub(super) _type: VariableType,
    pub source: Option<Module<TypeInfo>>,
}

impl TypeInfo {
    pub fn var_size(&self) -> usize {
        self._type.size()
    }

    pub fn source(&self) -> Option<Module<TypeInfo>> {
        self.source.clone()
    }
}
