use why_lib::{grammar, grammar::FromGrammar, parser::ast::*};

/// Integration tests for the complete rust-sitter parsing pipeline
/// These test the full grammar::parse -> FromGrammar::transform flow

#[test]
fn test_simple_function() {
    let code = r#"fn main(): i32 {
        42
    }"#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    assert_eq!(statements.len(), 1);
    match &statements[0] {
        TopLevelStatement::Function(func) => {
            assert_eq!(func.id.name, "main");
            assert_eq!(func.parameters.len(), 0);
            assert!(matches!(func.return_type, TypeName::Literal(ref name, _) if name == "i32"));
            assert_eq!(func.statements.len(), 1);
            assert!(matches!(
                func.statements[0],
                Statement::YieldingExpression(Expression::Num(Num::Integer(42, (), _)))
            ));
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_variable_declaration() {
    let code = r#"fn main(): void {
        let x: i32 = 42;
    }"#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    assert_eq!(statements.len(), 1);
    match &statements[0] {
        TopLevelStatement::Function(func) => {
            assert_eq!(func.statements.len(), 1);
            match &func.statements[0] {
                Statement::Initialization(init) => {
                    assert_eq!(init.id.name, "x");
                    assert!(
                        matches!(init.type_name, Some(TypeName::Literal(ref name, _)) if name == "i32")
                    );
                    assert!(matches!(
                        init.value,
                        Expression::Num(Num::Integer(42, (), _))
                    ));
                }
                _ => panic!("Expected variable initialization"),
            }
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_struct_declaration() {
    let code = r#"struct Point {
        x: i32;
        y: i32;
    }"#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    assert_eq!(statements.len(), 1);
    match &statements[0] {
        TopLevelStatement::StructDeclaration(struct_decl) => {
            assert_eq!(struct_decl.id.name, "Point");
            assert_eq!(struct_decl.fields.len(), 2);

            assert_eq!(struct_decl.fields[0].name.name, "x");
            assert!(
                matches!(struct_decl.fields[0].type_name, TypeName::Literal(ref name, _) if name == "i32")
            );

            assert_eq!(struct_decl.fields[1].name.name, "y");
            assert!(
                matches!(struct_decl.fields[1].type_name, TypeName::Literal(ref name, _) if name == "i32")
            );
        }
        _ => panic!("Expected struct declaration"),
    }
}

#[test]
fn test_binary_expression() {
    let code = r#"fn main(): i32 {
        1 + 2
    }"#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    match &statements[0] {
        TopLevelStatement::Function(func) => match &func.statements[0] {
            Statement::YieldingExpression(Expression::Binary(binary)) => {
                assert!(matches!(binary.operator, BinaryOperator::Add));
                assert!(matches!(
                    &binary.left,
                    Expression::Num(Num::Integer(1, (), _))
                ));
                assert!(matches!(
                    &binary.right,
                    Expression::Num(Num::Integer(2, (), _))
                ));
            }
            _ => panic!("Expected binary expression"),
        },
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_if_expression() {
    let code = r#"fn main(): void {
        if (true) {
            42;
        };
    }"#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    match &statements[0] {
        TopLevelStatement::Function(func) => {
            match &func.statements[0] {
                Statement::Expression(Expression::If(if_expr)) => {
                    // Check condition - should be true
                    // Check then statements - should have one statement (42;)
                    assert_eq!(if_expr.statements.len(), 1);
                    assert!(matches!(
                        if_expr.statements[0],
                        Statement::Expression(Expression::Num(Num::Integer(42, (), _)))
                    ));
                    assert_eq!(if_expr.else_statements.len(), 0);
                }
                _ => panic!("Expected if expression, got {:?}", &func.statements[0]),
            }
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_array_literal() {
    let code = r#"fn main(): void {
        &[1, 2, 3];
    }"#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    match &statements[0] {
        TopLevelStatement::Function(func) => {
            match &func.statements[0] {
                Statement::Expression(Expression::Array(Array::Literal { values, .. })) => {
                    assert_eq!(values.len(), 3);
                    // Check all values are numbers
                    for (i, value) in values.iter().enumerate() {
                        let i = i as u64;
                        assert!(
                            matches!(value, Expression::Num(Num::Integer(n, (), _)) if *n == i + 1)
                        );
                    }
                }
                _ => panic!("Expected array expression"),
            }
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_struct_initialization() {
    let code = r#"fn main(): void {
        Point { x: 1, y: 2 };
    }"#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    match &statements[0] {
        TopLevelStatement::Function(func) => match &func.statements[0] {
            Statement::Expression(Expression::StructInitialisation(struct_init)) => {
                assert_eq!(struct_init.id.name, "Point");
                assert_eq!(struct_init.fields.len(), 2);

                assert_eq!(struct_init.fields[0].name.name, "x");
                assert!(matches!(
                    struct_init.fields[0].value,
                    Expression::Num(Num::Integer(1, (), _))
                ));

                assert_eq!(struct_init.fields[1].name.name, "y");
                assert!(matches!(
                    struct_init.fields[1].value,
                    Expression::Num(Num::Integer(2, (), _))
                ));
            }
            _ => panic!("Expected struct initialization"),
        },
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_function_with_parameters() {
    let code = r#"fn add(a: i32, b: i32): i32 {
        a + b
    }"#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    match &statements[0] {
        TopLevelStatement::Function(func) => {
            assert_eq!(func.id.name, "add");
            assert_eq!(func.parameters.len(), 2);

            assert_eq!(func.parameters[0].name.name, "a");
            assert!(
                matches!(func.parameters[0].type_name, TypeName::Literal(ref name, _) if name == "i32")
            );

            assert_eq!(func.parameters[1].name.name, "b");
            assert!(
                matches!(func.parameters[1].type_name, TypeName::Literal(ref name, _) if name == "i32")
            );

            assert!(matches!(func.return_type, TypeName::Literal(ref name, _) if name == "i32"));
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_while_loop() {
    let code = r#"fn main(): void {
        while (true) {
            break;
        }
    }"#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    match &statements[0] {
        TopLevelStatement::Function(func) => {
            match &func.statements[0] {
                Statement::WhileLoop(while_loop) => {
                    // Should have true as condition and break statement in block
                    assert_eq!(while_loop.block.statements.len(), 1);
                }
                _ => panic!("Expected while loop"),
            }
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_complex_program() {
    let code = r#"
    struct Point {
        x: i32;
        y: i32;
    }
    
    fn distance(p1: Point, p2: Point): i32 {
        let dx: i32 = p1.x - p2.x;
        let dy: i32 = p1.y - p2.y;
        sqrt((dx * dx + dy * dy))
    }
    
    fn main(): void {
        let origin: Point = Point { x: 0, y: 0 };
        let point: Point = Point { x: 3, y: 4 };
        let dist: f64 = distance(origin, point);
        println("Distance: {}", dist);
    }
    "#;

    let program = grammar::parse(code).expect("Failed to parse");
    let statements = program
        .statements
        .into_iter()
        .map(|stmt| TopLevelStatement::transform(stmt, code))
        .collect::<Vec<_>>();

    // Should have 3 top-level statements: struct, function, function
    assert_eq!(statements.len(), 3);

    // First should be struct declaration
    assert!(matches!(
        statements[0],
        TopLevelStatement::StructDeclaration(_)
    ));

    // Second and third should be function declarations
    assert!(matches!(statements[1], TopLevelStatement::Function(_)));
    assert!(matches!(statements[2], TopLevelStatement::Function(_)));

    // Verify specific details
    if let TopLevelStatement::StructDeclaration(struct_decl) = &statements[0] {
        assert_eq!(struct_decl.id.name, "Point");
        assert_eq!(struct_decl.fields.len(), 2);
    }

    if let TopLevelStatement::Function(func) = &statements[1] {
        assert_eq!(func.id.name, "distance");
        assert_eq!(func.parameters.len(), 2);
    }

    if let TopLevelStatement::Function(func) = &statements[2] {
        assert_eq!(func.id.name, "main");
        assert_eq!(func.parameters.len(), 0);
        assert!(func.statements.len() > 0); // Should have multiple statements in body
    }
}
