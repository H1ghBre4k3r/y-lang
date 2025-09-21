# Grammar Reference

This page provides the complete formal grammar for the Y programming language. The grammar is defined using a BNF-like notation and corresponds to the actual parser implementation.

## Program Structure

### Top-Level Program

```
Program := ToplevelStatement*

ToplevelStatement := FunctionDeclaration
                   | Constant
                   | Declaration
                   | StructDeclaration
                   | Instance
                   | Comment
```

### Statements

```
Statement := FunctionDeclaration
           | VariableDeclaration
           | Assignment
           | WhileStatement
           | Constant
           | Expression ";"
           | YieldingExpression
           | Return
           | Declaration
           | StructDeclaration
           | Comment

Return := "return" Expression ";"
```

## Declarations

### Function Declaration

```
FunctionDeclaration := "fn" Identifier "(" ParameterList? ")" ":" TypeName Block

ParameterList := Parameter ("," Parameter)*
Parameter := Identifier ":" TypeName
```

### Variable Declaration

```
VariableDeclaration := "let" "mut"? Identifier (":" TypeName)? "=" Expression ";"
```

### Struct Declaration

```
StructDeclaration := "struct" Identifier "{" StructFieldDeclaration* "}"

StructFieldDeclaration := Identifier ":" TypeName ";"
```

### Constant Declaration

```
Constant := "const" Identifier ":" TypeName "=" Expression ";"
```

### External Declaration

```
Declaration := "declare" Identifier ":" TypeName ";"
```

### Instance Declaration

```
Instance := "instance" TypeName "{" InstanceMember* "}"

InstanceMember := MethodDeclaration
                | MethodDeclare

MethodDeclaration := "fn" Identifier "(" ParameterList? ")" ":" TypeName Block
MethodDeclare := "declare" Identifier "(" ParameterList? ")" ":" TypeName ";"
```

## Expressions

### Expression Types

```
Expression := Boolean
            | Identifier
            | Number
            | String
            | Character
            | IfExpression
            | ParenthesizedExpression
            | BinaryExpression
            | Block
            | Lambda
            | Postfix
            | Prefix
            | Array
            | StructInitialisation

YieldingExpression := Expression  // Expression without semicolon terminator
```

### Literals

```
Boolean := "true" | "false"

Number := Integer | Floating
Integer := [0-9]+
Floating := [0-9]+ "." [0-9]+

String := '"' ([^"\\] | \\.)* '"'
Character := "'" ([^'\\] | \\.) "'"

Identifier := [_a-zA-Z][_a-zA-Z0-9]*
```

### Binary Expressions

```
BinaryExpression := Expression BinaryOperator Expression

BinaryOperator := "+"     // Addition (precedence 1, left-associative)
                | "-"     // Subtraction (precedence 1, left-associative)
                | "*"     // Multiplication (precedence 2, left-associative)
                | "/"     // Division (precedence 2, left-associative)
                | "=="    // Equality (precedence 0, left-associative)
                | "!="    // Inequality (precedence 0, left-associative)
                | "<"     // Less than (precedence 0, left-associative)
                | ">"     // Greater than (precedence 0, left-associative)
                | "<="    // Less or equal (precedence 0, left-associative)
                | ">="    // Greater or equal (precedence 0, left-associative)
```

### Control Flow Expressions

```
IfExpression := "if" "(" Expression ")" Block ("else" Block)?

WhileStatement := "while" "(" Expression ")" Block

Block := "{" Statement* "}"
```

### Function and Lambda Expressions

```
Lambda := "\" "(" ParameterList? ")" "=>" Expression

Postfix := Expression PostfixOperator

PostfixOperator := FunctionCall
                 | PropertyAccess
                 | IndexExpression

FunctionCall := "(" ArgumentList? ")"
ArgumentList := Expression ("," Expression)*

PropertyAccess := "." Identifier
IndexExpression := "[" Expression "]"
```

### Data Structure Expressions

```
Array := "&" "[" (Expression ("," Expression)*)? "]"

StructInitialisation := Identifier "{" FieldInitList? "}"
FieldInitList := FieldInit ("," FieldInit)*
FieldInit := Identifier ":" Expression

ParenthesizedExpression := "(" Expression ")"
```

## Assignment and L-Values

```
Assignment := LValue "=" Expression ";"

LValue := Identifier
        | PropertyAccess
        | IndexExpression

PropertyAccess := Expression "." Identifier
IndexExpression := Expression "[" Expression "]"
```

## Type System

### Type Names

```
TypeName := PrimitiveType
          | ArrayType
          | FunctionType
          | UserType

PrimitiveType := "i64" | "u32" | "f64" | "bool" | "char" | "str" | "void"

ArrayType := "&" "[" TypeName "]"

FunctionType := "(" (TypeName ("," TypeName)*)? ")" "->" TypeName

UserType := Identifier  // User-defined struct types
```

### Type Annotations

```
TypeAnnotation := ":" TypeName
```

## Comments

```
Comment := "//" [^\n]*
```

## Operator Precedence

From highest to lowest precedence:

1. **Postfix operators** (function calls, property access, array indexing)
2. **Prefix operators** (unary operations)
3. **Multiplicative** (`*`, `/`) - precedence 2, left-associative
4. **Additive** (`+`, `-`) - precedence 1, left-associative
5. **Comparison** (`==`, `!=`, `<`, `>`, `<=`, `>=`) - precedence 0, left-associative

## Complete Grammar Examples

### Function with All Features

```why
// Function declaration with parameters and return type
fn complex_function(
    param1: i64,
    param2: &[str],
    callback: (str) -> bool
): &[str] {
    let mut results: &[str] = &[];
    let mut i = 0;

    while (i < param2.length()) {
        let current = param2[i];
        if (callback(current)) {
            results = append(results, current);
        }
        i = i + 1;
    }

    return results;
}
```

### Struct with Instance Methods

```why
struct ComplexStruct {
    id: i64;
    name: str;
    values: &[f64];
    processor: (f64) -> f64;
}

instance ComplexStruct {
    fn get_id(): i64 {
        this.id
    }

    fn process_values(): &[f64] {
        let mut results: &[f64] = &[];
        let mut i = 0;

        while (i < this.values.length()) {
            let processed = this.processor(this.values[i]);
            results = append(results, processed);
            i = i + 1;
        }

        return results;
    }

    declare external_method(str): bool;
}
```

### Complex Expression

```why
fn expression_example(): i64 {
    let result = if (condition1 && condition2) {
        calculate_value(
            array[index],
            \(x) => x * 2 + offset,
            struct_instance.method_call()
        )
    } else {
        default_value
    };

    return result;
}
```

## Grammar Notes

### Expression vs Statement Context

- **Expressions** evaluate to values and can be used in expression contexts
- **Statements** perform actions and include variable declarations, assignments, and control flow
- **Yielding expressions** are expressions that serve as the final value of a block (no semicolon)

### Semicolon Rules

- Most statements end with semicolons
- Expression statements end with semicolons
- The last expression in a block can omit the semicolon (yielding expression)
- Function bodies, if-else bodies, and while bodies use blocks

### Type Inference

- Variable types can often be inferred from the assigned expression
- Function parameters and return types must be explicitly declared
- Empty arrays require explicit type annotations

This grammar reference corresponds to the actual implementation in the Y compiler and can be used to understand the valid syntax for all language constructs.