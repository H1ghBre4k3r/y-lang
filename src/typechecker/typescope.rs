use std::{cell::RefCell, collections::HashMap, rc::Rc};

use tracing::trace;

use super::{error::TypeError, variabletype::VariableType};

#[derive(Debug, Clone)]
pub struct Variable {
    pub variable_type: VariableType,
    pub is_mutable: bool,
}

type ScopeFrame = HashMap<String, Variable>;

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
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn find(&self, name: &str) -> Option<VariableType> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in scopes {
            if let Some(variable) = scope.borrow().get(name) {
                trace!("is contained");
                return Some(variable.variable_type.clone());
            }
        }

        trace!("not contained");
        None
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn is_mutable(&self, name: &str) -> bool {
        for (index, scope) in self.scope_stack.iter().rev().enumerate() {
            if let Some(Variable { is_mutable, .. }) = scope.borrow().get(name) {
                if *is_mutable || index == 0 {
                    trace!("is mutable");
                    return true;
                }
            }
        }

        trace!("is not mutable");
        false
    }

    /// Check, if a variable with a given name is present.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn contains(&self, name: &str) -> bool {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();
        for scope in &scopes {
            if scope.borrow().contains_key(name) {
                trace!("is contained");
                return true;
            }
        }

        trace!("is not contained");
        false
    }

    /// Check, if a variable is present in the current scope.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn contains_in_current_scope(&self, name: &str) -> bool {
        let Some(last) = self.scope_stack.last() else {
            return false;
        };
        return last.borrow().contains_key(name);
    }

    /// Push a new scope frame.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn push(&mut self) {
        trace!("pusing new stack frame");
        self.scope_stack.push(Rc::new(RefCell::new(HashMap::new())))
    }

    /// Pop the last scope frame.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn pop(&mut self) {
        trace!("popping last stack frame");
        self.scope_stack.pop();
    }

    /// Create a new variable on the current scope.
    #[tracing::instrument(level = "trace", skip(self, value))]
    pub fn set(&mut self, name: &str, value: VariableType, is_mutable: bool) {
        if let Some(scope) = self.scope_stack.last_mut() {
            trace!("inserting '{name}' = {value}");
            let variable = Variable {
                variable_type: value,
                is_mutable,
            };
            scope.borrow_mut().insert(name.to_owned(), variable);
        }
    }

    /// Update a value of an already present variable.
    #[tracing::instrument(level = "trace", skip(self, value))]
    pub fn update(
        &mut self,
        name: &str,
        value: VariableType,
        position: &(String, usize, usize),
    ) -> Result<(), TypeError> {
        let mut scopes = self.scope_stack.clone();
        scopes.reverse();

        for scope in &mut scopes {
            let mut scope = scope.borrow_mut();
            if let Some(old_variable) = scope.get(name) {
                let old_type = &old_variable.variable_type;
                if old_type.convert_to(&value).is_err() {
                    return Err(TypeError {
                        message: format!(
                            "Could not assign variable '{name}' with type '{old_type}' a value of type '{value}'"
                        ),
                        position: position.to_owned(),
                    });
                }

                trace!("updating variable '{name}' = {value}");

                let mut new_variable = old_variable.clone();
                new_variable.variable_type = value;
                scope.insert(name.to_owned(), new_variable);
                break;
            }
        }

        scopes.reverse();
        self.scope_stack = scopes;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn flatten(&self) -> HashMap<String, Variable> {
        let mut entries = HashMap::default();

        for scope in &self.scope_stack {
            let scope = scope.borrow();

            for (key, value) in scope.iter() {
                trace!("inserting [{key}] = {value:?}");
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
