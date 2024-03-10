use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    Float,
    Boolean,
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Struct(HashMap<String, Type>),
    Function {
        params: Vec<Type>,
        return_value: Box<Type>,
    },
}
