# Ch 2 — Programming a Guessing Game

## Variables and Mutability
- Variables are immutable by default — `let x` can't be reassigned
- `let mut` makes it mutable
- `String::new()` creates an empty, growable string (heap-allocated)

## User Input
```rust
io::stdin()
    .read_line(&mut guess)
    .expect("Failed to read line");
```
- `read_line` appends input (including `\n`) to the string
- `&mut guess` passes a mutable reference — required so the function can write into it
- `read_line` returns a `Result` — `.expect()` panics with the message if it's `Err`

## External Crates
- Add to `Cargo.toml` under `[dependencies]`: `rand = "0.8.5"`
- `cargo build` fetches and compiles it
- `Cargo.lock` pins exact versions for reproducibility
- `use rand::Rng;` brings the trait into scope (needed to call `.gen_range()`)

## Result and Error Handling
- Functions that can fail return `Result<T, E>` — an enum with variants `Ok(T)` and `Err(E)`
- No exceptions in Rust — the compiler forces you to handle the error case
- `.expect("msg")` is a shortcut that panics on `Err` (fine for prototyping, not production)
- `match` is the idiomatic way to handle both variants:
```rust
match guess.trim().parse() {
    Ok(num) => num,
    Err(_) => continue,
}
```

## match Expressions
- Like a `switch` in C but exhaustive — compiler errors if you miss a case
- Each `pattern => expression` is called an **arm**
- Patterns can destructure values (e.g. `Ok(num)` binds the inner value to `num`)
- `_` means "ignore this value"

## Type Conversion and Shadowing
```rust
let guess: u32 = match guess.trim().parse() { ... };
```
- `.trim()` strips whitespace and `\n`
- `.parse()` converts the string to the annotated type (`u32` here)
- Re-using the name `guess` is **shadowing** — intentional, lets you reuse a name with a new type

## Ordering and Comparison
```rust
use std::cmp::Ordering;

match guess.cmp(&secret_number) {
    Ordering::Less => println!("Too small!"),
    Ordering::Greater => println!("Too big!"),
    Ordering::Equal => { println!("You win!"); break; }
}
```
- `cmp` returns an `Ordering` enum: `Less`, `Greater`, or `Equal`
- `::` is the module/namespace separator

## loop, break, continue
- `loop` is an infinite loop — exit with `break`
- `continue` skips to the next iteration
- Used here to keep prompting on invalid input and exit on correct guess

## Range Syntax
- `1..=100` is an inclusive range (1 to 100)
- `1..100` would be exclusive (1 to 99)

## Cargo fmt / Clippy
- `cargo fmt` auto-formats code — just let it handle whitespace, don't fight it
- `cargo fmt --check` shows what would change without modifying files
- `cargo clippy` catches idiomatic issues and suggests better patterns (read-only, never modifies)
- `cargo clippy --fix` auto-applies easy fixes
