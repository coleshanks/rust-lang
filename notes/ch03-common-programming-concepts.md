# Ch 3 — Common Programming Concepts

---

## Variables and Mutability

Variables are immutable by default. You opt into mutability explicitly.

```rust
let x = 5;
x = 6;  // compile error: cannot assign twice to immutable variable
```

```rust
let mut x = 5;
x = 6;  // fine
```

### Constants

Always immutable. Must have a type annotation. Can be declared in any scope, including global. Only constant expressions allowed — no runtime values.

```rust
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
```

Convention: `SCREAMING_SNAKE_CASE`. Valid for the entire program within their scope.

### Shadowing

Re-declare with `let` to shadow a variable. The old binding is gone within the new scope.

```rust
let x = 5;
let x = x + 1;  // x is now 6

{
    let x = x * 2;
    println!("{x}");  // 12 — inner scope only
}

println!("{x}");  // 6 — outer x unchanged
```

Key difference from `mut`: shadowing lets you change the type. `mut` doesn't.

```rust
let spaces = "   ";       // &str
let spaces = spaces.len(); // usize — totally fine with shadowing

let mut spaces = "   ";
spaces = spaces.len();    // compile error: mismatched types
```

---

## Data Types

Rust is statically typed — all types known at compile time. The compiler infers most types, but sometimes you need to annotate.

```rust
let guess: u32 = "42".parse().expect("Not a number!");
// without the annotation, compiler doesn't know which numeric type to parse into
```

### Scalar Types

#### Integers

| Length | Signed | Unsigned |
|--------|--------|----------|
| 8-bit | `i8` | `u8` |
| 16-bit | `i16` | `u16` |
| 32-bit | `i32` | `u32` |
| 64-bit | `i64` | `u64` |
| 128-bit | `i128` | `u128` |
| arch | `isize` | `usize` |

Default is `i32`. Use `usize` for indexing into collections (pointer-sized).

Signed range: `-(2^(n-1))` to `2^(n-1) - 1`. E.g. `i8`: -128 to 127.

Integer literals:

```rust
let decimal     = 98_222;       // underscores for readability
let hex         = 0xff;
let octal       = 0o77;
let binary      = 0b1111_0000;
let byte        = b'A';         // u8 only
```

