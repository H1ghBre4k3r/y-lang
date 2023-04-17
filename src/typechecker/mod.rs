mod error;
mod fn_extractor;
mod info;
mod typescope;
mod variabletype;

use crate::{
    ast::{
        Array, Assignment, Ast, BinaryExpr, BinaryOp, Block, Boolean, Call, Character,
        CompilerDirective, Declaration, Definition, Expression, FnDef, Ident, If, Import, Indexing,
        InlineAssembly, Integer, Intrinsic, Param, Position, PostfixExpr, PostfixOp, PrefixExpr,
        PrefixOp, Statement, Str, Type, WhileLoop,
    },
    loader::Modules,
};

pub use self::fn_extractor::extract_exports;
pub use self::info::TypeInfo;
pub use self::typescope::TypeScope;
pub use self::variabletype::VariableType;

use self::{error::TypeError, typescope::setup_scope};

type TResult<T> = Result<T, TypeError>;

pub struct Typechecker {
    ast: Ast<()>,
    modules: Modules<TypeInfo>,
}

impl Typechecker {
    pub fn from_ast(ast: Ast<()>, modules: Modules<TypeInfo>) -> Self {
        Self { ast, modules }
    }

    pub fn check(&self) -> Result<Ast<TypeInfo>, TypeError> {
        let nodes = self.ast.nodes();

        let mut scope = setup_scope();

        let mut statements = vec![];

        for node in nodes {
            statements.push(self.check_statement(&node, &mut scope)?);
        }

        Ok(Ast::from_nodes(statements))
    }

    pub fn extract_exports(ast: &Ast<()>) -> Result<TypeScope, TypeError> {
        let nodes = ast.nodes();

        let mut scope = setup_scope();

        for intrinsic in nodes.iter() {
            match intrinsic {
                Statement::Intrinsic(Intrinsic::Definition(definition)) => {
                    let Definition { value, ident, .. } = definition;

                    let Expression::FnDef(FnDef { params, type_annotation , position, ..}) = value else {
                        continue;
                    };

                    let mut param_types = vec![];

                    for Param {
                        type_annotation,
                        position,
                        ..
                    } in params
                    {
                        param_types.push(Self::get_type_def(
                            &type_annotation.value,
                            position.clone(),
                        )?);
                    }

                    scope.set(
                        &ident.value,
                        VariableType::Func {
                            params: param_types,
                            return_type: Box::new(Self::get_type_def(
                                &type_annotation.value,
                                position.clone(),
                            )?),
                            source: None,
                        },
                        false,
                    )
                }
                Statement::Intrinsic(Intrinsic::Declaration(declaration)) => {
                    let Declaration {
                        ident,
                        type_annotation,
                        position,
                        ..
                    } = declaration;
                    let type_annotation =
                        Self::get_type_def(&type_annotation.value, position.clone())?;

                    if let VariableType::Func { .. } = &type_annotation {
                        scope.set(&ident.value, type_annotation, false);
                    }
                }
                Statement::CompilerDirective(compiler_directive) => {
                    todo!()
                }
                _ => {}
            }
        }
        Ok(scope)
    }

    fn check_statement(
        &self,
        statement: &Statement<()>,
        scope: &mut TypeScope,
    ) -> TResult<Statement<TypeInfo>> {
        Ok(match &statement {
            Statement::Expression(expression) => {
                Statement::Expression(self.check_expression(None, expression, scope)?)
            }
            Statement::Intrinsic(intrinsic) => {
                Statement::Intrinsic(self.check_intrinsic(intrinsic, scope)?)
            }
            Statement::Import(import) => Statement::Import(self.check_import(import, scope)?),
            Statement::CompilerDirective(compiler_directive) => Statement::CompilerDirective(
                self.check_compiler_directive(compiler_directive, scope)?,
            ),
            Statement::InlineAssembly(inline_assembly) => {
                Statement::InlineAssembly(self.check_inline_assembly(inline_assembly, scope)?)
            }
        })
    }

