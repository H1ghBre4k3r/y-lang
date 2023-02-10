use super::variabletype::VariableType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    pub(super) _type: VariableType,
}

impl TypeInfo {
    pub fn var_size(&self) -> usize {
        self._type.size()
    }
}
