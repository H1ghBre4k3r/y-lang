use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::parser::ast::Expression;

use super::{error::TypeCheckError, types::Type, TypeInformation, TypedConstruct};

#[derive(Clone)]
struct StoredVariable {
    value: Expression<TypeInformation>,
    type_id: Rc<RefCell<Option<Type>>>,
    mutable: bool,
}

// TODO: this should probably store the location (i.e, span) for all variables, constants and types
// as well
#[derive(Clone, Default)]
/// A frame within a stack, holding information about all variables, types, and constants.
pub struct Frame {
    /// All available variables in this frame
    variables: HashMap<String, StoredVariable>,
    /// All types available within this frame
    types: HashMap<String, Type>,
    /// All constants available in this frame
    constants: HashMap<String, Type>,
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field(
                "variables",
                &self
                    .variables
                    .iter()
                    .map(|(name, StoredVariable { type_id, .. })| {
                        (name, type_id.borrow().as_ref().cloned())
                    })
                    .collect::<HashMap<_, _>>(),
            )
            .field(
                "constants",
                &self.constants.iter().collect::<HashMap<_, _>>(),
            )
            .field("types", &self.types)
            .finish()
    }
}

type StackFrame = Rc<RefCell<Frame>>;

#[derive(Clone, Debug)]
pub struct Scope {
    stacks: Vec<StackFrame>,
    /// all method available for certain type
    methods: Rc<RefCell<HashMap<Type, HashMap<String, Type>>>>,
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            stacks: vec![StackFrame::default()],
            methods: Rc::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableAddError {
    pub name: String,
}

impl Display for VariableAddError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "tried to add already existing type '{}'",
            self.name
        ))
    }
}

impl std::error::Error for VariableAddError {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MethodAddError {
    pub name: String,
}

impl Display for MethodAddError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "tried to add already existing method or property '{}'",
            self.name
        ))
    }
}

impl std::error::Error for MethodAddError {}

impl Scope {
    pub fn new() -> Scope {
        Self::default()
    }

    pub fn enter_scope(&mut self) {
        self.stacks.push(StackFrame::default())
    }

    pub fn exit_scope(&mut self) {
        self.stacks.pop();
    }

    pub fn add_variable(
        &mut self,
        name: impl ToString,
        value: Expression<TypeInformation>,
        mutable: bool,
    ) -> Result<(), VariableAddError> {
        let name = name.to_string();

        if self.get_constant(&name).is_some() {
            return Err(VariableAddError { name });
        }

        self.stacks.last().and_then(|scope| {
            let type_id = value.get_info().type_id;
            scope.borrow_mut().variables.insert(
                name,
                StoredVariable {
                    value,
                    type_id,
                    mutable,
                },
            )
        });

        Ok(())
    }

    fn get_variable(&mut self, name: impl ToString) -> Option<Rc<RefCell<Option<Type>>>> {
        let name = name.to_string();
        self.stacks
            .iter()
            .rev()
            .find(|scope| scope.borrow().variables.contains_key(&name))
            .and_then(|scope| {
                scope
                    .borrow_mut()
                    .variables
                    .get(&name)
                    .cloned()
                    .map(|StoredVariable { type_id, .. }| type_id)
            })
    }

    pub fn is_variable_mutable(&mut self, name: impl ToString) -> Option<bool> {
        let name = name.to_string();
        self.stacks
            .iter()
            .rev()
            .find(|scope| scope.borrow().variables.contains_key(&name))
            .and_then(|scope| {
                scope
                    .borrow_mut()
                    .variables
                    .get(&name)
                    .cloned()
                    .map(|StoredVariable { mutable, .. }| mutable)
            })
    }

    pub fn update_variable(
        &mut self,
        name: impl ToString,
        type_id: Type,
    ) -> Result<(), TypeCheckError> {
        let name = name.to_string();
        let Some(scope) = self
            .stacks
            .iter_mut()
            .rev()
            .find(|scope| scope.borrow().variables.contains_key(&name))
        else {
            todo!()
        };

        let scope = scope.borrow_mut();

        let Some(StoredVariable {
            value: mut exp,
            type_id: variable_type,
            ..
        }) = scope.variables.get(&name).cloned()
        else {
            unreachable!()
        };

        // explicitly drop scope to prevent borrow checker from crashing
        drop(scope);

        // Update the type_id directly instead of calling update_type on the expression
        *variable_type.borrow_mut() = Some(type_id.clone());
        Ok(())
    }