    fn check_inline_assembly(
        &self,
        InlineAssembly {
            statements,
            position,
            ..
        }: &InlineAssembly<()>,
        _: &mut TypeScope,
    ) -> TResult<InlineAssembly<TypeInfo>> {
        Ok(InlineAssembly {
            statements: statements.clone(),
            position: position.clone(),
            info: TypeInfo {
                source: None,
                _type: VariableType::Unknown,
            },
        })
    }

    fn check_compiler_directive(
        &self,
        CompilerDirective {
            directive,
            statement,
            position,
        }: &CompilerDirective<()>,
        scope: &mut TypeScope,
    ) -> TResult<CompilerDirective<TypeInfo>> {
        let Expression::Binary(directive) = directive.clone() else {
            unimplemented!("Currently only compiler directives in the form of binary expressions are supported!");
        };
        let Some(statement) = statement.clone() else {
            return Ok(CompilerDirective {
                directive: Expression::Binary(directive),
                statement: None,
                position: position.to_owned()
            });
        };

        let is_valid = match (directive.lhs.as_ref(), directive.rhs.as_ref()) {
            (Expression::Ident(ident), Expression::Str(rhs)) => match ident.value.as_str() {
                "os" => std::env::consts::OS == rhs.value,
                _ => false,
            },
            _ => unimplemented!(
                "Currently only compiler directives in the form of 'ident == str' are supported!"
            ),
        };

        if is_valid {
            Ok(CompilerDirective {
                directive: Expression::Binary(directive),
                statement: Some(Box::new(self.check_statement(&statement, scope)?)),
                position: position.clone(),
            })
        } else {
            Ok(CompilerDirective {
                directive: Expression::Binary(directive),
                statement: None,
                position: position.clone(),
            })
        }
    }

    fn check_import(&self, import: &Import, scope: &mut TypeScope) -> TResult<Import> {
        let Import { path, position } = import;
        let Some(module) = self.modules.get(path) else {
           return Err(TypeError {
               message: format!("Could not import module '{path}'"),
               position: position.clone()
           });
        };

        let imports = module.exports.flatten();

        for (key, value) in imports {
            if module.is_wildcard {
                scope.set(&key, value.variable_type.set_source(module.clone()), false);
            } else {
                scope.set(
                    &format!("{path}::{key}"),
                    value.variable_type.set_source(module.clone()),
                    false,
                );
            }
        }

        Ok(import.clone())
    }

    fn check_intrinsic(
        &self,
        intrinsic: &Intrinsic<()>,
        scope: &mut TypeScope,
    ) -> TResult<Intrinsic<TypeInfo>> {
        Ok(match &intrinsic {
            Intrinsic::Definition(definition) => {
                Intrinsic::Definition(self.check_definition(definition, scope)?)
            }
            Intrinsic::Assignment(assignment) => {
                Intrinsic::Assignment(self.check_assignment(assignment, scope)?)
            }
            Intrinsic::Declaration(declaration) => {
                Intrinsic::Declaration(self.check_declaration(declaration, scope)?)
            }
            Intrinsic::WhileLoop(while_loop) => {
                Intrinsic::WhileLoop(self.check_while_loop(while_loop, scope)?)
            }
        })
    }

    fn check_while_loop(
        &self,
        WhileLoop {
            condition,
            block,
            position,
            ..
        }: &WhileLoop<()>,
        scope: &mut TypeScope,
    ) -> TResult<WhileLoop<TypeInfo>> {
        let condition = self.check_expression(None, condition, scope)?;
        if condition.info()._type != VariableType::Bool {
            return Err(TypeError {
                message: format!("Invalid type of condition '{}'", condition.info()._type),
                position: position.to_owned(),
            });
        }

        let block = self.check_block(block, scope)?;

        Ok(WhileLoop {
            condition,
            block,
            position: position.to_owned(),
            info: TypeInfo {
                _type: VariableType::Void,
                source: None,
            },
        })
    }

    fn check_declaration(
        &self,
        declaration: &Declaration,
        scope: &mut TypeScope,
    ) -> TResult<Declaration> {
        let ident = &declaration.ident;
        let type_annotation = &declaration.type_annotation;
        let type_def =
            Self::get_type_def(&type_annotation.value, type_annotation.position.clone())?;

        scope.set(&ident.value, type_def, false);
        Ok(declaration.clone())
    }

