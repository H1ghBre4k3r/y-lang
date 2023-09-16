# Pesca Parser

Parsing all the way down. 

An innovative parser for an ever more innovative programming language!

Ok, jokes aside: This is just a small project to "restart" my programming language (formerly known as `Y`).

# Language

*Note:* This language specification is subject to change and far from complete! 

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

*Note:* Both arms _must have_ the same return type. If the return type of the `if` arm is `void`, the `else` arm _can_ be omitted.

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

To make matters even more interesting, you can also bind existing variables to matches: 

```
match x in someFunction(x) {
    42 => doSomethingWhere42Matched()
    1337 => doSomethingWhere1377Matched()
    _ => noneOfTheAboveMatched()
}
```
