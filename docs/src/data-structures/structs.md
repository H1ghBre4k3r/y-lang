# Structs

Structs in Y allow you to create custom data types by grouping related fields together. They're fundamental for organizing data and creating meaningful abstractions in your programs.

## Struct Declaration

Define a struct using the `struct` keyword followed by field declarations:

```why
struct Person {
    name: str;
    age: i64;
}

struct Point {
    x: f64;
    y: f64;
}

struct TestStruct {
    x: i64;
    bar: (i64, i64) -> i64;  // Function type as field
}
```

Each field has:
- A name (identifier)
- A type annotation
- A semicolon terminator

## Struct Instantiation

Create struct instances using struct literal syntax:

```why
let person = Person {
    name: "Alice",
    age: 30
};

let origin = Point {
    x: 0.0,
    y: 0.0
};

// Using function references
let my_struct = TestStruct {
    x: 42,
    bar: add  // add is a function
};
```

All fields must be provided during instantiation.

## Field Access

Access struct fields using dot notation:

```why
let person = Person {
    name: "Bob",
    age: 25
};

let name = person.name;  // "Bob"
let age = person.age;    // 25
```

## Mutable Structs

Structs can be mutable, allowing field modification:

```why
let mut person = Person {
    name: "Charlie",
    age: 20
};

person.age = 21;        // Modify age field
person.name = "Chuck";  // Modify name field
```

## Nested Structs

Structs can contain other structs as fields:

```why
struct Address {
    street: str;
    city: str;
}

struct Person {
    name: str;
    address: Address;
}

let person = Person {
    name: "David",
    address: Address {
        street: "123 Main St",
        city: "Springfield"
    }
};

// Access nested fields
let city = person.address.city;
```

## Functions as Struct Fields

Y allows function types as struct fields:

```why
struct Calculator {
    operation: (i64, i64) -> i64;
    name: str;
}

fn add(x: i64, y: i64): i64 {
    x + y
}

let calc = Calculator {
    operation: add,
    name: "Adder"
};

// Call the function through the struct
let result = calc.operation(10, 20);  // 30
```

## Real Examples from Y Code

### Complex Struct Usage

```why
struct FooStruct {
    id: i64;
    amount: f64;
}

struct Bar {
    t: TestStruct;
}

fn main(): void {
    let foo = FooStruct {
        id: 42,
        amount: 133.7
    };

    let mut b = Bar {
        t: TestStruct {
            x: 1337,
            bar: add
        }
    };

    // Nested field access and modification
    b.t.x = 42;

    // Calling function through nested struct
    b.t.bar(4, 2);
}
```

### Assignment Example

```why
struct Foo {
    b: i64;
}

fn b(): i64 {
    17
}

fn main(): void {
    let mut a = Foo {
        b: b()  // Function call in field initialization
    };

    a.b = 42;  // Direct field assignment
}
```

## Methods on Structs

Structs can have methods defined through instance blocks:

```why
struct FooStruct {
    id: i64;
    amount: f64;
}

instance FooStruct {
    fn get_id(): i64 {
        this.id
    }

    fn get_amount(): f64 {
        this.amount
    }

    fn set_amount(new_amount: f64): void {
        this.amount = new_amount;
    }
}

// Usage
let foo = FooStruct {
    id: 42,
    amount: 133.7
};

let id = foo.get_id();        // Method call
let amount = foo.get_amount(); // Another method call
```

## Struct Patterns and Best Practices

### Grouping Related Data

```why
// Good: Related fields grouped together
struct Circle {
    center_x: f64;
    center_y: f64;
    radius: f64;
}

// Better: Using nested structs for better organization
struct Point {
    x: f64;
    y: f64;
}

struct Circle {
    center: Point;
    radius: f64;
}
```

### Using Structs as Parameters

```why
fn calculate_area(circle: Circle): f64 {
    const PI: f64 = 3.14159;
    return PI * circle.radius * circle.radius;
}

fn move_point(point: Point, dx: f64, dy: f64): Point {
    Point {
        x: point.x + dx,
        y: point.y + dy
    }
}
```

### Structs with Arrays

```why
struct Student {
    name: str;
    grades: &[i64];
}

struct Class {
    name: str;
    students: &[Student];
}

let math_class = Class {
    name: "Mathematics",
    students: &[
        Student {
            name: "Alice",
            grades: &[95, 87, 92]
        },
        Student {
            name: "Bob",
            grades: &[88, 79, 94]
        }
    ]
};
```

## Common Patterns

### Builder Pattern

```why
struct Config {
    debug: bool;
    port: i64;
    host: str;
}

fn create_config(): Config {
    Config {
        debug: false,
        port: 8080,
        host: "localhost"
    }
}
```

### Data Transfer Objects

```why
struct UserData {
    username: str;
    email: str;
    created_at: i64;  // timestamp
}

struct Response {
    status: i64;
    data: UserData;
    message: str;
}
```

## Memory and Performance

- Structs are value types - they contain the actual data
- Field access is direct and efficient
- Nested structs are stored inline
- Mutable structs allow in-place modification

## Best Practices

1. **Use descriptive names**: Choose clear field and struct names
2. **Group related data**: Keep related fields together in the same struct
3. **Consider immutability**: Use mutable structs only when necessary
4. **Organize with nesting**: Use nested structs for better data organization
5. **Document complex structures**: Use comments for complex struct relationships

```why
// Good struct design
struct BankAccount {
    account_number: str;
    balance: f64;
    owner: Person;      // Nested struct
    is_active: bool;
}

// Clear field access
let account = create_account();
let balance = account.balance;
let owner_name = account.owner.name;
```