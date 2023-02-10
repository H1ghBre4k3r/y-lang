use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{error::TypeError, variabletype::VariableType};

type ScopeFrame = HashMap<String, VariableType>;

type ScopeFrameReference = Rc<RefCell<ScopeFrame>>;

#[derive(Default, Debug, Clone)]
pub struct TypeScope {
    scope_stack: Vec<ScopeFrameReference>,
}

impl PartialEq for TypeScope {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for TypeScope {}

impl TypeScope {
    /// Find a value/reference in this scope by iterating over the scopes from back to front.
    pub fn find(&self, name: &str) -> Option<VariableType> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in scopes {
            if let Some(variable) = scope.borrow().get(name) {
                return Some(variable.clone());
            }
        }

        None
    }

    pub fn is_in_current_scope(&self, name: &str) -> bool {
        let scopes = self.scope_stack.clone();
        if let Some(last) = scopes.last() {
            return last.borrow().contains_key(name);
        }
        false
    }

    /// Check, if a variable with a given name is present.
    pub fn contains(&self, name: &str) -> bool {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in &scopes {
            if scope.borrow().contains_key(name) {
                return true;
            }
        }

        false
    }

    /// Push a new scope frame.
    pub fn push(&mut self) {
        self.scope_stack.push(Rc::new(RefCell::new(HashMap::new())))
    }

    /// Pop the last scope frame.
    pub fn pop(&mut self) {
        self.scope_stack.pop();
    }

    /// Create a new variable on the current scope.
    pub fn set(&mut self, name: &str, value: VariableType) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.borrow_mut().insert(name.to_owned(), value);
        }
    }

    /// Update a value of an already present variable.
    pub fn update(
        &mut self,
        name: &str,
        value: VariableType,
        position: &(usize, usize),
    ) -> Result<(), TypeError> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();

        for scope in &mut scopes {
            let mut scope = scope.borrow_mut();
            if let Some(old_type) = scope.get(name) {
                if *old_type != value {
                    return Err(TypeError {
                        message: format!(
                            "Could not assign variable '{name}' with type '{old_type}' a value of type '{value}'"
                        ),
                        position: position.to_owned(),
                    });
                }
                scope.insert(name.to_owned(), value);

                break;
            }
        }

        scopes.reverse();
        self.scope_stack = scopes;

        Ok(())
    }
}

pub fn setup_scope() -> TypeScope {
    let mut scope = TypeScope::default();

    scope.push();

    scope.set(
        "print",
        VariableType::Func {
            params: vec![VariableType::Any],
            return_value: Box::new(VariableType::Void),
        },
    );

    scope.set(
        "printi",
        VariableType::Func {
            params: vec![VariableType::Int],
            return_value: Box::new(VariableType::Void),
        },
    );

    scope
}
