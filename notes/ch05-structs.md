# Ch 5 — Using Structs to Structure Related Data

## What is a Struct

A struct is a custom type — a named collection of related fields. Think of it as a blueprint (no values), instances are the living versions with actual data.

```rust
struct Rectangle {
    width: u32,
    height: u32,
}
```

- **Struct** — the blueprint, just names and types
- **Instance** — a concrete value with actual data
- **Fields** — the named values inside

## Defining and Instantiating

```rust
let rect1 = Rectangle {
    width: 30,
    height: 50,
};
```

- Fields accessed via dot notation: `rect1.width`
- Mutability is all-or-nothing — `let mut rect1` makes every field mutable, you can't mark individual fields

## Field Init Shorthand

When parameter names match field names, you can skip the repetition:

```rust
fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username,  // instead of username: username
        email,
        sign_in_count: 1,
    }
}
```

## Struct Update Syntax

Create a new instance based on an existing one:

```rust
let user2 = User {
    email: String::from("another@example.com"),
    ..user1  // remaining fields from user1
};
```

Uses move semantics — any `String` fields pulled from `user1` are moved into `user2`, partially invalidating `user1`. `Copy` fields (`bool`, `u64`, etc.) are fine. To keep `user1` fully valid, clone it first: `..user1.clone()`.

## Tuple Structs

Named tuples — type distinction without named fields:

```rust
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);
```

`Color` and `Point` are different types even though both hold three `i32`s. Access by index: `black.0`, `black.1`. Useful when the name carries meaning but individual fields don't need labels.

## Unit-Like Structs

No fields at all:

```rust
struct AlwaysEqual;
let subject = AlwaysEqual;
```

Used when you need a type to implement a trait but don't need to store any data. Makes more sense in ch10 with traits.

## Ownership in Structs

Use owned types (`String`) not references (`&str`) in structs by default. Each instance then owns all its data and it lives as long as the struct does. Using references requires lifetimes (ch10).

## Passing Structs to Functions

Pass by reference to avoid losing ownership:

```rust
fn area(rectangle: &Rectangle) -> u32 {
    rectangle.width * rectangle.height
}
```

The parameter name (`rectangle`) and the type (`&Rectangle`) are separate — name is just the local label inside the function.

## Debugging with `#[derive(Debug)]`

Add this attribute to auto-generate debug printing:

```rust
#[derive(Debug)]
struct Rectangle { ... }

println!("{:?}", rect1);   // Rectangle { width: 30, height: 50 }
println!("{:#?}", rect1);  // pretty-printed, one field per line
```

- `{}` — requires `Display` trait (you write it yourself)
- `{:?}` — requires `Debug` trait (auto-derivable)
- `{:#?}` — same, formatted nicely

Good practice to add `#[derive(Debug)]` to most structs by default.

## `dbg!` Macro

Prints to stderr with file name, line number, and value — useful mid-expression:

```rust
let rect1 = Rectangle {
    width: dbg!(30 * scale),  // prints the expression and its result
    height: 50,
};
dbg!(&rect1);
```

Takes ownership (returns it back), unlike `println!` which borrows.

## Methods

Methods are functions attached to a struct, defined in an `impl` block. First parameter is always `self`.

```rust
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

rect1.area() // method syntax
```

Three ways to take `self`:
- `&self` — immutable borrow (read only, most common)
- `&mut self` — mutable borrow (can modify the instance)
- `self` — takes ownership (rare, consumes the instance)

Methods with the same name as a field are common as getters — `rect.width()` calls the method, `rect.width` accesses the field directly. Getters let you make a field private but expose it read-only via a public method. Full public/private visibility covered in ch7.

## Automatic Referencing

Rust auto-adds `&`, `&mut`, or `*` to match a method's signature — no `->` operator like in C/C++:

```rust
rect1.area();       // Rust figures out it needs &self
(&rect1).area();    // equivalent, but you never need to write this
```

## Associated Functions

Functions in `impl` without `self` — called on the type, not an instance. Use `::` syntax:

```rust
impl Rectangle {
    fn square(size: u32) -> Self {
        Self { width: size, height: size }
    }
}

let sq = Rectangle::square(3);
```

`String::from()` is an associated function. Common use: constructors. Rust has no `new` keyword — constructors are just associated functions by convention. Called with `::` on the type since no instance exists yet.

## Multiple `impl` Blocks

A struct can have multiple `impl` blocks — valid, though usually one is enough. Becomes useful with generics and traits (ch10).
