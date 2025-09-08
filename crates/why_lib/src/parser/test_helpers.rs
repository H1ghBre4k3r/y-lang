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

