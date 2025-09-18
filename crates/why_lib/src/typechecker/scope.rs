use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::parser::ast::Expression;

use super::{error::TypeCheckError, types::Type, TypeInformation, TypedConstruct};

#[derive(Clone)]
/// Internal stored variable metadata held in a frame.
/// Contains the original (typed) expression, a shared mutable type slot
/// used during inference / updates and mutability flag.
struct StoredVariable {
    value: Expression<TypeInformation>,
    type_id: Rc<RefCell<Option<Type>>>,
    mutable: bool,
}

// TODO: this should probably store the location (i.e, span) for all variables, constants and types
// as well
#[derive(Clone, Default)]
/// A single lexical frame in the scope stack.
/// Stores separately:
/// - variables: mutable bindings with evolving type slots
/// - constants: immutable bindings storing final types
/// - types: user defined types visible in this frame
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

/// Reference counted mutable pointer to a Frame.
/// Cheaply clonable handle passed around during scope operations.
type StackFrame = Rc<RefCell<Frame>>;

#[derive(Clone, Debug)]
/// Hierarchical lexical scope stack plus associated type → method map.
/// The `stacks` vector forms an inner-most at the end model; lookups walk
/// from end backwards. `methods` stores associated functions per concrete type.
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
    /// Create a new root scope with a single initial frame.
    /// Allocate a fresh root scope containing a single empty frame.
    pub fn new() -> Scope {
        Self::default()
    }

    /// Push a new empty lexical frame onto the stack (enter block/function).
    /// Push a new empty frame representing entry into a nested lexical region.
    pub fn enter_scope(&mut self) {
        self.stacks.push(StackFrame::default())
    }

    /// Pop the most recent lexical frame (leave block/function).
    /// Pop the most recent frame. Panics if called on an empty stack (should not happen).
    pub fn exit_scope(&mut self) {
        self.stacks.pop();
    }

    /// Insert or override a variable binding in the current frame.
    /// Constants take precedence; attempting to shadow a constant yields error.
    /// Add or override a variable binding in the current frame. Will fail if attempting to
    /// shadow an existing constant of the same name.
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

    /// Resolve a variable binding returning its shared type slot (internal).
    /// Internal helper: locate a variable binding walking outward and return its shared type slot.
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

    /// Return mutability flag for a variable if present.
    /// Determine whether a variable is mutable (if it exists); returns None if unresolved.
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

    /// Update (unify) the concrete type of an existing variable binding.
    /// Fails if variable not found (todo placeholder currently).
    /// Update the concrete type associated with a variable (unification result). Propagates the
    /// new concrete type into the underlying expression via `update_type`.
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

        exp.update_type(type_id.clone())?;

        *variable_type.borrow_mut() = Some(type_id);
        Ok(())
    }

    /// Register a user defined type in the current frame.
    /// Register a user defined type in the current innermost frame; errors on duplicate.
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

    /// Lookup a type by name walking outward through frames.
    /// Resolve a type name to its registered definition searching outward frames.
    pub fn get_type(&self, name: impl ToString) -> Option<Type> {
        let name = name.to_string();
        self.stacks
            .iter()
            .rev()
            .find(|scope| scope.borrow().types.contains_key(&name))
            .and_then(|scope| scope.borrow().types.get(&name).cloned())
    }

    /// Internal constant lookup helper.
    /// Internal helper: resolve a constant binding and return its final type.
    fn get_constant(&self, name: impl ToString) -> Option<Type> {
        let name = name.to_string();
        self.stacks
            .iter()
            .rev()
            .find(|scope| scope.borrow().constants.contains_key(&name))
            .and_then(|scope| scope.borrow_mut().constants.get(&name).cloned())
    }

    /// Add an immutable constant binding to the current frame.
    /// Insert a new immutable constant; fails if any value (constant or variable) exists.
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

    /// Resolve any value name (constant preferred, else variable) to a shared type slot.
    /// Resolve either a constant (preferred) or variable to a shared type slot.
    pub fn resolve_name(&mut self, name: impl ToString) -> Option<Rc<RefCell<Option<Type>>>> {
        let name = name.to_string();
        self.get_constant(&name)
            .map(|t| Rc::new(RefCell::new(Some(t))))
            .or_else(|| self.get_variable(&name))
    }

    /// Add a method (i.e., an associated function) to a type. This function will panic if you try
    /// to add a non-function.
    /// Attach an associated function (method) to a type. Panics for non-function type.
    /// Associate a function with a type as a method, ensuring no field / method collision.
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
    /// Resolve either a struct field or previously registered associated method.
    /// Resolve a struct field (priority) or an associated method for a given type.
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
