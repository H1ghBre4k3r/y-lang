# Variables

One of the building blocks of each Y program are variables. Variables are used to store values during the runtime of the program. Declaring and instantiating a variable is straight forward:

```why
let foo = 42;
```

This snippet creates a variable with the name `foo` and the value `42`. As you can see, a variable declaration & instantiation always consists of at least these three components:

- the `let` keyword
- an identifier for the variable
- (technically `=` to perform an assignment)
- a value to assign to this variable

## Mutability

By default, variables in Y are immutable. That means, once you assigned a value to a variable, you can not re-assign this variable. E.g., the following is invalid Y code:

```why
let foo = 42;
foo = 1337; // <- invalid
```

If you want to re-assign a variable, you first need to declare this variable as mutable using the `mut` keyword:

```why
let mut foo = 42;
foo = 1337;
```

This allows you to mutate variables at your own will. However, it is discouraged to just declare every variable as mutable, since this might introduce unwanted bugs to your program.
