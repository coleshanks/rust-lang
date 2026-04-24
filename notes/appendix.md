# Appendix — Reference Material

Quick-reference for keywords, operators, derivable traits, dev tools, editions, and how Rust's release process works.

---

## Appendix A — Keywords

### Currently in use

| Keyword | What it does |
|---|---|
| `fn` | Define a function or function pointer type |
| `let` | Bind a variable |
| `mut` | Mark a variable or reference as mutable |
| `const` | Compile-time constant — no fixed memory address, inlined everywhere |
| `static` | Global variable with a fixed memory address |
| `ref` | Bind by reference in a pattern |
| `move` | Force a closure to take ownership of captured values |
| `return` | Return early from a function |
| `if` / `else` | Conditional branching |
| `match` | Pattern matching |
| `loop` | Infinite loop |
| `while` | Condition-based loop |
| `for` | Iterator-based loop |
| `break` | Exit a loop (can return a value from `loop`) |
| `continue` | Skip to the next iteration |
| `struct` | Define a struct |
| `enum` | Define an enum |
| `union` | Define a union (like a struct, one field active at a time) |
| `trait` | Define a trait |
| `impl` | Implement a trait or add methods to a type |
| `type` | Type alias or associated type |
| `pub` | Make an item publicly visible |
| `mod` | Define or declare a module |
| `use` | Bring a path into scope |
| `crate` | Refers to the current crate root |
| `super` | Refers to the parent module |
| `self` / `Self` | The current instance / the current type |
| `as` | Cast a type, or disambiguate a trait method |
| `where` | Add trait bounds in a separate clause |
| `dyn` | Dynamic dispatch — `Box<dyn Trait>` |
| `unsafe` | Opt into unsafe superpowers |
| `extern` | Link external code (FFI) |
| `async` / `await` | Async programming |
| `in` | Part of `for x in ...` syntax |
| `true` / `false` | Boolean literals |

### Reserved for future use

These aren't valid today but are reserved so Rust can use them later without breaking compatibility:

`abstract`, `become`, `box`, `do`, `final`, `gen`, `macro`, `override`, `priv`, `try`, `typeof`, `unsized`, `virtual`, `yield`

### Raw identifiers

