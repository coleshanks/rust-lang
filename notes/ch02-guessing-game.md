# Ch 2 — Programming a Guessing Game

A practical intro chapter — builds a number guessing game while touching I/O, crates, error handling, pattern matching, and loops. Most concepts get a full treatment in later chapters.

---

## User Input

```rust
use std::io;

let mut guess = String::new();

io::stdin()
    .read_line(&mut guess)
    .expect("Failed to read line");
```

- `String::new()` — empty, growable, heap-allocated string
- `read_line` appends input (including the `\n`) into the string
- `&mut guess` — passes a mutable reference so the function can write into it
- `read_line` returns `Result` — `.expect()` panics with the message if it's `Err`

---

## External Crates

Add to `Cargo.toml`:

```toml
[dependencies]
rand = "0.8.5"
```

`cargo build` fetches and compiles it. `Cargo.lock` pins exact versions for reproducibility.

Generating a random number:

```rust
use rand::Rng;

let secret_number = rand::thread_rng().gen_range(1..=100);
```

- `use rand::Rng` — brings the trait into scope (required to call `.gen_range()`)
- `rand::thread_rng()` — gets the thread-local RNG, seeded by the OS
- `1..=100` — inclusive range. `1..100` would exclude 100.

---

## Result and Error Handling

Functions that can fail return `Result<T, E>` — an enum with `Ok(T)` and `Err(E)`. No exceptions in Rust — the compiler makes you handle the error case.

- `.expect("msg")` — panics on `Err`, fine for prototyping
- `match` is the idiomatic way to handle both variants:

```rust
let guess: u32 = match guess.trim().parse() {
    Ok(num) => num,
    Err(_) => continue,
};
```

---

## match Expressions

Like a `switch` in C but exhaustive — compiler errors if you miss a case.

- Each `pattern => expression` is an **arm**
- Patterns can destructure values (`Ok(num)` binds the inner value to `num`)
- `_` ignores the value (catch-all)

---

## Type Conversion and Shadowing

```rust
let guess: u32 = match guess.trim().parse() { ... };
```

- `.trim()` — strips whitespace and `\n` from the string
- `.parse()` — converts to the annotated type (`u32` here), returns `Result`
- Re-using the name `guess` is **shadowing** — lets you reuse a name with a new type without needing a separate variable

---

## Ordering and Comparison

```rust
use std::cmp::Ordering;

match guess.cmp(&secret_number) {
    Ordering::Less    => println!("Too small!"),
    Ordering::Greater => println!("Too big!"),
    Ordering::Equal   => {
        println!("You win!");
        break;
    }
}
```

- `cmp` returns an `Ordering` enum: `Less`, `Greater`, or `Equal`
- `::` is the path/namespace separator

---

## loop, break, continue

- `loop` — infinite loop, exit with `break`
- `continue` — skip to the next iteration
- Used here to keep prompting on invalid input and exit on a correct guess

---

## Cargo fmt / Clippy

```bash
cargo fmt              # auto-formats code
cargo fmt --check      # shows what would change, no modifications
cargo clippy           # catches unidiomatic patterns (read-only)
cargo clippy --fix     # auto-applies easy fixes
```

---

## Final Program

```rust
use std::cmp::Ordering;
use std::io;
use rand::Rng;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1..=100);

    loop {
        println!("Please input your guess.");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num)  => num,
            Err(_)   => continue,
        };

        println!("You guessed: {guess}");

        match guess.cmp(&secret_number) {
            Ordering::Less    => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal   => {
                println!("You win!");
                break;
            }
        }
    }
}
```
