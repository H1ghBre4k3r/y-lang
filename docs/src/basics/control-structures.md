# Control Structures

Y provides some structures to control the logic of your program.

## If-Else

If you want to conditionally execute certain parts of your program, you can utilise the common `if-else` structure:

```why
if (foo) {
    // do some stuff
} else {
    // do other stuff
}
```

The expression right after the `if` needs to evaluate to a boolean value. More on expressions will be discussed in a later chapter. For now, you can imagine simple comparison operations:

```why
if (bar == 42) {
    // ...
}
```

The first block will be executed if the expression evaluates to true. Similarly, the second block will be executed if the expression evaluates to false.

Generally, the else-block is not required, whereas the first block is required (although it can be empty).

## Loops

To repeatedly execute a block of code, Y provides you with loop structures.

### While Loops

To execute a block of code while a certain expression evaluates to true, you can use the `while` loop:

```why
while (foo) {
    // do something
}
```

Again, `foo` has to evaluate to a boolean value. It will be evaluated upon each run of the loop.