    fn check_if(&self, if_statement: &If<()>, scope: &mut TypeScope) -> TResult<If<TypeInfo>> {
        let condition = self.check_expression(None, &if_statement.condition, scope)?;
        let condition_info = condition.info();
        let condition_type = condition_info._type;

        if condition_type != VariableType::Bool {
            return Err(TypeError {
                message: format!("Invalid tye of condition '{condition_type:?}'"),
                position: if_statement.condition.position(),
            });
        }

        let if_block = self.check_block(&if_statement.if_block, scope)?;
        let if_block_type = if_block.info._type.clone();

        let mut new_if = If {
            condition: Box::new(condition),
            if_block,
            else_block: None,
            position: if_statement.position.clone(),
            info: TypeInfo {
                _type: if_block_type.clone(),
                source: None,
            },
        };

        if let Some(else_block) = &if_statement.else_block {
            let else_block = self.check_block(else_block, scope)?;
            let else_block_type = else_block.info._type.clone();

            if if_block_type != else_block_type {
                return Err(TypeError {
                    message: format!(
                        "Return type mismatch of if-else. Got '{if_block_type}' and '{else_block_type}'"
                    ),
                    position: if_statement.position.clone(),
                });
            }

            new_if.else_block = Some(else_block);
        }

        Ok(new_if)
    }

    fn check_block(&self, block: &Block<()>, scope: &mut TypeScope) -> TResult<Block<TypeInfo>> {
        scope.push();

        let mut new_block = Block {
            position: block.position.clone(),
            block: vec![],
            info: TypeInfo {
                _type: VariableType::Void,
                source: None,
            },
        };

        for statement in &block.block {
            let statement = self.check_statement(statement, scope)?;
            new_block.info._type = statement.info()._type;
            new_block.block.push(statement);
        }

        scope.pop();

        Ok(new_block)
    }

    fn check_definition(
        &self,
        definition: &Definition<()>,
        scope: &mut TypeScope,
    ) -> TResult<Definition<TypeInfo>> {
        let definition_rhs =
            self.check_expression(Some(&definition.ident), &definition.value, scope)?;

        if scope.contains_in_current_scope(&definition.ident.value) {
            return Err(TypeError {
                message: format!(
                    "Variable '{}' has already been defined!",
                    definition.ident.value
                ),
                position: definition.position.clone(),
            });
        }

        scope.set(
            &definition.ident.value,
            definition_rhs.info()._type,
            definition.is_mutable,
        );

        let ident = &definition.ident;

        Ok(Definition {
            ident: Ident {
                position: ident.position.clone(),
                value: ident.value.clone(),
                info: definition_rhs.info(),
            },
            value: definition_rhs,
            position: definition.position.clone(),
            is_mutable: definition.is_mutable,
            info: TypeInfo {
                _type: VariableType::Void,
                source: None,
            },
        })
    }

