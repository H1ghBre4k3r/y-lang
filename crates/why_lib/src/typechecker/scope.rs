use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::parser::ast::Expression;

use super::{error::TypeCheckError, types::Type, TypeInformation, TypedConstruct};

type StoredVariable = (Expression<TypeInformation>, Rc<RefCell<Option<Type>>>);

#[derive(Clone, Default)]
pub struct Stack {
    variables: HashMap<String, StoredVariable>,
    types: HashMap<String, Type>,
    constants: HashMap<String, Type>,
}

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stack")
            .field(
                "variables",
                &self
                    .variables
                    .iter()
                    .map(|(name, (_, type_id))| (name, type_id.borrow().as_ref().cloned()))
                    .collect::<HashMap<_, _>>(),
            )
            .field("types", &self.types)
            .finish()
    }
}

type StackFrame = Rc<RefCell<Stack>>;

#[derive(Clone, Debug)]
pub struct Scope {
    stacks: Vec<StackFrame>,
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            stacks: vec![StackFrame::default()],
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

#[derive(Debug, Clone)]
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
        expression: Expression<TypeInformation>,
    ) -> Result<(), VariableAddError> {
        let name = name.to_string();

        if self.get_constant(&name).is_some() {
            return Err(VariableAddError { name });
        }

        self.stacks.last().and_then(|scope| {
            let type_id = expression.get_info().type_id;
            scope
                .borrow_mut()
                .variables
                .insert(name, (expression, type_id))
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
                    .map(|(_, type_id)| type_id)
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

        let Some((mut exp, variable_type)) = scope.variables.get(&name).cloned() else {
            unreachable!()
        };

        // explicitly drop scope to prevent borrow checker from crashing
        drop(scope);

        exp.update_type(type_id.clone())?;

        *variable_type.borrow_mut() = Some(type_id);
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
        self.stacks
            .iter()
            .rev()
            .find(|scope| scope.borrow().constants.contains_key(&name))
            .and_then(|scope| scope.borrow_mut().constants.get(&name).cloned())
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

    pub fn resolve_name(&mut self, name: impl ToString) -> Option<Rc<RefCell<Option<Type>>>> {
        let name = name.to_string();
        self.get_constant(&name)
            .map(|t| Rc::new(RefCell::new(Some(t))))
            .or_else(|| self.get_variable(&name))
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
            .add_variable("foo", expression)
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
            .add_variable("foo", expression.clone())
            .expect("something went wrong");

        *expression.get_info().type_id.borrow_mut() = Some(Type::Boolean);

        scope
            .add_variable("foo", expression.clone())
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
            .add_variable("foo", expression.clone())
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
            .add_variable("foo", expression.clone())
            .expect("something went wrong");

        let foo = scope.get_variable("foo").unwrap();
        let bar = scope.get_variable("foo").unwrap();

        assert_eq!(foo, bar);

        *foo.borrow_mut() = None;

        assert_eq!(foo, Rc::new(RefCell::new(None)));
        assert_eq!(bar, Rc::new(RefCell::new(None)));
    }
}
