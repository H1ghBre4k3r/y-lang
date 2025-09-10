use crate::{
    grammar::{self, FromGrammar},
    parser::ast::{Expression, Function, Statement, TopLevelStatement, TypeName},
};

/// Helper function to parse a single expression by wrapping it in a minimal function context
pub fn parse_expression(code: &str) -> Result<Expression<()>, String> {
    let wrapped = format!("fn main(): void {{ {}; }}", code);
    let program = grammar::parse(&wrapped).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), &wrapped);
        if let TopLevelStatement::Function(function) = top_level {
            if let Some(Statement::Expression(expr)) = function.statements.first() {
                return Ok(expr.clone());
            }
        }
    }

    Err("Failed to extract expression from parsed result".to_string())
}

/// Helper function to parse a single statement by wrapping it in a minimal function context
pub fn parse_statement(code: &str) -> Result<Statement<()>, String> {
    let wrapped = format!("fn main(): void {{ {} }}", code);
    let program = grammar::parse(&wrapped).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), &wrapped);
        if let TopLevelStatement::Function(function) = top_level {
            if let Some(stmt) = function.statements.first() {
                return Ok(stmt.clone());
            }
        }
    }

    Err("Failed to extract statement from parsed result".to_string())
}

/// Helper function to parse a type name by wrapping it in a declaration context
pub fn parse_type_name(code: &str) -> Result<TypeName, String> {
    let wrapped = format!("declare x: {};", code);
    let program = grammar::parse(&wrapped).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), &wrapped);
        if let TopLevelStatement::Declaration(declaration) = top_level {
            return Ok(declaration.type_name);
        }
    }

    Err("Failed to extract type name from parsed result".to_string())
}

/// Helper function to parse a complete function
pub fn parse_function(code: &str) -> Result<Function<()>, String> {
    let program = grammar::parse(code).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), code);
        if let TopLevelStatement::Function(function) = top_level {
            return Ok(function);
        }
    }

    Err("Failed to extract function from parsed result".to_string())
}

/// Helper function to parse a program and extract the first statement of a specific type
pub fn parse_program_single<T>(code: &str) -> Result<T, String>
where
    T: TryFrom<TopLevelStatement<()>>,
{
    let program = grammar::parse(code).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), code);
        return T::try_from(top_level).map_err(|_| "Failed to convert to target type".to_string());
    }

    Err("No statements found in program".to_string())
}

/// Helper to parse expressions that should be wrapped in yielding context
pub fn parse_yielding_expression(code: &str) -> Result<Expression<()>, String> {
    let wrapped = format!("fn main(): void {{ {} }}", code);
    let program = grammar::parse(&wrapped).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), &wrapped);
        if let TopLevelStatement::Function(function) = top_level {
            if let Some(Statement::YieldingExpression(expr)) = function.statements.first() {
                return Ok(expr.clone());
            }
        }
    }

    Err("Failed to extract yielding expression from parsed result".to_string())
}

/// Helper to parse a number literal directly
pub fn parse_number(code: &str) -> Result<crate::parser::ast::Num<()>, String> {
    let expr = parse_expression(code)?;
    match expr {
        Expression::Num(num) => Ok(num),
        _ => Err("Expected number expression".to_string()),
    }
}

/// Helper to parse an identifier directly  
pub fn parse_id(code: &str) -> Result<crate::parser::ast::Id<()>, String> {
    let expr = parse_expression(code)?;
    match expr {
        Expression::Id(id) => Ok(id),
        _ => Err("Expected identifier expression".to_string()),
    }
}

/// Helper to parse a string literal directly
pub fn parse_string(code: &str) -> Result<crate::parser::ast::AstString<()>, String> {
    let expr = parse_expression(code)?;
    match expr {
        Expression::AstString(string) => Ok(string),
        _ => Err("Expected string expression".to_string()),
    }
}

/// Helper to parse an array literal directly
pub fn parse_array(code: &str) -> Result<crate::parser::ast::Array<()>, String> {
    let expr = parse_expression(code)?;
    match expr {
        Expression::Array(array) => Ok(array),
        _ => Err("Expected array expression".to_string()),
    }
}

/// Helper to parse a character literal directly
pub fn parse_character(code: &str) -> Result<crate::parser::ast::Character<()>, String> {
    let expr = parse_expression(code)?;
    match expr {
        Expression::Character(character) => Ok(character),
        _ => Err("Expected character expression".to_string()),
    }
}