    fn check_assignment(
        &self,
        assignment: &Assignment<()>,
        scope: &mut TypeScope,
    ) -> TResult<Assignment<TypeInfo>> {
        let lhs = &assignment.lhs;

        match lhs {
            Expression::Postfix(PostfixExpr {
                op: PostfixOp::Indexing(indexing),
                lhs: indexing_lhs,
                position,
                ..
            }) => {
                let indexing_lhs = self.check_expression(None, indexing_lhs, scope)?;
                let indexing = self.check_indexing(&indexing_lhs, indexing, scope)?;

                let assignment_rhs = self.check_expression(None, &assignment.value, scope)?;

                if assignment_rhs
                    .info()
                    ._type
                    .convert_to(&indexing.info._type)
                    .is_err()
                {
                    return Err(TypeError {
                        message: format!(
                            "Can not assign value of type '{}' to indexed variable of type '{}'",
                            assignment_rhs.info()._type,
                            indexing.info._type
                        ),
                        position: assignment.position.clone(),
                    });
                }

                Ok(Assignment {
                    lhs: Expression::Postfix(PostfixExpr {
                        op: PostfixOp::Indexing(indexing),
                        lhs: Box::new(indexing_lhs),
                        position: position.clone(),
                        info: assignment_rhs.info(),
                    }),
                    value: assignment_rhs,
                    position: assignment.position.clone(),
                    info: TypeInfo {
                        source: None,
                        _type: VariableType::Void,
                    },
                })
            }
            Expression::Ident(lhs) => {
                if !scope.contains(&lhs.value) {
                    return Err(TypeError {
                        message: format!("Undefined identifier '{}'", lhs.value),
                        position: lhs.position.clone(),
                    });
                }

                if !scope.is_mutable(&lhs.value) {
                    return Err(TypeError {
                        message: format!(
                    "Variable '{}' can not be modified, because it is not defined in current scope",
                    lhs.value
                ),
                        position: lhs.position.clone(),
                    });
                }

                let assignment_rhs = self.check_expression(Some(lhs), &assignment.value, scope)?;

                scope.update(
                    &lhs.value,
                    assignment_rhs.info()._type,
                    &assignment.position,
                )?;

                Ok(Assignment {
                    lhs: Expression::Ident(Ident {
                        position: lhs.position.clone(),
                        value: lhs.value.clone(),
                        info: assignment_rhs.info(),
                    }),
                    value: assignment_rhs,
                    position: assignment.position.clone(),
                    info: TypeInfo {
                        source: None,
                        _type: VariableType::Void,
                    },
                })
            }
            _ => Err(TypeError {
                message: format!("Invalid lvalue of assignment '{lhs:?}'"),
                position: lhs.position(),
            }),
        }
    }

    fn check_expression(
        &self,
        identifier: Option<&Ident<()>>,
        expression: &Expression<()>,
        scope: &mut TypeScope,
    ) -> TResult<Expression<TypeInfo>> {
        Ok(match expression {
            Expression::If(if_statement) => Expression::If(self.check_if(if_statement, scope)?),
            Expression::Binary(binary_expr) => {
                Expression::Binary(self.check_binary_expression(binary_expr, scope)?)
            }
            Expression::Integer(Integer {
                value, position, ..
            }) => Expression::Integer(Integer {
                value: *value,
                position: position.clone(),
                info: TypeInfo {
                    _type: VariableType::Int,
                    source: None,
                },
            }),
            Expression::Str(Str {
                value, position, ..
            }) => Expression::Str(Str {
                value: value.to_owned(),
                position: position.clone(),
                info: TypeInfo {
                    _type: VariableType::Str,
                    source: None,
                },
            }),
            Expression::Boolean(Boolean {
                value, position, ..
            }) => Expression::Boolean(Boolean {
                value: *value,
                position: position.clone(),
                info: TypeInfo {
                    _type: VariableType::Bool,
                    source: None,
                },
            }),
            Expression::Ident(ident) => Expression::Ident(self.check_identifier(ident, scope)?),
            Expression::Prefix(prefix_expr) => {
                Expression::Prefix(self.check_prefix_expression(prefix_expr, scope)?)
            }
            Expression::Postfix(postfix_expr) => {
                Expression::Postfix(self.check_postfix_expression(postfix_expr, scope)?)
            }
            Expression::FnDef(fn_def) => {
                Expression::FnDef(self.check_fn_def(identifier, fn_def, scope)?)
            }
            Expression::Block(block) => Expression::Block(self.check_block(block, scope)?),
            Expression::Array(array) => Expression::Array(self.check_array(array, scope)?),
            Expression::Character(Character {
                value, position, ..
            }) => Expression::Character(Character {
                value: *value,
                position: position.clone(),
                info: TypeInfo {
                    _type: VariableType::Char,
                    source: None,
                },
            }),
        })
    }

    fn check_array(
        &self,
        Array {
            initializer,
            size,
            position,
            ..
        }: &Array<()>,
        scope: &mut TypeScope,
    ) -> TResult<Array<TypeInfo>> {
        let initializer = self.check_expression(None, initializer, scope)?;

        Ok(Array {
            initializer: Box::new(initializer.clone()),
            size: size.to_owned(),
            position: position.to_owned(),
            info: TypeInfo {
                _type: VariableType::TupleArray {
                    item_type: Box::new(initializer.info()._type),
                    size: if size.value >= 0 {
                        size.value as usize
                    } else {
                        return Err(TypeError {
                            message: "Negative length arrays are not supported!".to_string(),
                            position: position.clone(),
                        });
                    },
                },
                source: initializer.info()._type.get_source(),
            },
        })
    }

