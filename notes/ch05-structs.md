# Ch 5 — Using Structs to Structure Related Data

A struct is a custom type — a named collection of related fields. Struct = blueprint (just names and types). Instance = the living version with actual data.

---

## Defining and Instantiating

```rust
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

let user1 = User {
    active: true,
    username: String::from("someusername123"),
    email: String::from("someone@example.com"),
    sign_in_count: 1,
};

println!("{}", user1.email); // dot notation
```

Mutability is all-or-nothing — `let mut user1` makes every field mutable. You can't mark individual fields mutable.

```rust
let mut user1 = User { ... };
user1.email = String::from("newemail@example.com"); // ok with mut
```

---

## Field Init Shorthand

When a parameter name matches a field name, skip the repetition:

```rust
fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username,  // shorthand for username: username
        email,     // shorthand for email: email
        sign_in_count: 1,
    }
}
```

---

## Struct Update Syntax

Create a new instance based on an existing one with `..`:

```rust
let user2 = User {
    email: String::from("another@example.com"),
    ..user1  // fill remaining fields from user1
};
```

Uses move semantics — any `String` fields pulled from `user1` are moved into `user2`, partially invalidating `user1`. `Copy` fields (`bool`, `u64`, etc.) are copied, not moved. To keep `user1` fully valid, clone it first: `..user1.clone()`.

---

## Tuple Structs

Named tuples — type distinction without named fields:

```rust
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

let black = Color(0, 0, 0);
let origin = Point(0, 0, 0);
```

`Color` and `Point` are different types even though both hold three `i32`s — a function that takes `Color` won't accept `Point`. Access by index: `black.0`, `black.1`. Useful when the name carries meaning but individual fields don't need labels.

---

## Unit-Like Structs

No fields at all:

```rust
struct AlwaysEqual;
let subject = AlwaysEqual;
```

Used when you need a type to implement a trait but don't need to store any data. Makes more sense once you've seen traits (ch10).

---

## Ownership in Structs

Use owned types (`String`) not references (`&str`) by default. Each instance then owns all its data and it lives as long as the struct does. Using references requires lifetimes (ch10).

```rust
struct User {
    username: &str, // compile error: missing lifetime specifier
}
```

---

## Why Structs: The Rectangle Example

Starting with separate variables:

```rust
fn area(width: u32, height: u32) -> u32 {
    width * height
}
```

The signature doesn't tell you the two values are related. Refactoring with a tuple groups them but loses meaning:

```rust
fn area(dimensions: (u32, u32)) -> u32 {
    dimensions.0 * dimensions.1  // which is which?
}
```

Struct version is clear:

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

fn area(rectangle: &Rectangle) -> u32 {
    rectangle.width * rectangle.height
}

fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    println!("Area: {}", area(&rect1));
}
```

Pass by reference to avoid losing ownership — the function uses the data without consuming it.

---

## Debugging with `#[derive(Debug)]`

Structs don't implement `Display` by default — there's no obvious format. Derive `Debug` to get an auto-generated one:

```rust
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

let rect1 = Rectangle { width: 30, height: 50 };

println!("{:?}", rect1);   // Rectangle { width: 30, height: 50 }
println!("{:#?}", rect1);  // pretty-printed, one field per line
```

- `{}` — requires `Display` (you implement it yourself)
- `{:?}` — requires `Debug` (auto-derivable)
- `{:#?}` — same, formatted nicely

Good habit: add `#[derive(Debug)]` to most structs by default.

---

## `dbg!` Macro

Prints to stderr with file name, line number, and value — useful mid-expression:

```rust
let scale = 2;
let rect1 = Rectangle {
    width: dbg!(30 * scale),  // prints: [src/main.rs:10] 30 * scale = 60
    height: 50,
};

dbg!(&rect1);
// [src/main.rs:14] &rect1 = Rectangle {
//     width: 60,
//     height: 50,
// }
```

Takes ownership and returns the value (pass `&rect1` to avoid moving it). Unlike `println!` which borrows. Prints to `stderr` not `stdout`.

---

## Methods

Methods are functions attached to a struct, defined in an `impl` block. First parameter is always `self`.

```rust
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

rect1.area() // method call syntax
```

Three ways to take `self`:
- `&self` — immutable borrow, read-only, most common
- `&mut self` — mutable borrow, can modify the instance
- `self` — takes ownership, consumes the instance (rare)

`&self` is shorthand for `self: &Self`, where `Self` is an alias for the type the `impl` block is for.

### Methods with Parameters

```rust
impl Rectangle {
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

let rect1 = Rectangle { width: 30, height: 50 };
let rect2 = Rectangle { width: 10, height: 40 };
let rect3 = Rectangle { width: 60, height: 45 };

println!("{}", rect1.can_hold(&rect2)); // true
println!("{}", rect1.can_hold(&rect3)); // false
```

### Methods Named After Fields (Getters)

```rust
impl Rectangle {
    fn width(&self) -> bool {
        self.width > 0
    }
}

rect1.width()  // calls the method
rect1.width    // accesses the field
```

Getters let you make a field private but expose a read-only view via a public method. Visibility covered in ch7.

---

## Automatic Referencing

Rust auto-adds `&`, `&mut`, or `*` to match a method's `self` signature — no `->` operator like in C/C++:

```rust
rect1.area();      // Rust sees &self, automatically borrows rect1
(&rect1).area();   // equivalent, but you never need to write this
```

---

## Associated Functions

Functions in `impl` without `self` — not methods, just associated with the type. Call with `::` syntax:

```rust
impl Rectangle {
    fn square(size: u32) -> Self {
        Self { width: size, height: size }
    }
}

let sq = Rectangle::square(3);
```

`String::from()` is an associated function. Common use: constructors. Rust has no `new` keyword — constructors are just associated functions by convention, often named `new`.

---

## Multiple `impl` Blocks

Valid — one struct can have multiple `impl` blocks. Usually one is enough; becomes useful with generics and traits (ch10).

```rust
impl Rectangle {
    fn area(&self) -> u32 { self.width * self.height }
}

impl Rectangle {
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}
```