If you need to use a keyword as an identifier (e.g. calling into a library written in an older edition where something wasn't a keyword), prefix it with `r#`:

```rust
fn r#match(needle: &str, haystack: &str) -> bool {
    haystack.contains(needle)
}

fn main() {
    assert!(r#match("foo", "foobar"));
}
```

Mostly an edition interop tool — you won't write this often.

---

## Appendix B — Operators and Symbols

### Arithmetic and logic

| Operator | Example | Trait |
|---|---|---|
| `+` | `a + b` | `Add` |
| `-` | `a - b` | `Sub` |
| `*` | `a * b` | `Mul` |
| `/` | `a / b` | `Div` |
| `%` | `a % b` | `Rem` |
| `!` | `!expr` | `Not` (bitwise/logical complement) |
| `&` | `a & b` | `BitAnd` |
| `\|` | `a \| b` | `BitOr` |
| `^` | `a ^ b` | `BitXor` |
| `<<` | `a << b` | `Shl` |
| `>>` | `a >> b` | `Shr` |

### Comparison

| Operator | Trait |
|---|---|
| `==`, `!=` | `PartialEq` |
| `<`, `>`, `<=`, `>=` | `PartialOrd` |

`&&` and `||` (short-circuit logical) are not overloadable.

### Assignment

| Operator | Notes |
|---|---|
| `=` | Basic assignment |
| `+=`, `-=`, `*=`, etc. | Compound assignment — traits like `AddAssign` |

### Other common symbols

| Symbol | Context | Meaning |
|---|---|---|
| `->` | `fn foo() -> T` | Return type |
| `=>` | `match arm => body` | Match arm separator |
| `::` | `std::io::Result` | Path separator |
| `.` | `x.field`, `x.method()` | Field access / method call |
| `..` | `0..10` | Exclusive range |
| `..=` | `0..=10` | Inclusive range |
| `..` | `Point { x, .. }` | Ignore remaining fields in a pattern |
| `..` | `..Default::default()` | Struct update syntax |
| `?` | `file.read()?` | Propagate `Err` / `None` |
| `_` | `let _ = x` | Wildcard — discard without binding |
| `*` | `*ptr` | Dereference |
| `&` | `&x`, `&mut x` | Reference / mutable reference |
| `'a` | `&'a T` | Lifetime annotation |
| `'label` | `'outer: loop { break 'outer }` | Loop label |
| `!` | `fn foo() -> !` | Never type |
| `#[...]` | `#[derive(Debug)]` | Outer attribute |
| `#![...]` | `#![allow(unused)]` | Inner attribute (applies to enclosing item) |
| `\|...\|` | `\|x\| x + 1` | Closure |

### Path and generics

| Syntax | Meaning |
|---|---|
| `path::to::item` | Module path |
| `item::<T>` | Turbofish — explicit generic type at call site |
| `<Type as Trait>::method()` | Fully qualified — disambiguate same-name methods |
| `T: Trait` | Trait bound |
| `T: 'a` | Lifetime bound |
| `T: ?Sized` | Relax the `Sized` bound |
| `type Item = u32` | Associated type assignment in impl |

---

## Appendix C — Derivable Traits

`#[derive(...)]` auto-generates a trait implementation. Only works when all fields/variants also implement the trait.

### `Debug`

Enables `{:?}` and `{:#?}` formatting. Required by `assert_eq!` to print values on failure. Derive this on almost everything during development.

```rust
#[derive(Debug)]
struct Point { x: i32, y: i32 }

println!("{:?}", Point { x: 1, y: 2 }); // Point { x: 1, y: 2 }
```

### `PartialEq` and `Eq`

`PartialEq` — enables `==` and `!=`. For structs, all fields must be equal. For enums, each variant equals only itself.

`Eq` — marker trait that adds the guarantee "every value equals itself." No methods. Needed for `HashMap` keys and `HashSet` elements. Can't be derived for types with `f32`/`f64` fields (because `NaN != NaN`).

```rust
#[derive(PartialEq, Eq)]
struct Id(u32);
```

### `PartialOrd` and `Ord`

`PartialOrd` — enables `<`, `>`, `<=`, `>=`. Returns `Option<Ordering>` (can be `None` for things like `NaN`). For structs, fields are compared top-to-bottom in declaration order.

`Ord` — guarantees a total ordering. Returns `Ordering` (never `None`). Required for `BTreeMap`/`BTreeSet`. Must also derive `PartialOrd` and `Eq`.

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Version { major: u32, minor: u32, patch: u32 }
// now you can sort Vec<Version>, use Version in BTreeSet, etc.
```

### `Clone` and `Copy`

`Clone` — deep copy via `.clone()`. Can run arbitrary code (heap allocation, etc.). All fields must implement `Clone`.

`Copy` — bit-for-bit copy, no code runs, no heap involvement. Types that implement `Copy` are implicitly copied instead of moved. Must also implement `Clone`. All fields must implement `Copy`. Can't implement `Copy` if a field has a `Drop` impl.

```rust
#[derive(Clone, Copy)]
struct Point { x: f64, y: f64 } // stack-only, fine to Copy
```

### `Hash`

Maps a value to a fixed-size hash. Required for `HashMap` keys and `HashSet` elements. All fields must implement `Hash`. If you derive `Eq`, you should also derive `Hash` — they need to be consistent (values that are equal must hash the same).

```rust
#[derive(Hash, PartialEq, Eq)]
struct UserId(u64);
```

### `Default`

Provides a sensible zero value via `Type::default()`. Used in struct update syntax and `unwrap_or_default()`. Each field's default is called when deriving.

```rust
#[derive(Default)]
struct Config {
    width: u32,   // defaults to 0
    verbose: bool, // defaults to false
}

let c = Config { width: 80, ..Default::default() }; // verbose = false
```

---

## Appendix D — Dev Tools

Four tools you should have in your workflow:

### `rustfmt`

Auto-formats code to community style standards. Comes with Rust.

```bash
cargo fmt
```

Non-negotiable for collaborative projects. Eliminates style debates.

### `rustfix`

Auto-applies compiler warning fixes. Comes with Rust.

```bash
cargo fix
```

Also handles edition migrations — `cargo fix --edition` rewrites code to compile under the next edition.

### Clippy

Linter that catches common mistakes, bad patterns, and non-idiomatic code. Comes with Rust.

```bash
cargo clippy
```

Example — it'll catch things like hardcoding `3.1415` instead of `std::f64::consts::PI`. Good habit to run before committing.

### `rust-analyzer`

The Language Server Protocol (LSP) backend for Rust — powers autocompletion, jump-to-definition, inline errors, and refactoring in any LSP-compatible editor. Install it via your editor's extension marketplace (VS Code has an official extension).

This is what makes editor support actually good. If it's not installed, get it.

---

## Appendix E — Editions

Rust releases on a 6-week cycle. Every 3 years, a new **edition** is cut that bundles breaking-ish changes (new keywords, syntax changes) into an opt-in package.

Current editions: `2015`, `2018`, `2021`, `2024`. This book uses `2024`.

Set your edition in `Cargo.toml`:

```toml
[package]
edition = "2024"
```

Missing `edition` key defaults to `2015`.

Key points:
- Editions are **opt-in** per crate — your crate can be on 2021 and depend on a crate on 2018, no problem
- The compiler handles cross-edition interop
- Breaking changes only affect code that opts into the new edition
- `cargo fix --edition` automates most of the migration

---

## Appendix G — Nightly Rust and the Release Process

### Three channels

| Channel | What it is |
|---|---|
| **Nightly** | Cuts every night from `main`. All unstable features available. |
| **Beta** | Cut from `main` every 6 weeks. Becomes the next stable after 6 weeks of testing. |
| **Stable** | What most people use. No unstable features. |

### The train schedule

New features land on nightly behind a feature flag. After real-world testing, they get stabilized and promoted to stable. The cadence:

```
nightly: * - - * - - * - - * - - * - - *
                     |                 |
beta:                * - - - - - - - - *
                                       |
stable:                                *
```

Every 6 weeks a new stable ships. Features that aren't ready just ride the train longer.

### Using nightly

```bash
rustup toolchain install nightly

# Override for a specific project only
cd my-project
rustup override set nightly

# List installed toolchains
rustup toolchain list
```

To use an unstable feature in nightly:

```rust
#![feature(some_unstable_feature)] // at the top of main.rs or lib.rs
```

This will fail to compile on stable — intentional. Unstable features can change or be removed before stabilization.

### RFC process

Changes to the language go through an RFC (Request for Comments). Anyone can submit one. The relevant team reviews, discussion happens publicly, and if accepted it lands on nightly first. Stabilization happens after sufficient real-world feedback.

---

## Quick Reference Summary

| Topic | Key takeaway |
|---|---|
| Keywords | `r#keyword` for raw identifiers; reserved words are future-proofed |
| Operators | Most arithmetic/comparison ops are overloadable via `std::ops` traits |
| `?` | Propagates `Err`/`None` — desugars to a `match` + early return |
| Turbofish `::<>` | Explicit generic at call site when inference fails |
| `#[derive]` | Auto-impl for `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`, `Ord` |
| `rustfmt` | `cargo fmt` — always run it |
| Clippy | `cargo clippy` — catches what the compiler misses |
| `rust-analyzer` | LSP backend — powers editor features |
| Editions | Opt-in per crate, fully interoperable, `cargo fix` migrates |
| Nightly | Feature flags for unstable work; `rustup override set nightly` per-project |