    fn check_identifier(
        &self,
        identifier: &Ident<()>,
        scope: &mut TypeScope,
    ) -> TResult<Ident<TypeInfo>> {
        match scope.find(&identifier.value) {
            Some(identifier_type) => Ok(Ident {
                value: identifier.value.clone(),
                position: identifier.position.clone(),
                info: TypeInfo {
                    _type: identifier_type,
                    source: None,
                },
            }),
            None => Err(TypeError {
                message: format!("Undefined identifier '{}'", identifier.value),
                position: identifier.position.clone(),
            }),
        }
    }

    fn get_type_def(type_: &Type, position: Position) -> Result<VariableType, TypeError> {
        match type_ {
            Type::Literal(literal) => literal.parse().map_err(|_| TypeError {
                message: format!("Unexpected type annotation '{type_:?}'"),
                position,
            }),
            Type::Function {
                params,
                return_type,
            } => {
                let mut fn_params = vec![];
                for param in params {
                    fn_params.push(Self::get_type_def(param, position.clone())?);
                }

                let return_type = Self::get_type_def(return_type, position)?;
                Ok(VariableType::Func {
                    return_type: Box::new(return_type),
                    params: fn_params,
                    source: None,
                })
            }
            Type::ArraySlice(item_type) => {
                let item_type = Self::get_type_def(item_type, position)?;

                Ok(VariableType::ArraySlice(Box::new(item_type)))
            }
            Type::TupleArray { item_type, size } => {
                let item_type = Self::get_type_def(item_type, position.clone())?;

                Ok(VariableType::TupleArray {
                    item_type: Box::new(item_type),
                    size: if size.value >= 0 {
                        size.value as usize
                    } else {
                        return Err(TypeError {
                            message: "Negative length arrays are not supported!".to_string(),
                            position,
                        });
                    },
                })
            }
        }
    }

    fn check_fn_def(
        &self,
        identifier: Option<&Ident<()>>,
        fn_def: &FnDef<()>,
        scope: &mut TypeScope,
    ) -> TResult<FnDef<TypeInfo>> {
        let type_annotation = Self::get_type_def(
            &fn_def.type_annotation.value,
            fn_def.type_annotation.position.clone(),
        )?;
        scope.push();

        let mut params = vec![];

        for param in &fn_def.params {
            let param_type = Self::get_type_def(
                &param.type_annotation.value,
                param.type_annotation.position.clone(),
            )?;

            scope.set(&param.ident.value, param_type.clone(), false);
            params.push(param_type);
        }

        if let Some(ident) = identifier {
            scope.set(
                &ident.value,
                VariableType::Func {
                    params: params.clone(),
                    return_type: Box::new(type_annotation.clone()),
                    source: None,
                },
                // TODO: This should handle mutable definitions
                false,
            )
        }

        let block = self.check_block(&fn_def.block, scope)?;

        let Ok(return_type) = block.info._type.convert_to(&type_annotation) else {
            return Err(TypeError {
                message: format!(
                    "Expected return type of '{type_annotation}' but got '{}'", block.info._type
                ),
                position: fn_def.position.clone(),
            });
        };

        scope.pop();

        Ok(FnDef {
            params: self.check_fn_params(&fn_def.params)?,
            type_annotation: fn_def.type_annotation.clone(),
            block,
            position: fn_def.position.clone(),
            info: TypeInfo {
                _type: VariableType::Func {
                    params,
                    return_type: Box::new(return_type),
                    source: None,
                },
                source: None,
            },
        })
    }

    fn check_fn_params(&self, params: &Vec<Param<()>>) -> TResult<Vec<Param<TypeInfo>>> {
        let mut new_params = vec![];

        for param in params {
            let Ident {
                value, position, ..
            } = &param.ident;
            let type_annotation = &param.type_annotation;
            let param_type =
                Self::get_type_def(&type_annotation.value, type_annotation.position.clone())?;

            new_params.push(Param {
                ident: Ident {
                    value: value.clone(),
                    position: position.clone(),
                    info: TypeInfo {
                        _type: param_type,
                        source: None,
                    },
                },
                position: param.position.clone(),
                type_annotation: type_annotation.clone(),
            });
        }

        Ok(new_params)
    }