    pub fn add_type(&mut self, name: impl ToString, type_id: Type) -> Result<(), TypeAddError> {
        let name = name.to_string();
        let Some(last) = self.stacks.last_mut() else {
            unreachable!("trying to add type {name} in empty scope");
        };

        if last.borrow().types.contains_key(&name) {
            return Err(TypeAddError { name, type_id });
        }

        last.borrow_mut().types.insert(name, type_id);

        Ok(())
    }

    pub fn get_type(&self, name: impl ToString) -> Option<Type> {
        let name = name.to_string();
        self.stacks
            .iter()
            .rev()
            .find(|scope| scope.borrow().types.contains_key(&name))
            .and_then(|scope| scope.borrow().types.get(&name).cloned())
    }

    fn get_constant(&self, name: impl ToString) -> Option<Type> {
        let name = name.to_string();
        let found_scope = self.stacks
            .iter()
            .rev()
            .find(|scope| scope.borrow().constants.contains_key(&name));
        if let Some(scope) = found_scope {
            let binding = scope.borrow();
            let found_type = binding.constants.get(&name);
            eprintln!("DEBUG: get_constant: found {} in scope with type: {:?}", name, found_type);
            if let Some(Type::Function { params, return_value }) = found_type {
                eprintln!("DEBUG: get_constant: function params: {:?}, return_value: {:?}", params, return_value);
                match return_value.as_ref() {
                    Type::Closure { params: closure_params, return_value: closure_return, captures: closure_captures } => {
                        eprintln!("DEBUG: get_constant: FOUND CLOSURE! params: {:?}, return_value: {:?}, captures: {:?}", closure_params, closure_return, closure_captures);
                    }
                    other => {
                        eprintln!("DEBUG: get_constant: return_value is NOT a closure: {:?}", other);
                    }
                }
            }
            found_type.cloned()
        } else {
            eprintln!("DEBUG: get_constant: {} not found in any scope", name);
            None
        }
    }

    pub fn add_constant(
        &mut self,
        name: impl ToString,
        type_id: Type,
    ) -> Result<(), VariableAddError> {
        let name = name.to_string();

        if self.resolve_name(&name).is_some() {
            return Err(VariableAddError { name });
        }

        let Some(last) = self.stacks.last_mut() else {
            unreachable!("trying to add type {name} in empty scope");
        };

        last.borrow_mut().constants.insert(name, type_id);

        Ok(())
    }

    pub fn update_constant(
        &mut self,
        name: impl ToString,
        type_id: Type,
    ) -> Result<(), VariableAddError> {
        let name = name.to_string();

        // Remove ALL existing entries for this constant to prevent duplicates
        for frame in self.stacks.iter_mut() {
            if frame.borrow_mut().constants.remove(&name).is_some() {
                eprintln!("DEBUG: update_constant: removed existing entry for {}", name);
            }
        }

        // Add the updated type to the current (top) frame
        let Some(last) = self.stacks.last_mut() else {
            unreachable!("trying to update constant {name} in empty scope");
        };
        eprintln!("DEBUG: Scope update_constant: storing {} with type {:?}", name, type_id);
        // Verify the stored type by reading it back
        last.borrow_mut().constants.insert(name.clone(), type_id.clone());
        let binding = last.borrow();
        let stored_type = binding.constants.get(&name).unwrap();
        eprintln!("DEBUG: Scope update_constant: verified stored type for {}: {:?}", name, stored_type);
        Ok(())
    }

    pub fn resolve_name(&mut self, name: impl ToString) -> Option<Rc<RefCell<Option<Type>>>> {
        let name = name.to_string();
        let result = self.get_constant(&name)
            .map(|t| {
                eprintln!("DEBUG: Scope resolve_name: found constant {} with type {:?}", name, t);
                let cell = Rc::new(RefCell::new(Some(t)));
                eprintln!("DEBUG: Scope resolve_name: storing in cell with value: {:?}", cell.borrow());
                cell
            })
            .or_else(|| self.get_variable(&name));
        if result.is_none() {
            eprintln!("DEBUG: Scope resolve_name: {} not found", name);
        }
        result
    }