/// Helper to parse an if expression directly
pub fn parse_if(code: &str) -> Result<crate::parser::ast::If<()>, String> {
    let expr = parse_expression(code)?;
    match expr {
        Expression::If(if_expr) => Ok(if_expr),
        _ => Err("Expected if expression".to_string()),
    }
}

/// Helper to parse a struct initialization directly
pub fn parse_struct_init(
    code: &str,
) -> Result<crate::parser::ast::StructInitialisation<()>, String> {
    let expr = parse_expression(code)?;
    match expr {
        Expression::StructInitialisation(struct_init) => Ok(struct_init),
        _ => Err("Expected struct initialization expression".to_string()),
    }
}

/// Helper to parse a lambda expression directly
pub fn parse_lambda(code: &str) -> Result<crate::parser::ast::Lambda<()>, String> {
    let expr = parse_expression(code)?;
    match expr {
        Expression::Lambda(lambda) => Ok(lambda),
        _ => Err("Expected lambda expression".to_string()),
    }
}

/// Helper to parse a declaration directly
pub fn parse_declaration(code: &str) -> Result<crate::parser::ast::Declaration<()>, String> {
    let program = grammar::parse(code).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), code);
        if let TopLevelStatement::Declaration(declaration) = top_level {
            return Ok(declaration);
        }
    }

    Err("Failed to extract declaration from parsed result".to_string())
}

/// Helper to parse an initialization statement by wrapping it in a function context
pub fn parse_initialization(code: &str) -> Result<crate::parser::ast::Initialisation<()>, String> {
    let wrapped = format!("fn main(): void {{ {} }}", code);
    let program = grammar::parse(&wrapped).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), &wrapped);
        if let TopLevelStatement::Function(function) = top_level {
            if let Some(Statement::Initialization(init)) = function.statements.first() {
                return Ok(init.clone());
            }
        }
    }

    Err("Failed to extract initialization from parsed result".to_string())
}

/// Helper to parse an assignment statement by wrapping it in a function context
pub fn parse_assignment(code: &str) -> Result<crate::parser::ast::Assignment<()>, String> {
    let wrapped = format!("fn main(): void {{ {}; }}", code);
    let program = grammar::parse(&wrapped).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), &wrapped);
        if let TopLevelStatement::Function(function) = top_level {
            if let Some(Statement::Assignment(assignment)) = function.statements.first() {
                return Ok(assignment.clone());
            }
        }
    }

    Err("Failed to extract assignment from parsed result".to_string())
}

/// Helper to parse a block by wrapping it in a function context
pub fn parse_block(code: &str) -> Result<crate::parser::ast::Block<()>, String> {
    let wrapped = format!("fn test(): void {}", code);
    let program = grammar::parse(&wrapped).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), &wrapped);
        if let TopLevelStatement::Function(function) = top_level {
            // Return a Block constructed from the function's statements
            return Ok(crate::parser::ast::Block {
                statements: function.statements,
                info: (),
                position: crate::lexer::Span::default(),
            });
        }
    }

    Err("Failed to extract block from parsed result".to_string())
}

/// Helper to parse a boolean literal by wrapping it in a function context
pub fn parse_bool(code: &str) -> Result<crate::parser::ast::Bool<()>, String> {
    let wrapped = format!("fn main(): void {{ {}; }}", code);
    let program = grammar::parse(&wrapped).map_err(|e| format!("Parse error: {:?}", e))?;

    if let Some(statement) = program.statements.first() {
        let top_level = TopLevelStatement::transform(statement.clone(), &wrapped);
        if let TopLevelStatement::Function(function) = top_level {
            if let Some(Statement::Expression(Expression::Bool(bool_val))) =
                function.statements.first()
            {
                return Ok(bool_val.clone());
            }
        }
    }

    Err("Failed to extract boolean from parsed result".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let result = parse_number("42").unwrap();
        assert!(matches!(
            result,
            crate::parser::ast::Num::Integer(42, (), _)
        ));
    }

    #[test]
    fn test_parse_id() {
        let result = parse_id("foo").unwrap();
        assert_eq!(result.name, "foo");
    }

    #[test]
    fn test_parse_type_name() {
        let result = parse_type_name("i32").unwrap();
        assert!(matches!(result, TypeName::Literal(ref name, _) if name == "i32"));
    }
}
