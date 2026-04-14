# Ch 9 — Error Handling

Rust has no exceptions. Two categories of errors:

- **Unrecoverable** — bug, something that should never happen → `panic!`
- **Recoverable** — expected failure, caller should decide → `Result<T, E>`

---

## `panic!` — Unrecoverable Errors

Prints an error message, unwinds the stack, and quits. Two ways to trigger:

```rust
panic!("crash and burn");  // explicit
let v = vec![1, 2, 3];
v[99];                     // implicit — index out of bounds, panics at runtime
```

Code compiles fine — panic happens at runtime.

**Backtraces:**
```bash
RUST_BACKTRACE=1 cargo run
RUST_BACKTRACE=full cargo run  # more verbose
```

Shows the call stack at the point of the panic. Read from the **top** — start at the top and read down until you see a file you wrote. That's where the problem originated. Lines above it are std/library internals, lines below are what called your code.

**Unwinding vs abort:**

Default: Rust unwinds the stack on panic (walks back up, runs `drop` for each frame). For smaller binaries, abort immediately instead:

```toml
[profile.release]
panic = 'abort'
```

Abort skips cleanup — the OS reclaims memory. Smaller binary, but no destructors run.

---

## `Result<T, E>` — Recoverable Errors

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`T` is the success type, `E` is the error type. Both generic — filled in from context.

**Basic handling:**
```rust
use std::fs::File;

let greeting_file = match File::open("hello.txt") {
    Ok(file) => file,
    Err(error) => panic!("Problem opening the file: {error:?}"),
};
```

**Matching on specific error kinds:**
```rust
use std::fs::File;
use std::io::ErrorKind;

let greeting_file = match File::open("hello.txt") {
    Ok(file) => file,
    Err(error) => match error.kind() {
        ErrorKind::NotFound => match File::create("hello.txt") {
            Ok(fc) => fc,
            Err(e) => panic!("Problem creating the file: {e:?}"),
        },
        _ => panic!("Problem opening the file: {error:?}"),
    },
};
```

**`unwrap()`** — returns the `Ok` value or panics with a generic message:
```rust
let f = File::open("hello.txt").unwrap();
```

**`expect()`** — same but with a custom panic message. Prefer this — the message tells you *why* you expected success:
```rust
let f = File::open("hello.txt").expect("hello.txt should be present in the project");
```

**`unwrap_or_else()`** — panics or runs a closure on error (cleaner than nested `match` for complex error logic):
```rust
let greeting_file = File::open("hello.txt").unwrap_or_else(|error| {
    if error.kind() == ErrorKind::NotFound {
        File::create("hello.txt").unwrap_or_else(|error| {
            panic!("Problem creating the file: {error:?}");
        })
    } else {
        panic!("Problem opening the file: {error:?}");
    }
});
```

---

## Propagating Errors with `?`

Instead of handling the error inside the function, return it to the caller.

Manual way — verbose:
```rust
use std::fs::File;
use std::io::{self, Read};

fn read_username_from_file() -> Result<String, io::Error> {
    let mut username_file = match File::open("hello.txt") {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let mut username = String::new();

    match username_file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}
```

With `?` — same logic, much less noise:
```rust
fn read_username_from_file() -> Result<String, io::Error> {
    let mut username_file = File::open("hello.txt")?;
    let mut username = String::new();
    username_file.read_to_string(&mut username)?;
    Ok(username)
}
```

`?` on a `Result`:
- `Ok(val)` → unwraps to `val`, execution continues
- `Err(e)` → converts `e` via `From::from` if needed, then returns early with that error

Can chain calls:
```rust
fn read_username_from_file() -> Result<String, io::Error> {
    let mut username = String::new();
    File::open("hello.txt")?.read_to_string(&mut username)?;
    Ok(username)
}
```

Or even shorter with the stdlib shortcut:
```rust
use std::fs;

fn read_username_from_file() -> Result<String, io::Error> {
    fs::read_to_string("hello.txt")  // opens, reads, and returns — all in one
}
```

**`?` on `Option`:**

`?` works on `Option<T>` too — returns `None` early if the value is `None`:

```rust
fn last_char_of_first_line(text: &str) -> Option<char> {
    text.lines().next()?.chars().last()
    // .next() returns Option<&str>
    // ? returns None early if there are no lines
    // .chars().last() returns Option<char>
}
```

Can't mix `Result` and `Option` with `?` — must convert explicitly using `.ok()` or `.ok_or()`.

**Constraint:** `?` only works in functions that return `Result`, `Option`, or a type implementing `FromResidual`. Using it in a function that returns `()` is a compile error.

**`?` in `main`:**
```rust
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let f = File::open("hello.txt")?;
    Ok(())
}
```

`Ok(())` → exit code 0. `Err` → nonzero exit. `Box<dyn Error>` means "any error type" — covered more in ch17 (trait objects).

---

## When to `panic!` vs `Result`

**Default: return `Result`** — gives the caller the choice of how to handle failure.

**Use `panic!` when:**
- Writing examples, prototypes, or tests — `unwrap`/`expect` are fine placeholders
- You know more than the compiler (e.g. hardcoded value that's provably valid):
  ```rust
  let home: IpAddr = "127.0.0.1"
      .parse()
      .expect("hardcoded IP is always valid");
  ```
- A contract has been violated — the caller passed nonsensical values and continuing would be a bug, not a recoverable situation

**Library code:** panic on invalid input that indicates a caller bug. Return `Result` for expected failures (bad user input, missing files, network errors, etc.).

---

## Custom Validation Types

Encode validity in the type system so you don't need to re-validate everywhere:

```rust
pub struct Guess {
    value: i32,  // private — can only be set through new()
}

impl Guess {
    pub fn new(value: i32) -> Guess {
        if value < 1 || value > 100 {
            panic!("Guess value must be between 1 and 100, got {value}.");
        }
        Guess { value }
    }

    pub fn value(&self) -> i32 {
        self.value  // read-only getter
    }
}
```

Any function that takes a `Guess` knows the value is already 1–100 — no need to check again. The type carries the invariant.
