# Pesca Lang

> **Attention:** Everything in this project is subject to change! The documentation of the code is nearly non-existent. Furthermore, the currently executable does nothing really useful.

Parsing all the way down.

An innovative parser for an ever more innovative programming language!

Ok, jokes aside: This is just a small project to "restart" my programming language (formerly known as `Y`).

# Language

_Note:_ This language specification is subject to change and far from complete!

Here are my thoughts on the syntax of the language :)

## Expressions

Everything is an expression. Well, almost everything. There are some things which are no expressions, e.g., statements. For statements, see a later section.

A very basic expression is this:

```
17 + 25
```

Even this is an expression:

```
1337
```

Or this:

```
"foo"
```

### Variables

You can assign values to variables:

```
let foo = 42;
```

Aside from these simple expressions, we also have more...complicated expressions.

### Control Flow

To control the flow of your program, you can utilize several control flow constructs.

#### If-expressions

Whoop whoop, the basic foundations of every sane programming language: `if`-`else`

```
if someCondition {
    42
} else {
    1337
}
```

_Note:_ Both arms _must have_ the same return type. If the return type of the `if` arm is `void`, the `else` arm _can_ be omitted.

#### Match

A more advanced version of control flow is the `match` expression. It is like `switch` from other languages. But on steriods.

```
match someValue {
    42 => doSomething()
    1337 => doSomethingElse()
    _ => doSomethingVeryDifferent()
    ^-- this is a wildcard
}
```

You may notice the weird strings with `()` after them - these are function calls. We get to them at a later point.

To make matters even more interesting, you can also bind variables to matches:

```
match x in someFunction(x) {
    42 => doSomethingWhere42Matched()
    1337 => doSomethingWhere1377Matched()
    _ => noneOfTheAboveMatched()
}
```

#### Functions

In this programming language, functions are first class citizens. You can use them as values and hand them to other functions. Here's how you declare a function which adds two integers (we'll get to types later):

```
fn add(x: i32, y: i32): i32 {
    return x + y;
}
```

Additionally, you can define functions as lambdas:

```
let add: (i32, i32) -> i32 = \(x, y) => x + y;
```

Note, how we have to explicitly annotate the type of the function at the variable. Although, it might seem to very verbose, lambdas are very usefull, when passing function as arguments to other functions:

```
fn foo(func: (i32, i32) -> i32): i32 {
    func(42, 1337) * 3
};

let bar = foo(\(x, y) => x + y));
```

Furthermore, you can assign functions to variables:

```
let add = fn (x: i32, y: i32): i32 {
    return x + y;
}
```

... I would not recommend this way. ^^

BUT, you can also assign _existing_ functions to variables (or use them as parameters, etc.):

```
fn add(x: i32, y: i32): i32 {
    return x + y;
}

fn foo(func: (i32, i32) -> i32): i32 {
    func(42, 1337) * 3
};

let test = foo;

let bar = test(add);
```
