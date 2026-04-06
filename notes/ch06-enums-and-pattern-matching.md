# Ch 6 — Enums and Pattern Matching

## What is an Enum

An enum defines a type that can be one of several possible variants. Where a struct groups related data together, an enum says "this value is one of these options."

Think of it like a mux — exactly one variant is active at a time. `::` selects which variant you're working with.

**Struct vs Enum:** struct is "this AND that" (all fields exist at once). Enum is "this OR that" (exactly one variant active). Fields belong to structs; variants belong to enums.

```rust
enum IpAddrKind {
    V4,
    V6,
}

let four = IpAddrKind::V4;
```

Variants are accessed with `::`. Both variants are the same type (`IpAddrKind`), so you can write a function that accepts either:

```rust
fn route(ip_kind: IpAddrKind) {}
```

## Enums with Data

Variants can hold data directly — no need for a separate struct:

```rust
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

let home = IpAddr::V4(127, 0, 0, 1);
let loopback = IpAddr::V6(String::from("::1"));
```

Each variant can hold different types and amounts of data:

```rust
enum Message {
    Quit,                       // no data
    Move { x: i32, y: i32 },   // named fields like a struct
    Write(String),              // single value
    ChangeColor(i32, i32, i32), // tuple
}
```

## Methods on Enums

Same as structs — use `impl`:

```rust
impl Message {
    fn call(&self) { ... }
}

let m = Message::Write(String::from("hello"));
m.call();
```

## Variants with data

Variants can be plain tags (no data) or carry data of any type:

```rust
enum Coin {
    Dime,               // just a label, no payload
    Quarter(UsState),   // label + a UsState value inside
}
```

The type in the parens is what goes in that slot. When you create one: `Coin::Quarter(UsState::Alaska)`. When you match on it, the pattern extracts it: `Coin::Quarter(state)` binds `state` to the inner value.

## `Option<T>` — Rust's answer to null

Rust has no null. Instead, the standard library provides `Option<T>`:

```rust
enum Option<T> {
    None,
    Some(T),
}
```

`Option` and its variants are in the prelude — use `Some` and `None` directly without `Option::`.

```rust
let some_number = Some(5);        // Option<i32>
let absent: Option<i32> = None;
```

`Option<T>` and `T` are different types — you can't use one where the other is expected. This forces explicit handling of the "maybe nothing" case at compile time. No null pointer surprises.

The big idea: normal Rust types can never be null, so you avoid a whole class of bugs. If something might be nothing, you use `Option<T>` and must explicitly handle both cases — usually with `match`. C/C++ lets you ignore null checks; Rust makes them mandatory at compile time.

`Option<T>` vs `T`: if a function returns `String` it always has one, guaranteed. If it returns `Option<String>` it might not — and you can't use it without handling both cases. Nullability is opt-in and impossible to ignore.

`None` is not `()`. `None` means "no value." `()` is the unit type (like void) — it's a value, just an empty one. Similar concept, different things.

Real use case — a function that might not find what it's looking for:

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 { Some(String::from("Cole")) } else { None }
}
```

`<T>` is a generic placeholder — Rust fills it in from context (`Some(5)` → `Option<i32>`, `Some("x")` → `Option<&str>`). Full generics in ch10.

`Option` is in the prelude alongside `println!`, `Vec`, `String` etc. — no `use` needed.

## `match`

`match` compares a value against patterns and runs the first one that matches. Unlike `if`, it works on any type, not just booleans.

```rust
fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}
```

Arms can have multiple lines with `{}`. The last expression in an arm is its return value.

### Binding values in match

Extract data out of enum variants:

```rust
Coin::Quarter(state) => {
    println!("State quarter from {state:?}!");
    25
}
```

### Matching `Option<T>`

```rust
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}
```

### Exhaustiveness

`match` must cover every possible case — the compiler enforces this. Missing a variant is a compile error.

### Catch-all patterns

```rust
match dice_roll {
    3 => add_fancy_hat(),
    7 => remove_fancy_hat(),
    other => move_player(other), // binds the value
}
```

Or ignore the value entirely with `_`:

```rust
_ => reroll()   // don't care about the value
_ => ()         // do nothing
```

Any name works as a catch-all (`other`, `val`, `x`, etc.) — it's just a variable. `_` is the special case that matches without binding (use when you don't need the value). Catch-all must come last.

`format!` is like `println!` but returns a `String` instead of printing it.

## `if let`

Shorthand for `match` when you only care about one pattern:

```rust
// instead of:
match config_max {
    Some(max) => println!("max is {max}"),
    _ => (),
}

// write:
if let Some(max) = config_max {
    println!("max is {max}");
}
```

Can add `else` for the non-matching case. Trade-off: less verbose but loses exhaustive checking.

## `let...else`

For extracting a value or returning early — keeps the happy path unindented:

```rust
fn describe(coin: Coin) -> Option<String> {
    let Coin::Quarter(state) = coin else {
        return None;
    };
    // state is now in scope here
    Some(format!("{state:?}"))
}
```

If the pattern matches, the value is bound in the outer scope. If it doesn't, the `else` branch must exit (return, break, etc.).

## When to use what

| | Use when |
|---|---|
| `match` | Multiple patterns, need exhaustive handling |
| `if let` | One pattern you care about, don't need exhaustiveness |
| `let...else` | Extract value or bail early |