**Overflow behavior:**
- Debug mode: panics
- Release mode: wraps (two's complement)

To handle overflow explicitly:

```rust
value.wrapping_add(1)    // always wraps
value.checked_add(1)     // returns Option<T>
value.saturating_add(1)  // clamps at min/max
value.overflowing_add(1) // returns (value, did_overflow)
```

#### Floats

`f32` and `f64` (default). Both signed, IEEE-754. `f64` has more precision, similar speed on modern CPUs.

```rust
let x = 2.0;       // f64
let y: f32 = 3.0;  // f32
```

#### Numeric Operations

```rust
let sum        = 5 + 10;
let difference = 95.5 - 4.3;
let product    = 4 * 30;
let quotient   = 56.7 / 32.2;
let truncated  = -5 / 3;   // -1 (integer division truncates toward zero)
let remainder  = 43 % 5;   // 3
```

#### Boolean

```rust
let t = true;
let f: bool = false;
```

One byte. Used in conditionals — Rust won't auto-coerce integers to bool.

#### Character

```rust
let c = 'z';
let z: char = 'ℤ';
let heart_eyed_cat = '😻';
```

`char` is 4 bytes — a full Unicode scalar value, not just ASCII. Single quotes, not double.

---

### Compound Types

#### Tuples

Fixed length, mixed types. Access by index with `.0`, `.1`, etc., or destructure.

```rust
let tup: (i32, f64, u8) = (500, 6.4, 1);

let (x, y, z) = tup;  // destructure
println!("{y}");       // 6.4

let five_hundred = tup.0;
let six_point_four = tup.1;
```

The empty tuple `()` is the **unit type** — it's what functions return when there's no explicit return value.

#### Arrays

Fixed length, same type, stack-allocated. Not growable — use `Vec` when size is dynamic.

```rust
let a = [1, 2, 3, 4, 5];
let a: [i32; 5] = [1, 2, 3, 4, 5];  // explicit type + length
let a = [3; 5];  // [3, 3, 3, 3, 3]
```

Access:

```rust
let first = a[0];
let second = a[1];
```

Out-of-bounds access **panics at runtime** — Rust checks bounds and won't let you read invalid memory.

---

## Functions

`fn` keyword, `snake_case` convention. Parameters always need type annotations. Order of definition doesn't matter — just needs to be in scope.

```rust
fn main() {
    another_function(5);
    print_labeled_measurement(5, 'h');
}

fn another_function(x: i32) {
    println!("x is: {x}");
}

fn print_labeled_measurement(value: i32, unit_label: char) {
    println!("The measurement is: {value}{unit_label}");
}
```

### Statements vs. Expressions

- **Statement**: performs an action, returns no value (`let y = 6;`)
- **Expression**: evaluates to a value (`5 + 6`, `{ let x = 3; x + 1 }`)

A block `{}` is an expression — its value is the last expression inside (no trailing semicolon).

```rust
let y = {
    let x = 3;
    x + 1   // no semicolon — this is the value of the block
};
// y == 4
```

Adding `;` turns an expression into a statement, making it return `()`.

### Return Values

Declared with `->`. Implicit return is the tail expression (no semicolon).

```rust
fn five() -> i32 {
    5   // returns 5
}

fn plus_one(x: i32) -> i32 {
    x + 1   // returns x + 1
}
```

If you accidentally add a semicolon to the tail:

```rust
fn plus_one(x: i32) -> i32 {
    x + 1;  // compile error: returns () not i32
}
```

You can also use `return` explicitly for early returns.

---

## Comments

```rust
// single line

// for longer comments, just keep
// adding // on each line

let lucky_number = 7; // inline comment — works but less idiomatic

// preferred: put the comment above the line it describes
let lucky_number = 7;
```

Doc comments (`///`) are different — they generate documentation and are covered in Ch 14.

---

## Control Flow

### `if` Expressions

Condition must be `bool` — no C-style truthy coercion.

```rust
let number = 3;

if number < 5 {
    println!("less than five");
} else {
    println!("five or more");
}
```

Won't compile:

```rust
if number {  // error: expected bool, found integer
    ...
}
```

#### `else if`

```rust
let number = 6;

if number % 4 == 0 {
    println!("divisible by 4");
} else if number % 3 == 0 {
    println!("divisible by 3");   // this runs, rest skipped
} else if number % 2 == 0 {
    println!("divisible by 2");
} else {
    println!("not divisible by 4, 3, or 2");
}
```

Only the first matching arm runs.

#### `if` in a `let` binding

`if` is an expression, so it can be used directly in assignments:

```rust
let condition = true;
let number = if condition { 5 } else { 6 };
```

Both arms must return the same type — Rust needs to know `number`'s type at compile time.

```rust
let number = if condition { 5 } else { "six" };  // compile error: type mismatch
```

---

### Loops

#### `loop` — Infinite Loop

Runs until you `break`. Can return a value.

```rust
let mut counter = 0;

let result = loop {
    counter += 1;

    if counter == 10 {
        break counter * 2;  // exits and returns this value
    }
};

println!("{result}");  // 20
```

`continue` skips the rest of the current iteration.

#### Loop Labels

For nested loops, labels let you `break` or `continue` an outer loop from inside an inner one.

```rust
let mut count = 0;

'counting_up: loop {
    let mut remaining = 10;

    loop {
        if remaining == 9 {
            break;              // breaks inner loop
        }
        if count == 2 {
            break 'counting_up; // breaks outer loop
        }
        remaining -= 1;
    }

    count += 1;
}

println!("End count = {count}");  // 2
```

Labels start with `'`.

#### `while` — Conditional Loop

```rust
let mut number = 3;

while number != 0 {
    println!("{number}!");
    number -= 1;
}

println!("LIFTOFF!!!");
```

Cleaner than `loop` + manual break when you have a condition. But still not ideal for iterating over collections (index management, possible bounds panic).

#### `for` — Iteration

Most idiomatic way to loop. No off-by-one risk, no manual indexing, no bounds panics.

```rust
let a = [10, 20, 30, 40, 50];

for element in a {
    println!("the value is: {element}");
}
```

With a range:

```rust
for number in (1..4).rev() {
    println!("{number}!");
}
// 3! 2! 1!
println!("LIFTOFF!!!");
```

`1..4` is exclusive of 4. `.rev()` reverses it. `1..=4` would include 4.

---

## Fibonacci Project

Built an nth Fibonacci number generator:

- `Vec<u128>` pre-seeded with `[0, 1]` — `u128` handles values up to ~186th term
- `loop` for input validation with re-prompting on bad input
- Match guard (`Ok(num) if num > 0`) to validate and bind in one arm
- `while` loop with a `usize` counter to grow the sequence
- Early `return` for the `n == 1` edge case
