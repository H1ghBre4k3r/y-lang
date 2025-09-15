mod context;
mod error;
mod scope;
mod typed_ast;
mod types;

use crate::lexer::Span;
use crate::parser::ast::TopLevelStatement;
use error::{InvalidMainSignature, MissingMainFunction};
use std::fmt::{Display, Formatter};
use std::{cell::RefCell, error::Error, fmt::Debug, rc::Rc};

use self::context::Context;
pub use self::{error::TypeCheckError, types::Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInformation {
    pub type_id: Rc<RefCell<Option<Type>>>,
    pub context: Context,
}

impl TypeInformation {
    pub fn has_type(&self) -> bool {
        self.type_id.borrow().is_some()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidatedTypeInformation {
    pub type_id: Type,
    #[serde(skip)]
    pub context: Context,
}

impl TypeInformation {
    fn validate(self, position: &Span) -> Result<ValidatedTypeInformation, TypeValidationError> {
        let TypeInformation { type_id, context } = self;
        let verified_type_information = if let Some(type_id) = type_id.borrow().clone() {
            Ok(ValidatedTypeInformation { type_id, context })
        } else {
            Err(TypeValidationError(position.clone()))
        };

        verified_type_information
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TypeValidationError(Span);

impl TypeValidationError {
    const MESSAGE: &'static str = "Type must be known at compile time!";

    pub fn span(&self) -> Span {
        self.0.clone()
    }

    pub fn err(&self) -> String {
        Self::MESSAGE.to_string()
    }
}

impl Display for TypeValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_string(Self::MESSAGE).as_str())
    }
}

impl Error for TypeValidationError {}

pub type TypeResult<T> = Result<T, TypeCheckError>;

#[derive(Debug, Clone, Default)]
pub struct TypeChecker {
    context: Context,
    statements: Vec<TopLevelStatement<()>>,
}

trait TypeCheckable {
    type Typed;

    fn check(self, ctx: &mut Context) -> TypeResult<Self::Typed>;

    fn revert(this: &Self::Typed) -> Self;
}

trait ShallowCheck {
    fn shallow_check(&self, ctx: &mut Context) -> TypeResult<()>;
}

trait TypedConstruct
where
    Self: Debug,
{
    type Validated;

    fn update_type(&mut self, type_id: Type) -> TypeResult<()> {
        unimplemented!(
            "TypedConstruct::update_type({type_id:?}) is not implemented for {:?}",
            self
        )
    }

    fn validate(self) -> Result<Self::Validated, TypeValidationError>;
}

impl TypeChecker {
    pub fn new(statements: Vec<TopLevelStatement<()>>) -> TypeChecker {
        TypeChecker {
            statements,
            ..Default::default()
        }
    }

    fn shallow_check(&mut self) -> TypeResult<()> {
        let struct_declarations = self
            .statements
            .iter()
            .filter(|stm| matches!(stm, TopLevelStatement::StructDeclaration(_)))
            .collect::<Vec<_>>();

        let other_tl_statements = self
            .statements
            .iter()
            .filter(|stm| !matches!(stm, TopLevelStatement::StructDeclaration(_)))
            .collect::<Vec<_>>();

        for s in struct_declarations.iter() {
            s.shallow_check(&mut self.context)?;
        }

        for s in other_tl_statements.iter() {
            s.shallow_check(&mut self.context)?;
        }

        Ok(())
    }

    pub fn check(mut self) -> TypeResult<Vec<TopLevelStatement<TypeInformation>>> {
        self.shallow_check()?;

        // Process function definitions first, then other statements
        let function_declaration_indices = self
            .statements
            .iter()
            .enumerate()
            .filter(|(_, stm)| matches!(stm, TopLevelStatement::Function(_)))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let other_statement_indices = self
            .statements
            .iter()
            .enumerate()
            .filter(|(_, stm)| !matches!(stm, TopLevelStatement::Function(_)))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let mut checked = vec![];

        // Process function definitions first
        for &i in &function_declaration_indices {
            checked.push(self.statements[i].clone().check(&mut self.context)?);
        }

        // Then process other statements (which may call the functions)
        for &i in &other_statement_indices {
            checked.push(self.statements[i].clone().check(&mut self.context)?);
        }

        // CRITICAL FIX: After all statements are processed, update function call sites
        // with the latest closure type information from scope
        eprintln!("DEBUG: Starting post-processing to fix function call types");
        self.finalize_function_call_types(&mut checked)?;
        eprintln!("DEBUG: Completed post-processing function call types");

        self.check_main_function()?;

        Ok(checked)
    }

    /// Post-processing step to update all function call sites with the latest closure type information
    fn finalize_function_call_types(&mut self, statements: &mut [TopLevelStatement<TypeInformation>]) -> Result<(), TypeCheckError> {
        for statement in statements {
            match statement {
                TopLevelStatement::Function(func) => {
                    // Update function calls in function body by iterating through statements
                    for stmt in &mut func.body.statements {
                        self.update_function_calls_in_statement(stmt)?;
                    }
                }
                _ => {
                    // Update function calls in other statement types if needed
                }
            }
        }
        Ok(())
    }

    fn update_function_calls_in_statement(&mut self, statement: &mut crate::parser::ast::Statement<TypeInformation>) -> Result<(), TypeCheckError> {
        match statement {
            crate::parser::ast::Statement::Expression(expr) |
            crate::parser::ast::Statement::YieldingExpression(expr) => {
                eprintln!("DEBUG: FINALIZER - Processing expression statement");
                self.refresh_function_call_types_in_expression(expr)?;
            }
            crate::parser::ast::Statement::Initialization(init) => {
                eprintln!("DEBUG: FINALIZER - Processing initialization statement");
                self.refresh_function_call_types_in_expression(&mut init.value)?;
            }
            _ => {
                eprintln!("DEBUG: FINALIZER - Processing other statement type");
            }
        }
        Ok(())
    }

    fn refresh_function_call_types_in_expression(&mut self, expr: &mut crate::parser::ast::Expression<TypeInformation>) -> Result<(), TypeCheckError> {
        match expr {
            crate::parser::ast::Expression::Postfix(postfix) => {
                match postfix {
                    crate::parser::ast::Postfix::Call { expr: call_expr, info, .. } => {
                        // Check if this is a call to a function and refresh the return type
                        if let crate::parser::ast::Expression::Id(id_expr) = call_expr.as_ref() {
                            if let Some(updated_function_type) = self.context.scope.resolve_name(&id_expr.name) {
                                if let Some(Type::Function { return_value, .. }) = updated_function_type.borrow().as_ref() {
                                    if matches!(return_value.as_ref(), Type::Closure { .. }) {
                                        eprintln!("DEBUG: FINALIZER - Updating call to {} with closure return type: {:?}", id_expr.name, return_value.as_ref());
                                        *info.type_id.borrow_mut() = Some(return_value.as_ref().clone());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                eprintln!("DEBUG: FINALIZER - Processing other expression type");
            }
        }
        Ok(())
    }

    fn check_main_function(&mut self) -> Result<(), TypeCheckError> {
        let main = self.context.scope.resolve_name("main");

        let Some(main) = main else {
            return Err(TypeCheckError::MissingMainFunction(MissingMainFunction));
        };

        let main = { main.borrow().clone().unwrap() };

        match main {
            Type::Function {
                params,
                return_value,
            } => {
                if !params.is_empty()
                    && (*return_value != Type::Void || *return_value != Type::Integer)
                {
                    // TODO: we need to return the correct span of the main function for better
                    // error display
                    return Err(TypeCheckError::InvalidMainSignature(
                        InvalidMainSignature,
                        Span::default(),
                    ));
                }
            }
            _ => return Err(TypeCheckError::MissingMainFunction(MissingMainFunction)),
        }

        Ok(())
    }

    pub fn validate(
        statements: Vec<TopLevelStatement<TypeInformation>>,
    ) -> Result<Vec<TopLevelStatement<ValidatedTypeInformation>>, TypeValidationError> {
        let mut validated = vec![];

        for stm in statements {
            validated.push(stm.validate()?);
        }

        Ok(validated)
    }
}
