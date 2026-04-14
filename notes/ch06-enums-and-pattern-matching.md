# Ch 6 — Enums and Pattern Matching

---

## What is an Enum

An enum defines a type that can be one of several possible variants. Where a struct groups related data together ("this AND that"), an enum says "this OR that" — exactly one variant is active at a time.

```rust
enum IpAddrKind {
    V4,
    V6,
}

let four = IpAddrKind::V4;
let six  = IpAddrKind::V6;
```

Variants are namespaced with `::`. Both variants are the same type (`IpAddrKind`), so one function handles either:

```rust
fn route(ip_kind: IpAddrKind) {}

route(IpAddrKind::V4);
route(IpAddrKind::V6);
```

---

## Enums with Data

Variants can hold data directly — no separate struct needed:

```rust
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

let home    = IpAddr::V4(127, 0, 0, 1);
let loopback = IpAddr::V6(String::from("::1"));
```

Each variant can hold different types and amounts of data:

```rust
enum Message {
    Quit,                       // no data
    Move { x: i32, y: i32 },   // named fields (like a struct)
    Write(String),              // single value
    ChangeColor(i32, i32, i32), // tuple of values
}
```

---

## Methods on Enums

Same as structs — use `impl`:

```rust
impl Message {
    fn call(&self) {
        // method body
    }
}

let m = Message::Write(String::from("hello"));
m.call();
```

---

## `Option<T>` — Rust's Answer to Null

Rust has no null. Instead, the standard library provides `Option<T>`:

```rust
enum Option<T> {
    None,
    Some(T),
}
```

`Option` and its variants are in the prelude — use `Some` and `None` directly, no `use` needed.

```rust
let some_number = Some(5);         // Option<i32>, inferred
let some_char   = Some('e');       // Option<char>, inferred
let absent: Option<i32> = None;    // type annotation required for None
```

`Option<T>` and `T` are different types — the compiler won't let you use one where the other is expected:

```rust
let x: i8 = 5;
let y: Option<i8> = Some(5);

let sum = x + y; // compile error: cannot add `Option<i8>` to `i8`
```

This forces explicit handling of the "maybe nothing" case. If a function returns `String`, it always has one. If it returns `Option<String>`, you must handle both cases. Nullability is opt-in and impossible to silently ignore.

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 { Some(String::from("Cole")) } else { None }
}
```

`<T>` is a generic placeholder — Rust fills it in from context. Full generics in ch10.

`None` is not `()`. `None` means "no value." `()` is the unit type — it's a value, just an empty one.

---

## `match`

`match` compares a value against a series of patterns and runs the first one that matches. Works on any type — not just booleans like `if`.

```rust
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny   => 1,
        Coin::Nickel  => 5,
        Coin::Dime    => 10,
        Coin::Quarter => 25,
    }
}
```

Arms can run multiple lines with `{}` — the last expression is the arm's value:

```rust
Coin::Penny => {
    println!("Lucky penny!");
    1
}
```

### Binding Values in Match

Extract data out of enum variants in the pattern:

```rust
#[derive(Debug)]
enum UsState { Alabama, Alaska, /* ... */ }

enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(UsState),
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny          => 1,
        Coin::Nickel         => 5,
        Coin::Dime           => 10,
        Coin::Quarter(state) => {
            println!("State quarter from {state:?}!");
            25
        }
    }
}

value_in_cents(Coin::Quarter(UsState::Alaska));
// prints: State quarter from Alaska!
```

### Matching `Option<T>`

```rust
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None    => None,
        Some(i) => Some(i + 1),
    }
}

let five = Some(5);
let six  = plus_one(five);  // Some(6)
let none = plus_one(None);  // None
```

`Some(i)` binds the inner value to `i`. `None` matches the empty case and returns `None`.

### Exhaustiveness

`match` must cover every possible variant — the compiler enforces this:

```rust
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        Some(i) => Some(i + 1),
        // compile error: non-exhaustive patterns: `None` not covered
    }
}
```

This is one of Rust's most valuable features — you can't forget a case.

### Catch-All Patterns

Use a variable name to catch remaining values and bind them:

```rust
let dice_roll = 9;

match dice_roll {
    3 => add_fancy_hat(),
    7 => remove_fancy_hat(),
    other => move_player(other), // binds the value to `other`
}
```

Use `_` when you don't need the value:

```rust
match dice_roll {
    3 => add_fancy_hat(),
    7 => remove_fancy_hat(),
    _ => reroll(),  // match without binding
}
```

Use `_ => ()` to explicitly do nothing:

```rust
match dice_roll {
    3 => add_fancy_hat(),
    7 => remove_fancy_hat(),
    _ => (),  // unit value — do nothing
}
```

Catch-all must come last — patterns are evaluated in order.

---

## `if let`

Shorthand for `match` when you only care about one pattern:

```rust
// verbose match:
match config_max {
    Some(max) => println!("max is {max}"),
    _ => (),
}

// concise if let:
if let Some(max) = config_max {
    println!("max is {max}");
}
```

Add `else` for the non-matching case:

```rust
let mut count = 0;

if let Coin::Quarter(state) = coin {
    println!("State quarter from {state:?}!");
} else {
    count += 1;
}
```

Trade-off: less boilerplate, but you lose the exhaustive checking that `match` enforces.

---

## `let...else`

For extracting a value or returning early — keeps the happy path unindented:

```rust
fn describe(coin: Coin) -> Option<String> {
    let Coin::Quarter(state) = coin else {
        return None;  // must exit — return, break, continue, or panic
    };

    // state is bound in the outer scope here
    Some(format!("{state:?}"))
}
```

If the pattern matches, the value is bound in the outer scope and execution continues. If it doesn't, the `else` block must diverge (return early, break, etc.).

---

## When to Use What

| Construct | Use when |
|---|---|
| `match` | Multiple patterns, need exhaustive handling |
| `if let` | Only care about one pattern, exhaustiveness not needed |
| `let...else` | Extract a value or bail out early |
