# Methods and Instances

Y supports object-oriented programming concepts through instance blocks, which allow you to define methods associated with specific types. This enables you to extend both built-in types and custom structs with additional functionality.

## Instance Blocks

Instance blocks define methods for a specific type using the `instance` keyword:

```why
instance TypeName {
    fn method_name(parameters): ReturnType {
        // method implementation
    }
}
```

## Methods on Custom Structs

### Basic Method Definition

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
```

### Using the `this` Keyword

The `this` keyword refers to the current instance:

```why
struct TestStruct {
    x: i64;
    bar: (i64, i64) -> i64;
}

instance TestStruct {
    fn get_x(): i64 {
        return this.x;  // Access field through this
    }

    fn set_x(x: i64): void {
        this.x = x;     // Modify field through this
    }

    fn double_x(): i64 {
        this.x * 2     // Use field in computation
    }
}
```

## Method Calls

Call methods using dot notation:

```why
let foo = FooStruct {
    id: 42,
    amount: 133.7
};

let id = foo.get_id();        // 42
let amount = foo.get_amount(); // 133.7

foo.set_amount(200.0);
let new_amount = foo.get_amount(); // 200.0
```

## External Method Declarations

You can declare methods that are implemented externally:

```why
instance TestStruct {
    fn get_x(): i64 {
        return this.x;
    }

    declare get_id(): i64;  // External implementation
}

instance str {
    declare len(): i64;     // String length function
}
```

## Methods on Built-in Types

Y allows extending built-in types with custom methods:

```why
instance i64 {
    declare add(i64): i64;    // Custom addition
    declare multiply(i64): i64;
}

instance str {
    declare len(): i64;       // String length
    declare to_upper(): str;  // Convert to uppercase
}

// Usage
let text = "hello";
let length = text.len();      // Call method on string
```

## Complex Method Examples

### Methods with Business Logic

```why
struct BankAccount {
    balance: f64;
    account_number: str;
    is_active: bool;
}

instance BankAccount {
    fn deposit(amount: f64): void {
        if (amount > 0.0) {
            this.balance = this.balance + amount;
        }
    }

    fn withdraw(amount: f64): bool {
        if (amount > 0.0 && amount <= this.balance && this.is_active) {
            this.balance = this.balance - amount;
            return true;
        } else {
            return false;
        }
    }

    fn get_balance(): f64 {
        if (this.is_active) {
            this.balance
        } else {
            0.0
        }
    }
}
```

### Methods Using Other Methods

```why
struct Point {
    x: f64;
    y: f64;
}

instance Point {
    fn distance_from_origin(): f64 {
        sqrt(this.x * this.x + this.y * this.y)
    }

    fn distance_from(other: Point): f64 {
        let dx = this.x - other.x;
        let dy = this.y - other.y;
        sqrt(dx * dx + dy * dy)
    }

    fn move_by(dx: f64, dy: f64): void {
        this.x = this.x + dx;
        this.y = this.y + dy;
    }

    fn normalize(): void {
        let distance = this.distance_from_origin();
        if (distance > 0.0) {
            this.x = this.x / distance;
            this.y = this.y / distance;
        }
    }
}
```

## System Integration

Y supports system-level method declarations:

```why
struct System {}

instance System {
    fn answer(): i64 {
        42
    }

    declare print(i64): void;   // External print function
}

declare Sys: System;  // Global system instance

fn main(): void {
    Sys.print(Sys.answer());   // Call system methods
}
```

## Real Examples from Y Code

### Struct with Methods and External Declarations

```why
struct FooStruct {
    id: i64;
}

instance FooStruct {
    fn get_id(): i64 {
        this.id
    }
}

instance i64 {
    declare add(i64): i64;
}

struct System {}

instance System {
    fn answer(): i64 {
        42
    }

    declare print(i64): void;
}

declare Sys: System;

fn main(): void {
    Sys.print(Sys.answer());
}
```

### Complete Workflow Example

```why
fn main(): i64 {
    let my_struct = TestStruct {
        x: 42,
        bar: add
    };

    // Calling struct methods
    let value_of_x = my_struct.get_x();   // Get current value
    my_struct.set_x(1337);                // Set new value
    let new_value = my_struct.get_x();    // Get updated value

    // Using external method
    let id = my_struct.get_id();

    return 0;
}
```

## Method Chaining

Methods can be designed for chaining (though return types must support it):

```why
struct Builder {
    value: i64;
}

instance Builder {
    fn set_value(val: i64): Builder {
        this.value = val;
        return this;  // Return self for chaining
    }

    fn add(val: i64): Builder {
        this.value = this.value + val;
        return this;
    }

    fn build(): i64 {
        this.value
    }
}

// Usage (conceptual - depends on Y's ownership model)
let result = Builder { value: 0 }
    .set_value(10)
    .add(5)
    .build();  // 15
```

## Best Practices

### Method Organization

```why
struct User {
    name: str;
    email: str;
    age: i64;
    is_active: bool;
}

instance User {
    // Getters
    fn get_name(): str { this.name }
    fn get_email(): str { this.email }
    fn get_age(): i64 { this.age }

    // Setters (with validation)
    fn set_age(new_age: i64): bool {
        if (new_age >= 0 && new_age <= 150) {
            this.age = new_age;
            return true;
        } else {
            return false;
        }
    }

    // Business logic
    fn is_adult(): bool {
        this.age >= 18
    }

    fn deactivate(): void {
        this.is_active = false;
    }
}
```

### Method Naming

```why
// Good: Clear, descriptive method names
instance BankAccount {
    fn get_balance(): f64 { ... }
    fn deposit(amount: f64): void { ... }
    fn is_account_active(): bool { ... }
    fn calculate_interest(rate: f64): f64 { ... }
}

// Less ideal: Unclear names
instance BankAccount {
    fn bal(): f64 { ... }
    fn add(amount: f64): void { ... }
    fn check(): bool { ... }
    fn calc(rate: f64): f64 { ... }
}
```

### Error Handling in Methods

```why
instance Calculator {
    fn divide(a: f64, b: f64): f64 {
        if (b == 0.0) {
            return 0.0;  // Or appropriate error handling
        } else {
            return a / b;
        }
    }

    fn safe_access_array(arr: &[i64], index: i64): i64 {
        // Bounds checking would go here
        return arr[index];
    }
}
```

### Encapsulation

```why
struct Counter {
    value: i64;
    max_value: i64;
}

instance Counter {
    // Public interface
    fn increment(): bool {
        if (this.can_increment()) {
            this.value = this.value + 1;
            return true;
        } else {
            return false;
        }
    }

    fn get_value(): i64 {
        this.value
    }

    // Helper method (conceptually private)
    fn can_increment(): bool {
        this.value < this.max_value
    }
}
```

## Integration with Functions

Methods work seamlessly with regular functions:

```why
fn process_user(user: User): void {
    if (user.is_adult()) {           // Method call
        let name = user.get_name();  // Another method call
        printf(name);                // Function call
    }
}

fn create_and_setup_user(): User {
    let mut user = User {
        name: "Alice",
        email: "alice@example.com",
        age: 25,
        is_active: true
    };

    user.set_age(26);  // Method call
    return user;       // Return the modified struct
}
```