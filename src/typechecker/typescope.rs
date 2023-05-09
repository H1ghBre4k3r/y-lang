use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{error::TypeError, variabletype::VariableType};

#[derive(Debug, Clone)]
pub struct Variable {
    pub variable_type: VariableType,
    pub is_mutable: bool,
}

#[derive(Default, Debug, Clone)]
struct ScopeFrame {
    pub variables: HashMap<String, Variable>,
    pub type_defs: HashMap<String, VariableType>,
}

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
    pub fn find_variable(&self, name: &str) -> Option<VariableType> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in scopes {
            if let Some(variable) = scope.borrow().variables.get(name) {
                return Some(variable.variable_type.clone());
            }
        }

        None
    }

    pub fn is_variable_mutable(&self, name: &str) -> bool {
        for (index, scope) in self.scope_stack.iter().rev().enumerate() {
            if let Some(Variable { is_mutable, .. }) = scope.borrow().variables.get(name) {
                if *is_mutable || index == 0 {
                    return true;
                }
            }
        }

        false
    }

    /// Check, if a variable with a given name is present.
    pub fn contains_variable(&self, name: &str) -> bool {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in &scopes {
            if scope.borrow().variables.contains_key(name) {
                return true;
            }
        }

        false
    }

    /// Check, if a variable is present in the current scope.
    pub fn contains_in_current_scope(&self, name: &str) -> bool {
        let Some(last) = self.scope_stack.last() else {
            return false;
        };
        return last.borrow().variables.contains_key(name);
    }

    /// Push a new scope frame.
    pub fn push(&mut self) {
        self.scope_stack.push(ScopeFrameReference::default());
    }

    /// Pop the last scope frame.
    pub fn pop(&mut self) {
        self.scope_stack.pop();
    }

    /// Create a new variable on the current scope.
    pub fn set_variable(&mut self, name: &str, value: VariableType, is_mutable: bool) {
        if let Some(scope) = self.scope_stack.last_mut() {
            let variable = Variable {
                variable_type: value,
                is_mutable,
            };
            scope
                .borrow_mut()
                .variables
                .insert(name.to_owned(), variable);
        }
    }

    /// Update a value of an already present variable.
    pub fn update_variable(
        &mut self,
        name: &str,
        value: VariableType,
        position: &(String, usize, usize),
    ) -> Result<(), TypeError> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();

        for scope in &mut scopes {
            let mut scope = scope.borrow_mut();
            if let Some(old_variable) = scope.variables.get(name) {
                let old_type = &old_variable.variable_type;
                if old_type.convert_to(&value).is_err() {
                    return Err(TypeError {
                        message: format!(
                            "Could not assign variable '{name}' with type '{old_type}' a value of type '{value}'"
                        ),
                        position: position.to_owned(),
                    });
                }
                let mut new_variable = old_variable.clone();
                new_variable.variable_type = value;
                scope.variables.insert(name.to_owned(), new_variable);

                break;
            }
        }

        scopes.reverse();
        self.scope_stack = scopes;

        Ok(())
    }

    pub fn contains_type_def(&mut self, name: &str) -> bool {
        let scopes = self.scope_stack.clone();
        for scope in &scopes {
            if scope.borrow().type_defs.contains_key(name) {
                return true;
            }
        }

        false
    }

    pub fn add_type_def(
        &mut self,
        name: &str,
        value: VariableType,
        position: &(String, usize, usize),
    ) -> Result<(), TypeError> {
        if self.contains_type_def(name) {
            return Err(TypeError {
                message: format!("Type '{name}' has already been defined!"),
                position: position.clone(),
            });
        }

        let mut scope = self.scope_stack.last().unwrap().borrow_mut();

        scope.type_defs.insert(name.to_owned(), value);

        Ok(())
    }

    pub fn get_type_def(&self, name: &str) -> Option<VariableType> {
        let scopes = self.scope_stack.clone();
        for scope in &scopes {
            if let Some(variable_type) = scope.borrow().type_defs.get(name) {
                return Some(variable_type.clone());
            }
        }

        None
    }

    pub fn determine_type_literal(&self, name: &str) -> Option<VariableType> {
        if let Some(type_def) = self.get_type_def(name) {
            return Some(type_def);
        }
        name.parse::<VariableType>().ok()
    }

    pub fn flatten(&self) -> HashMap<String, Variable> {
        // TODO: include type defs aswell?
        let mut entries = HashMap::default();

        for scope in &self.scope_stack {
            let scope = scope.borrow();

            for (key, value) in scope.variables.iter() {
                entries.insert(key.to_owned(), value.to_owned());
            }
        }

        entries
    }
}

pub fn setup_scope() -> TypeScope {
    let mut scope = TypeScope::default();

    scope.push();

    scope
}
