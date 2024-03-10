use std::collections::HashMap;

use super::types::Type;

#[derive(Debug, Clone)]
pub struct Scope {
    stacks: Vec<HashMap<String, Type>>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            stacks: vec![HashMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.stacks.push(HashMap::new())
    }

    pub fn exit_scope(&mut self) {
        self.stacks.pop();
    }

    pub fn add_variable(&mut self, name: impl ToString, type_id: Type) {
        self.stacks
            .last_mut()
            .and_then(|scope| scope.insert(name.to_string(), type_id));
    }

    pub fn get_variable(&mut self, name: impl ToString) -> Option<Type> {
        let name = name.to_string();
        self.stacks
            .iter()
            .rev()
            .find(|scope| scope.contains_key(&name))
            .and_then(|scope| scope.get(&name).cloned())
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
