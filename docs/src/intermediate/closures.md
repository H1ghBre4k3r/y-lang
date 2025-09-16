# Closures and Variable Capture

Closures in Y allow lambda functions to "capture" variables from their surrounding scope, creating powerful functional programming patterns. This enables functions to carry their environment with them, making them truly first-class citizens.

## What are Closures?

A closure is a lambda function that captures (remembers) variables from the scope where it was created. Unlike regular lambdas that only have access to their parameters, closures can access and use variables from their outer scope.

```why
fn createAdder(x: i64): (i64) -> i64 {
    // This lambda captures the variable 'x'
    \(y) => x + y
}

fn main(): i64 {
    let addFive = createAdder(5);
    addFive(10)  // Returns 15
}
```

## Variable Capture Mechanics

### Capture by Value

Y captures variables **by value** at the time the closure is created. This means:

1. The captured value is copied into the closure's environment
2. Changes to the original variable after closure creation don't affect the closure
3. The closure maintains its own copy of the captured values

```why
fn captureExample(): (i64) -> i64 {
    let x = 10;
    let closure = \(y) => x + y;

    // Even if we could modify x here, it wouldn't affect the closure
    // because x was captured by value when closure was created

    closure  // Returns a function that adds 10 to its argument
}
```

### What Can Be Captured

Closures can capture:

- **Simple variables**: integers, floats, booleans, characters
- **Struct fields**: accessing fields of captured structs
- **Complex types**: strings, arrays, other functions

```why
struct Point {
    x: i64;
    y: i64;
}

fn createTranslator(offset: Point): (Point) -> Point {
    \(p) => Point {
        x: p.x + offset.x,
        y: p.y + offset.y
    }
}

fn main(): i64 {
    let translator = createTranslator(Point { x: 5, y: 3 });
    let result = translator(Point { x: 1, y: 2 });
    result.x + result.y  // Returns 11 (6 + 5)
}
```

## Common Patterns

### Factory Functions

Closures are excellent for creating specialized functions:

```why
fn createMultiplier(factor: i64): (i64) -> i64 {
    \(n) => n * factor
}

fn main(): i64 {
    let double = createMultiplier(2);
    let triple = createMultiplier(3);

    double(5) + triple(4)  // Returns 22 (10 + 12)
}
```

### Configuration Capture

Capture configuration or settings for later use:

```why
struct Config {
    baseUrl: i64;  // Simplified for example
    timeout: i64;
}

fn createApiCall(config: Config): (i64) -> i64 {
    \(endpoint) => config.baseUrl + endpoint + config.timeout
}
```

### Partial Application

Create partially applied functions by capturing some arguments:

```why
fn add3(a: i64, b: i64, c: i64): i64 {
    a + b + c
}

fn partialAdd(a: i64, b: i64): (i64) -> i64 {
    \(c) => add3(a, b, c)
}

fn main(): i64 {
    let addToTen = partialAdd(3, 7);  // Captures a=3, b=7
    addToTen(5)  // Returns 15 (3 + 7 + 5)
}
```

## Advanced Examples

### Multiple Variable Capture

Closures can capture multiple variables from different scopes:

```why
fn complexCapture(x: i64): ((i64) -> i64) {
    let y = x * 2;
    let z = 5;

    \(w) => x + y + z + w  // Captures x, y, and z
}

fn main(): i64 {
    let func = complexCapture(3);  // x=3, y=6, z=5
    func(2)  // Returns 16 (3 + 6 + 5 + 2)
}
```

### Nested Closures

Closures can be nested and each level can capture variables:

```why
fn createDoubleAdder(x: i64): ((i64) -> (i64) -> i64) {
    \(y) => \(z) => x + y + z
}

fn main(): i64 {
    let adder = createDoubleAdder(1);  // Captures x=1
    let nextAdder = adder(2);          // Captures x=1, y=2
    nextAdder(3)                       // Returns 6 (1 + 2 + 3)
}
```

## Performance Considerations

### Memory Allocation

- Capturing closures allocate memory on the heap to store captured variables
- Non-capturing lambdas are more efficient as they don't need environment allocation
- Each closure instance gets its own copy of captured variables

### When to Use Closures

**Good uses:**
- Factory functions that create specialized behavior
- Event handlers that need context
- Functional programming patterns like map/filter (when available)
- Configuration or state encapsulation

**Consider alternatives when:**
- No variables need to be captured (use regular lambdas)
- Performance is critical and the capture overhead matters
- The lifetime of captured data is unclear

## Best Practices

1. **Minimize Captures**: Only capture what you actually need
2. **Be Aware of Copying**: Remember that large structs are copied entirely
3. **Use Clear Names**: Make it obvious what variables are being captured
4. **Consider Lifetime**: Ensure captured values remain valid for the closure's lifetime

```why
// Good: Minimal, clear capture
fn createValidator(minLength: i64): (i64) -> bool {
    \(length) => length >= minLength
}

// Less ideal: Captures unnecessary data
struct ValidationConfig {
    minLength: i64;
    maxLength: i64;
    allowEmpty: bool;
    errorMessage: String;
}

fn createValidatorComplex(config: ValidationConfig): (i64) -> bool {
    // Captures entire config even if we only need minLength
    \(length) => length >= config.minLength
}
```

## Troubleshooting

### Common Issues

1. **Unexpected Values**: Remember capture happens at creation time, not call time
2. **Memory Usage**: Large captured structs can impact performance
3. **Scope Confusion**: Variables are captured from where the lambda is defined

### Debugging Tips

- Use simple examples to understand capture behavior
- Print captured values to verify they're what you expect
- Consider the timing of when captures occur vs when the closure is called

---

Closures provide powerful abstraction capabilities while maintaining the performance characteristics suitable for systems programming. Use them to create clean, reusable code that encapsulates behavior with its necessary context.