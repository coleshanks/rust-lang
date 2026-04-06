# Ch 9 — Error Handling

Rust has no exceptions. Two categories of errors, handled differently:

- **Unrecoverable** — bug, something that should never happen → `panic!`
- **Recoverable** — expected failure, caller should decide what to do → `Result<T, E>`

---

## `panic!` — Unrecoverable Errors

Prints an error message, unwinds the stack, and quits. Two ways to trigger:

```rust
panic!("crash and burn");   // explicit
v[99];                      // implicit — index out of bounds
```

Code compiles fine — panic happens at runtime.

**Backtraces:**
```bash
RUST_BACKTRACE=1 ./main
```

Shows the call stack at the point of the panic. Read it bottom to top — bottom is where execution started, top is where it crashed. Find the first line that's your code (not `std::` or `core::`) — that's where the bug is.

**Unwinding vs abort:**

By default Rust unwinds the stack on panic (walks back up, cleans up each frame). For smaller binaries you can abort immediately instead:

```toml
[profile.release]
panic = 'abort'
```

---

## `Result<T, E>` — Recoverable Errors

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`T` is the success type, `E` is the error type. Both generic — filled in by context.

**Basic handling:**
```rust
let greeting_file = match File::open("hello.txt") {
    Ok(file) => file,
    Err(error) => panic!("Problem opening the file: {error:?}"),
};
```

**Matching on specific error kinds:**
```rust
match File::open("hello.txt") {
    Ok(file) => file,
    Err(error) => match error.kind() {
        ErrorKind::NotFound => // create it
        _ => panic!("Problem opening the file: {error:?}"),
    },
}
```

**`unwrap()`** — returns the value or panics:
```rust
let f = File::open("hello.txt").unwrap();
```

**`expect()`** — same but with a custom panic message (prefer this over `unwrap` in real code):
```rust
let f = File::open("hello.txt").expect("hello.txt should exist");
```

---

## Propagating Errors with `?`

Instead of handling the error yourself, pass it up to the caller.

Manual way:
```rust
fn read_username_from_file() -> Result<String, io::Error> {
    let mut f = match File::open("hello.txt") {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    // ...
}
```

With `?`:
```rust
fn read_username_from_file() -> Result<String, io::Error> {
    let mut f = File::open("hello.txt")?;
    let mut username = String::new();
    f.read_to_string(&mut username)?;
    Ok(username)
}
```

`?` on a `Result`:
- `Ok` → unwraps and continues
- `Err` → returns early with that error

Can chain:
```rust
File::open("hello.txt")?.read_to_string(&mut username)?;
```

**Constraint:** `?` only works in functions that return `Result`, `Option`, or a type implementing `FromResidual`. Using it in a function that returns `()` is a compile error.

**`?` in `main`:**
```rust
fn main() -> Result<(), Box<dyn Error>> {
    let f = File::open("hello.txt")?;
    Ok(())
}
```
`Ok(())` → exit code 0. `Err` → nonzero exit.

---

## When to `panic!` vs `Result`

**Use `Result` by default** when writing functions that can fail — gives the caller the choice.

**Use `panic!` when:**
- Writing examples, prototypes, or tests — `unwrap`/`expect` are fine as placeholders
- You know more than the compiler (e.g. you've verified the value is valid, use `expect` with a comment explaining why)
- A contract has been violated — the caller passed invalid data and continuing would be a bug, not a recoverable situation

**Library code:** panic on invalid input that indicates a caller bug. Return `Result` for expected failures (bad input from users, missing files, network errors, etc.).

**Custom validation types:**

Encode validity in the type system so you don't have to re-validate everywhere:

```rust
pub struct Guess {
    value: i32,
}

impl Guess {
    pub fn new(value: i32) -> Guess {
        if value < 1 || value > 100 {
            panic!("Guess value must be between 1 and 100, got {value}.");
        }
        Guess { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}
```

Any function that takes a `Guess` knows the value is already valid — no need to check again.
