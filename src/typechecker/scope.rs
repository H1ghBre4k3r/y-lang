use std::{collections::HashMap, fmt::Display};

use super::types::Type;

#[derive(Debug, Clone, Default)]
pub struct Stack {
    variables: HashMap<String, Type>,
    types: HashMap<String, Type>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    stacks: Vec<Stack>,
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            stacks: vec![Stack::default()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeAddError {
    pub name: String,
    pub type_id: Type,
}

impl Display for TypeAddError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "tried to add already existing type '{}'",
            self.name
        ))
    }
}

impl std::error::Error for TypeAddError {}

impl Scope {
    pub fn new() -> Scope {
        Self::default()
    }

    pub fn enter_scope(&mut self) {
        self.stacks.push(Stack::default())
    }

    pub fn exit_scope(&mut self) {
        self.stacks.pop();
    }

    pub fn add_variable(&mut self, name: impl ToString, type_id: Type) {
        self.stacks
            .last_mut()
            .and_then(|scope| scope.variables.insert(name.to_string(), type_id));
    }

    pub fn get_variable(&self, name: impl ToString) -> Option<Type> {
        let name = name.to_string();
        self.stacks
            .iter()
            .rev()
            .find(|scope| scope.variables.contains_key(&name))
            .and_then(|scope| scope.variables.get(&name).cloned())
    }

    pub fn add_type(&mut self, name: impl ToString, type_id: Type) -> Result<(), TypeAddError> {
        let name = name.to_string();
        let Some(last) = self.stacks.last_mut() else {
            unreachable!("trying to add type {name} in empty scope");
        };

        if last.types.contains_key(&name) {
            return Err(TypeAddError { name, type_id });
        }

        last.types.insert(name, type_id);

        Ok(())
    }

    pub fn get_type(&self, name: impl ToString) -> Option<Type> {
        let name = name.to_string();
        self.stacks
            .iter()
            .rev()
            .find(|scope| scope.types.contains_key(&name))
            .and_then(|scope| scope.types.get(&name).cloned())
    }
}

#[cfg(test)]
mod tests {
    use crate::typechecker::types::Type;

    use super::Scope;

    #[test]
    fn test_new() {
        let scope = Scope::new();
        assert_eq!(scope.stacks.len(), 1);
    }

    #[test]
    fn test_add_variable() {
        let mut scope = Scope::new();
        scope.add_variable("foo", Type::Integer);

        assert_eq!(scope.get_variable("foo"), Some(Type::Integer));
    }

    #[test]
    fn test_add_override() {
        let mut scope = Scope::new();
        scope.add_variable("foo", Type::Integer);
        scope.add_variable("foo", Type::Boolean);

        assert_eq!(scope.get_variable("foo"), Some(Type::Boolean));
    }

    #[test]
    fn test_enter_scope() {
        let mut scope = Scope::new();

        scope.enter_scope();
        assert_eq!(scope.stacks.len(), 2);

        scope.add_variable("foo", Type::Integer);
        assert_eq!(scope.get_variable("foo"), Some(Type::Integer));

        scope.exit_scope();
        assert!(scope.get_variable("foo").is_none())
    }
}