    /// Add a method (i.e., an associated function) to a type. This function will panic if you try
    /// to add a non-function.
    pub fn add_method_to_type(
        &mut self,
        type_id: Type,
        method_name: impl ToString,
        method_type: Type,
    ) -> Result<(), MethodAddError> {
        assert!(
            matches!(method_type, Type::Function { .. }),
            "tried to add non function as method"
        );
        let method_name = method_name.to_string();

        if let Type::Struct(_, props) = &type_id {
            if props.iter().any(|(name, _)| *name == method_name) {
                return Err(MethodAddError { name: method_name });
            }
        };

        let mut current_methods = {
            self.methods
                .borrow()
                .get(&type_id)
                .cloned()
                .unwrap_or(HashMap::default())
        };

        if current_methods.contains_key(&method_name) {
            return Err(MethodAddError { name: method_name });
        }

        current_methods.insert(method_name, method_type);

        self.methods.borrow_mut().insert(type_id, current_methods);

        Ok(())
    }

    /// Try to resolve a property associated with a given type. For structs, fields are checked
    /// first. After that (and by default for every other type), associated functions are checked.
    pub fn resolve_property_for_type(
        &mut self,
        type_id: Type,
        property: impl ToString,
    ) -> Option<Type> {
        let property_name = property.to_string();

        if let Type::Struct(_, props) = &type_id {
            if let Some(prop) = props
                .iter()
                .find(|(name, _)| *name == property_name)
                .map(|(_, prop)| prop.clone())
            {
                return Some(prop);
            }
        }

        self.methods
            .borrow()
            .get(&type_id)
            .and_then(|methods| methods.get(&property_name))
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        lexer::Span,
        parser::ast::{Expression, Id},
        typechecker::{context::Context, types::Type, TypeInformation},
    };

    use super::Scope;

    #[test]
    fn test_new() {
        let scope = Scope::new();
        assert_eq!(scope.stacks.len(), 1);
    }

    #[test]
    fn test_add_variable() {
        let mut scope = Scope::new();

        let expression = Expression::Id(Id {
            name: "foo".into(),
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
            },
            position: Span::default(),
        });

        scope
            .add_variable("foo", expression, false)
            .expect("something went wrong");

        assert_eq!(
            scope.get_variable("foo"),
            Some(Rc::new(RefCell::new(Some(Type::Integer))))
        );
    }

    #[test]
    fn test_add_override() {
        let mut scope = Scope::new();

        let expression = Expression::Id(Id {
            name: "foo".into(),
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
            },
            position: Span::default(),
        });

        scope
            .add_variable("foo", expression.clone(), false)
            .expect("something went wrong");

        *expression.get_info().type_id.borrow_mut() = Some(Type::Boolean);

        scope
            .add_variable("foo", expression.clone(), false)
            .expect("something went wrong");

        assert_eq!(
            scope.get_variable("foo"),
            Some(Rc::new(RefCell::new(Some(Type::Boolean))))
        );
    }

    #[test]
    fn test_enter_scope() {
        let mut scope = Scope::new();

        let expression = Expression::Id(Id {
            name: "foo".into(),
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
            },
            position: Span::default(),
        });

        scope.enter_scope();
        assert_eq!(scope.stacks.len(), 2);

        scope
            .add_variable("foo", expression.clone(), false)
            .expect("something went wrong");

        assert_eq!(
            scope.get_variable("foo"),
            Some(Rc::new(RefCell::new(Some(Type::Integer))))
        );

        scope.exit_scope();
        assert!(scope.get_variable("foo").is_none())
    }

    #[test]
    fn test_shared_variable_values() {
        let mut scope = Scope::new();

        let expression = Expression::Id(Id {
            name: "foo".into(),
            info: TypeInformation {
                type_id: Rc::new(RefCell::new(Some(Type::Integer))),
                context: Context::default(),
            },
            position: Span::default(),
        });

        scope
            .add_variable("foo", expression.clone(), false)
            .expect("something went wrong");

        let foo = scope.get_variable("foo").unwrap();
        let bar = scope.get_variable("foo").unwrap();

        assert_eq!(foo, bar);

        *foo.borrow_mut() = None;

        assert_eq!(foo, Rc::new(RefCell::new(None)));
        assert_eq!(bar, Rc::new(RefCell::new(None)));
    }
}
