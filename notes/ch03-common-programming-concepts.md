# Ch 3 — Common Programming Concepts

## Variables and Mutability
- Variables are immutable by default — `let x` can't be reassigned
- `let mut x` makes it mutable
- **Constants** — `const`, always immutable, type annotation required, ALL_CAPS convention:
  ```rust
  const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
  ```
- **Shadowing** — re-declare with `let` to reuse a name, even with a different type:
  ```rust
  let x = 5;
  let x = x + 1; // shadows previous x
  ```
  Key difference from `mut`: shadowing lets you change the type, `mut` doesn't

## Data Types

### Scalar Types
- **Integers**: `i8`–`i128`, `u8`–`u128`, `isize`/`usize` — default is `i32`
  - Use `usize` for collection indexing
  - Debug mode panics on overflow; release mode wraps silently
- **Floats**: `f32`, `f64` (default) — IEEE-754, both signed
- **Boolean**: `bool` — `true` or `false`, 1 byte
- **Character**: `char` — 4 bytes, full Unicode scalar value (not just ASCII)

### Compound Types
- **Tuple** — fixed length, mixed types, access by index (`tup.0`) or destructure
  - Empty tuple `()` is the **unit type** — represents "no value"
- **Array** — fixed length, same type, stack-allocated
  - `let a = [3; 5]` → `[3, 3, 3, 3, 3]`
  - Out-of-bounds access panics at runtime (Rust guarantees memory safety)
  - Use `Vec` when you need a growable collection

## Functions
- Named in `snake_case`, parameters always need type annotations
- **Statements** don't return a value; **expressions** do
- A block `{}` is an expression — its value is the last expression (no semicolon)
- Adding `;` to a tail expression turns it into a statement returning `()`
- Return type declared with `->`, implicit return via tail expression:
  ```rust
  fn plus_one(x: i32) -> i32 {
      x + 1  // no semicolon
  }
  ```

## Control Flow

### `if` Expressions
- Condition must be `bool` — no auto-coercion from integers like in C/JS
- `if` is an expression — can be used in a `let` binding:
  ```rust
  let number = if condition { 5 } else { 6 };
  ```
- All arms must return the same type (enforced at compile time)

### `loop`
- Runs forever until `break`
- Can return a value via `break value` — the whole `loop` becomes an expression:
  ```rust
  let result = loop {
      counter += 1;
      if counter == 10 { break counter * 2; }
  };
  ```
- **Loop labels** — `'label: loop { ... }` lets you `break` or `continue` an outer loop from inside an inner one

### `while`
- Runs while a condition is true — cleaner than `loop` + manual `break`

### `for`
- Most idiomatic way to iterate — no off-by-one risk, no manual indexing:
  ```rust
  for element in collection { ... }
  ```
- Use `Range` with `.rev()` for countdowns:
  ```rust
  for number in (1..4).rev() { ... }
  ```

## Vec
- Growable array — use when size is known only at runtime
- `Vec::new()` creates empty; `vec![0, 1]` creates with initial values
- `.push(value)` appends an element
- Indices must be `usize` — cast with `as usize` when needed

## Fibonacci Project
Built an nth fibonacci number generator using:
- `Vec<u128>` pre-seeded with `[0, 1]` — `u128` to handle large values (up to ~186th term)
- `loop` for input validation with re-prompting on invalid input
- Match guards (`Ok(num) if num > 0`) to validate and bind in one arm
- `while` loop with a `usize` counter to grow the sequence
- Early `return` for the `n == 1` edge case
