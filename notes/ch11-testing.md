# Ch 11 — Writing Automated Tests

Tests verify that code does what you intend. Rust's type system catches a lot, but not logic errors — that's what tests are for.

---

## Writing Tests

A test is any function annotated with `#[test]`. `cargo test` builds a test runner binary and runs them all.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
```

### Assertion Macros

- `assert!(expr)` — passes if `expr` is true
- `assert_eq!(a, b)` — passes if `a == b`, shows both values on failure
- `assert_ne!(a, b)` — passes if `a != b`

Values passed to `assert_eq!` / `assert_ne!` must implement `PartialEq` and `Debug`.

Custom failure messages (extra args are passed to `format!`):
```rust
assert!(result.contains("Carol"), "Got: `{result}`");
```

### Panics and `should_panic`

```rust
#[test]
#[should_panic]
fn rejects_out_of_range() {
    Guess::new(200);
}
```

Add `expected` to pin down which panic message is acceptable:
```rust
#[test]
#[should_panic(expected = "less than or equal to 100")]
fn rejects_out_of_range() {
    Guess::new(200);
}
```

### Returning `Result<T, E>`

Tests can return `Result` instead of panicking — lets you use `?` inside tests:
```rust
#[test]
fn it_works() -> Result<(), String> {
    if add(2, 2) == 4 {
        Ok(())
    } else {
        Err(String::from("two plus two does not equal four"))
    }
}
```

Cannot combine `#[should_panic]` with `Result` tests. Use `assert!(value.is_err())` instead.

---

## Running Tests

```bash
cargo test                        # run all tests
cargo test -- --test-threads=1   # run sequentially (no parallelism)
cargo test -- --show-output      # show stdout from passing tests
cargo test add                   # run tests whose name contains "add"
cargo test --test integration_test  # run a specific integration test file
```

Tests run in parallel by default. If tests share state (files, env vars, etc.), use `--test-threads=1`.

Output from passing tests is captured by default — only failures show their stdout.

### Ignoring Tests

```rust
#[test]
#[ignore]
fn slow_test() { ... }
```

```bash
cargo test -- --ignored          # run only ignored tests
cargo test -- --include-ignored  # run everything
```

---

## Test Organization

### Unit Tests

Live in `src/` alongside the code they test, inside a `#[cfg(test)]` module. The `#[cfg(test)]` annotation means this code is only compiled when running `cargo test`, not `cargo build`.

```rust
#[cfg(test)]
mod tests {
    use super::*;  // gives access to parent module, including private items

    #[test]
    fn test_private_fn() { ... }
}
```

Unit tests can test private functions — `use super::*` pulls in everything from the parent scope, including non-`pub` items.

### Integration Tests

Live in a top-level `tests/` directory. Each file is compiled as its own crate. No `#[cfg(test)]` needed — Cargo handles that automatically.

```
adder/
├── src/lib.rs
└── tests/
    └── integration_test.rs
```

```rust
// tests/integration_test.rs
use adder::add_two;

#[test]
fn it_adds_two() {
    assert_eq!(add_two(2), 4);
}
```

Integration tests can only use the public API.

### Shared Helpers Across Integration Tests

Use `tests/common/mod.rs` (not `tests/common.rs`) so Cargo doesn't treat it as a test file:

```
tests/
├── common/
│   └── mod.rs      ← shared setup code
└── integration_test.rs
```

```rust
// integration_test.rs
mod common;

#[test]
fn it_adds_two() {
    common::setup();
    assert_eq!(add_two(2), 4);
}
```

### Binary Crates

Integration tests only work on library crates. If your project is a binary (`src/main.rs` only), there's nothing to `use`. Convention: put logic in `src/lib.rs`, keep `main.rs` thin — then integration tests can target the lib.