    fn check_fn_call(
        &self,
        ident: &Ident<()>,
        fn_call: &Call<()>,
        scope: &mut TypeScope,
    ) -> TResult<Call<TypeInfo>> {
        scope.push();

        let ident = &ident.value;

        let Some(fn_def) = scope.find(ident) else {
            return Err(TypeError {
                message: format!("Call to undefined function '{ident}'"),
                position: fn_call.position.clone(),
            });
        };

        let VariableType::Func { params, return_type, .. } = fn_def.clone() else {
            return Err(TypeError {
                message: format!("Trying to call an invalid function '{ident}'"),
                position: fn_call.position.clone(),
            });
        };

        if params.len() != fn_call.params.len() {
            return Err(TypeError {
                message: format!(
                    "Invalid amount of parameters! Expected {} but got {}",
                    params.len(),
                    fn_call.params.len()
                ),
                position: fn_call.position.clone(),
            });
        }

        let mut new_params = vec![];

        for (i, param) in params.iter().enumerate() {
            let call_param = self.check_expression(None, &fn_call.params[i], scope)?;
            let call_param_type = call_param.info()._type;

            if call_param_type.convert_to(param).is_err() {
                return Err(TypeError {
                    message: format!(
                        "Invalid type of parameter! Expected '{param}' but got '{call_param_type}'"
                    ),
                    position: fn_call.params[i].position(),
                });
            }

            new_params.push(call_param);
        }

        scope.pop();

        Ok(Call {
            params: new_params,
            position: fn_call.position.clone(),
            info: TypeInfo {
                _type: *return_type,
                source: fn_def.get_source(),
            },
        })
    }

    fn check_binary_expression(
        &self,
        binary_expression: &BinaryExpr<()>,
        scope: &mut TypeScope,
    ) -> TResult<BinaryExpr<TypeInfo>> {
        let position = binary_expression.position.clone();

        let lhs = &binary_expression.lhs;
        let rhs = &binary_expression.rhs;

        let lhs = self.check_expression(None, lhs, scope)?;
        let l_type = lhs.info()._type;

        let rhs = self.check_expression(None, rhs, scope)?;
        let r_type = rhs.info()._type;

        match binary_expression.op {
            BinaryOp::Equal => {
                if l_type != r_type {
                    return Err(TypeError {
                        message: format!(
                        "Left and right value of binary operation do not match! ('{l_type}' and '{r_type}')"
                    ),
                        position,
                    });
                }
                Ok(BinaryExpr {
                    op: binary_expression.op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    position: binary_expression.position.clone(),
                    info: TypeInfo {
                        _type: VariableType::Bool,
                        source: None,
                    },
                })
            }
            BinaryOp::LessThan | BinaryOp::GreaterThan => {
                if l_type != VariableType::Int || r_type != VariableType::Int {
                    return Err(TypeError {
                        message: format!(
                            "Invalid types for binary operation '{}'. Got '{}' and '{}'",
                            binary_expression.op, l_type, r_type
                        ),
                        position,
                    });
                }
                Ok(BinaryExpr {
                    op: binary_expression.op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    position: binary_expression.position.clone(),
                    info: TypeInfo {
                        _type: VariableType::Bool,
                        source: None,
                    },
                })
            }
            BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Times | BinaryOp::DividedBy => {
                if l_type != VariableType::Int {
                    return Err(TypeError {
                        message: format!(
                        "Left value of numeric binary operation has to be of type Int. Found '{l_type}'"
                    ),
                        position: lhs.position(),
                    });
                } else if r_type != VariableType::Int {
                    return Err(TypeError {
                        message: format!(
                        "Right value of numeric binary operation has to be of type Int. Found '{r_type}'"
                    ),
                        position: rhs.position(),
                    });
                }

                Ok(BinaryExpr {
                    op: binary_expression.op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    position: binary_expression.position.clone(),
                    info: TypeInfo {
                        _type: VariableType::Int,
                        source: None,
                    },
                })
            }
        }
    }

