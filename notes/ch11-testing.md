# Ch 11 — Writing Automated Tests

Tests verify that code does what you intend. Rust's type system catches a lot, but not logic errors — that's what tests are for.

---

## Writing Tests

A test is any function annotated with `#[test]`. `cargo test` builds a test runner binary and runs them all. A test fails if it panics.

```rust
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

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

- `assert!(expr)` — passes if `expr` is `true`
- `assert_eq!(a, b)` — passes if `a == b`, prints both values on failure
- `assert_ne!(a, b)` — passes if `a != b`

Values passed to `assert_eq!` / `assert_ne!` must implement `PartialEq` and `Debug` (usually via `#[derive(PartialEq, Debug)]`).

What `assert_eq!` failure looks like:
```
assertion `left == right` failed
  left: 5
 right: 4
```

Custom failure messages — extra args are passed to `format!`:
```rust
assert!(
    result.contains("Carol"),
    "Greeting did not contain name, value was `{result}`"
);
```

### `#[should_panic]`

Tests that expect a panic:
```rust
#[test]
#[should_panic]
fn rejects_out_of_range() {
    Guess::new(200);
}
```

Add `expected` to require a specific substring in the panic message:
```rust
#[test]
#[should_panic(expected = "between 1 and 100")]
fn rejects_out_of_range() {
    Guess::new(200);
}
// passes if the panic message *contains* "between 1 and 100"
```

### Returning `Result<T, E>`

Tests can return `Result` instead of panicking — lets you use `?` inside test bodies:
```rust
#[test]
fn it_works() -> Result<(), String> {
    let result = add(2, 2);
    if result == 4 {
        Ok(())
    } else {
        Err(String::from("two plus two does not equal four"))
    }
}
```

Cannot combine `#[should_panic]` with `Result` tests. To assert an operation fails, use `assert!(value.is_err())`.

---

## Running Tests

```bash
cargo test                          # run all tests
cargo test -- --test-threads=1      # run sequentially (no parallelism)
cargo test -- --show-output         # show stdout from passing tests too
cargo test add                      # run tests whose name contains "add" (substring filter)
cargo test -- --ignored             # run only #[ignore]-marked tests
cargo test -- --include-ignored     # run everything including ignored
cargo test --test integration_test  # run a specific integration test file
```

Tests run in parallel by default — each in its own thread. If tests share state (files, env vars, etc.), use `--test-threads=1` to avoid interference.

`println!` output from passing tests is captured and hidden by default. Only failing tests show their stdout unless you pass `--show-output`.

`cargo test add` runs anything with "add" anywhere in the test path — including the module name. So `tests::add_two_and_two` and `tests::add_three_and_two` both match.

### Ignoring Tests

```rust
#[test]
#[ignore]
fn slow_test() {
    // takes a long time — skip during normal runs
}
```

Ignored tests appear in output as `ignored`, not `ok` or `FAILED`.

---

## Test Organization

### Unit Tests

Live in `src/` alongside the code they test, inside a `#[cfg(test)]` module. The annotation means this code is only compiled during `cargo test`, not `cargo build` — no cost in production binaries.

```rust
fn internal_adder(a: u64, b: u64) -> u64 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;  // pulls in parent scope — including private items

    #[test]
    fn test_private_fn() {
        assert_eq!(internal_adder(2, 2), 4);
    }
}
```

Unit tests can test private functions — `use super::*` brings in everything from the parent module, regardless of visibility. Child modules can always access ancestor items.

### Integration Tests

Live in a top-level `tests/` directory. Each file is compiled as its own separate crate. No `#[cfg(test)]` needed — Cargo handles that automatically.

```
adder/
├── src/
│   └── lib.rs
└── tests/
    └── integration_test.rs
```

```rust
// tests/integration_test.rs
use adder::add_two;  // must explicitly import — it's an external crate from here

#[test]
fn it_adds_two() {
    assert_eq!(add_two(2), 4);
}
```

Integration tests can only use the public API — no access to private internals.

`cargo test` output has three sections:
```
Running unittests src/lib.rs
... (unit test results)

Running tests/integration_test.rs
... (integration test results)

Doc-tests adder
... (doc test results)
```

### Shared Helpers Across Integration Tests

Use `tests/common/mod.rs` — not `tests/common.rs`. A `.rs` file directly in `tests/` is treated as an integration test file. The `common/mod.rs` form is not:

```
tests/
├── common/
│   └── mod.rs        ← shared setup, won't appear as a test file
└── integration_test.rs
```

```rust
// tests/common/mod.rs
pub fn setup() {
    // shared setup logic
}
```

```rust
// tests/integration_test.rs
use adder::add_two;
mod common;

#[test]
fn it_adds_two() {
    common::setup();
    assert_eq!(add_two(2), 4);
}
```

### Doc Tests

Code examples in doc comments are run as tests by `cargo test`. They appear in the `Doc-tests` section of output:

```rust
/// Adds two to the given number.
///
/// # Examples
///
/// ```
/// let result = adder::add_two(5);
/// assert_eq!(result, 7);
/// ```
pub fn add_two(a: u64) -> u64 {
    a + 2
}
```

Doc tests ensure examples in documentation actually compile and run correctly. Covered more in ch14.

### Binary Crates

Integration tests only work on library crates — there's nothing to `use` from a binary. Convention: put logic in `src/lib.rs`, keep `src/main.rs` thin. Integration tests then target the lib, and `main.rs` just wires things together.
