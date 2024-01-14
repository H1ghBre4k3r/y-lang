# Structs

Structs are a way to group and organize related data together, introducing a new [type](../basics/types.md) to your program. Declaring a struct is straight forward:

```why
struct Vector {
    x: i32,
    y: i32
};
```

Doing that introduces a new type to your program which can be used in any location where you are able to declare a type, e.g., function arguments or return types:

```
struct Foo {
     // ..
};

struct Bar {
    // ..
};

fn doSomething(input: Foo): Bar {
    // ...
}
```

As you can see, you can just use the name of the struct as the type name.

## Instantiation

If you want to instantiate a struct, you can simply do that straight forward:

```why
struct Vector {
    x: i32,
    y: i32
};

let someVec = Vector {
    x: 42,
    y: 1337,
};
```

## Property Access

To access properties of a struct, you can utilise the dot operator:

```why
struct Vector {
    x: i32,
    y: i32
};

let someVec = Vector {
    x: 42,
    y: 1337,
};

let xDirection = someVec.x;
```