    fn check_prefix_expression(
        &self,
        prefix_expression: &PrefixExpr<()>,
        scope: &mut TypeScope,
    ) -> TResult<PrefixExpr<TypeInfo>> {
        let position = prefix_expression.position.clone();

        let rhs = &prefix_expression.rhs;

        let rhs = self.check_expression(None, rhs, scope)?;
        let r_type = rhs.info()._type;

        match prefix_expression.op {
            PrefixOp::Not => {
                if r_type != VariableType::Bool {
                    return Err(TypeError {
                        message: format!(
                            "Invalid type for boolean prefix operation '{}'. Got '{}'",
                            prefix_expression.op, r_type
                        ),
                        position,
                    });
                }
                Ok(PrefixExpr {
                    op: prefix_expression.op,
                    rhs: Box::new(rhs),
                    position,
                    info: TypeInfo {
                        _type: VariableType::Bool,
                        source: None,
                    },
                })
            }
            PrefixOp::UnaryMinus => {
                if r_type != VariableType::Int {
                    return Err(TypeError {
                        message: format!(
                            "Invalid type for integral prefix operation '{}'. Got '{}'",
                            prefix_expression.op, r_type
                        ),
                        position,
                    });
                }
                Ok(PrefixExpr {
                    op: prefix_expression.op,
                    rhs: Box::new(rhs),
                    position,
                    info: TypeInfo {
                        _type: VariableType::Int,
                        source: None,
                    },
                })
            }
        }
    }

    fn check_postfix_expression(
        &self,
        postfix_expression: &PostfixExpr<()>,
        scope: &mut TypeScope,
    ) -> TResult<PostfixExpr<TypeInfo>> {
        let postfix_expression = postfix_expression.clone();

        let lhs = &postfix_expression.lhs;

        let lhs = self.check_expression(None, lhs, scope)?;

        match postfix_expression.op {
            PostfixOp::Call(call) => {
                let Expression::Ident(ident) = *postfix_expression.lhs else {
                    unimplemented!("Calls on non-identifier-expressions are not implemented yet")
                };
                let call = self.check_fn_call(&ident, &call, scope)?;
                let info = call.info.clone();
                Ok(PostfixExpr {
                    op: PostfixOp::Call(call),
                    lhs: Box::new(lhs),
                    position: postfix_expression.position,
                    info,
                })
            }
            PostfixOp::Indexing(indexing) => {
                let lhs = self.check_expression(None, &postfix_expression.lhs, scope)?;
                let indexing = self.check_indexing(&lhs, &indexing, scope)?;

                Ok(PostfixExpr {
                    op: PostfixOp::Indexing(indexing.clone()),
                    lhs: Box::new(lhs),
                    position: postfix_expression.position,
                    info: indexing.info,
                })
            }
        }
    }

    fn check_indexing(
        &self,
        lhs: &Expression<TypeInfo>,
        Indexing {
            index, position, ..
        }: &Indexing<()>,
        scope: &mut TypeScope,
    ) -> TResult<Indexing<TypeInfo>> {
        let Expression::Integer(index) = self.check_expression(None, index, scope)? else {
            unimplemented!("Indexing with a non-numeric index is currently not supported")
        };

        match lhs.info()._type {
            VariableType::ArraySlice(item_type) => Ok(Indexing {
                index: Box::new(Expression::Integer(index)),
                position: position.to_owned(),
                info: TypeInfo {
                    _type: *item_type.clone(),
                    source: item_type.get_source(),
                },
            }),
            VariableType::TupleArray { item_type, .. } => Ok(Indexing {
                index: Box::new(Expression::Integer(index)),
                position: position.to_owned(),
                info: TypeInfo {
                    _type: *item_type.clone(),
                    source: item_type.get_source(),
                },
            }),
            VariableType::Str => Ok(Indexing {
                index: Box::new(Expression::Integer(index)),
                position: position.to_owned(),
                info: TypeInfo {
                    _type: VariableType::Char,
                    source: lhs.info()._type.get_source(),
                },
            }),
            _ => unimplemented!("Indexing on non-array types is currently not supported"),
        }
    }
}
