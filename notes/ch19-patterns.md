# Ch 19 — Patterns and Matching

Patterns are a special syntax for matching against the structure of types. Combined with `match` and other constructs, they give fine-grained control over program flow.

A pattern is made of some combination of: literals, destructured arrays/enums/structs/tuples, variables, wildcards (`_`), and `..` placeholders. The runtime checks whether the value has the right "shape" — if it matches, bindings are created; if not, the associated code doesn't run.

---

## 19.1 — All the Places Patterns Can Be Used

Patterns show up more places than just `match`. You've been using them constantly without thinking about it.

### `match` arms

```rust
match VALUE {
    PATTERN => EXPRESSION,
    PATTERN => EXPRESSION,
}
```

`match` must be exhaustive — every possible value needs to be handled. `_` as the final arm is a common catch-all.

### `let` statements

Every `let` uses a pattern — even `let x = 5;`. The variable name `x` *is* the pattern.

```rust
let (x, y, z) = (1, 2, 3); // tuple destructuring — x=1, y=2, z=3
```

The pattern must match the shape of the right-hand side. Mismatched element counts are a compile error.

### `if let`

Shorter alternative to a `match` that only cares about one variant. Can be chained with `else if` and `else if let` — branches don't need to relate to each other.

```rust
let favorite_color: Option<&str> = None;
let is_tuesday = false;
let age: Result<u8, _> = "34".parse();

if let Some(color) = favorite_color {
    println!("Using {color}");
} else if is_tuesday {
    println!("Green day");
} else if let Ok(age) = age {
    if age > 30 {
        println!("Purple");
    } else {
        println!("Orange");
    }
} else {
    println!("Blue");
}
```

Key detail: `if let Ok(age) = age` introduces a new `age` binding that shadows the outer one. The inner `if age > 30` check must live *inside* that block — the new `age` doesn't exist outside it.

Downside: the compiler doesn't check for exhaustiveness. Missing cases are a logic bug, not a compile error.

### `while let`

Runs a loop as long as the pattern matches:

```rust
let (tx, rx) = std::sync::mpsc::channel();
std::thread::spawn(move || {
    for val in [1, 2, 3] { tx.send(val).unwrap(); }
});

while let Ok(value) = rx.recv() {
    println!("{value}"); // stops when sender disconnects and recv() returns Err
}
```

### `for` loops

The variable after `for` is a pattern. Useful for destructuring while iterating:

```rust
let v = vec!['a', 'b', 'c'];

for (index, value) in v.iter().enumerate() {
    println!("{value} is at index {index}");
}
```

`enumerate()` yields tuples `(index, &value)` — the `(index, value)` in the loop is a pattern that destructures each one.

### Function parameters

Parameter names are patterns. You can destructure directly in the signature:

```rust
fn print_coordinates(&(x, y): &(i32, i32)) {
    println!("({x}, {y})");
}

fn main() {
    let point = (3, 5);
    print_coordinates(&point); // prints (3, 5)
}
```

Same applies to closure parameters.

---

## 19.2 — Refutability

Patterns come in two forms:

- **Irrefutable** — always matches, no matter what value is passed. `x` in `let x = 5` — `x` matches everything.
- **Refutable** — can fail to match for some value. `Some(x)` in `if let Some(x) = val` — fails if `val` is `None`.

Each context only accepts one or the other:

| Context | Accepts |
|---|---|
| `let`, `for`, function params | Irrefutable only |
| `if let`, `while let` | Refutable (irrefutable works but gives a warning) |
| `let...else` | Refutable |
| `match` arms | Refutable (last arm must be irrefutable to ensure exhaustiveness) |

### Refutable pattern in `let` — compile error

```rust
let some_option_value: Option<i32> = None;
let Some(x) = some_option_value; // error: pattern `None` not covered
```

`let` requires an irrefutable pattern because there's no branch to fall through to if it doesn't match.

### Fix: use `let...else`

```rust
let Some(x) = some_option_value else {
    return;
};
// x is in scope here
```

`let...else` accepts a refutable pattern. If it doesn't match, the `else` block must diverge (return, break, panic, etc.).

### Irrefutable pattern in `if let` — warning

```rust
if let x = 5 { // warning: irrefutable pattern, else clause is useless
    println!("{x}");
}
```

Technically valid but pointless — just use `let x = 5`.

---

## 19.3 — Pattern Syntax

### Matching literals

```rust
let x = 1;
match x {
    1 => println!("one"),
    2 => println!("two"),
    _ => println!("other"),
}
```

### Named variables — shadowing in match

Variables in patterns create new bindings scoped to their arm. They shadow outer variables with the same name:

```rust
let x = Some(5);
let y = 10;

match x {
    Some(50) => println!("Got 50"),
    Some(y) => println!("Matched, y = {y}"), // new y — shadows outer y (10), matches inner value (5)
    _ => println!("Default, x = {x:?}"),
}

println!("at the end: x = {x:?}, y = {y}"); // y is still 10 here
```

This is a common gotcha. See match guards (below) for the solution.

### Multiple patterns with `|`

```rust
match x {
    1 | 2 => println!("one or two"),
    3     => println!("three"),
    _     => println!("other"),
}
```

### Ranges with `..=`

```rust
match x {
    1..=5 => println!("one through five"),
    _     => println!("other"),
}
```

Works for `char` too:

```rust
match c {
    'a'..='j' => println!("early letter"),
    'k'..='z' => println!("late letter"),
    _         => println!("other"),
}
```

Only inclusive ranges (`..=`) are allowed in patterns. Exclusive (`..`) is not.

### Destructuring structs

```rust
struct Point { x: i32, y: i32 }

let p = Point { x: 0, y: 7 };

let Point { x: a, y: b } = p; // a=0, b=7

// shorthand when variable names match field names:
let Point { x, y } = p; // x=0, y=7
```

Mix literals and bindings in the same pattern:

```rust
match p {
    Point { x, y: 0 } => println!("on x axis at {x}"),
    Point { x: 0, y } => println!("on y axis at {y}"),
    Point { x, y }    => println!("at ({x}, {y})"),
}
```

### Destructuring enums

Match on variant structure — unit, tuple, or struct variants each have their own syntax:

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

match msg {
    Message::Quit                   => println!("Quit"),
    Message::Move { x, y }         => println!("Move to ({x}, {y})"),
    Message::Write(text)            => println!("Text: {text}"),
    Message::ChangeColor(r, g, b)  => println!("Color: ({r}, {g}, {b})"),
}
```

### Nested destructuring

Match enums inside enums (or any nesting):

```rust
enum Color { Rgb(i32, i32, i32), Hsv(i32, i32, i32) }
enum Message { ChangeColor(Color), /* ... */ }

match msg {
    Message::ChangeColor(Color::Rgb(r, g, b)) => println!("RGB ({r}, {g}, {b})"),
    Message::ChangeColor(Color::Hsv(h, s, v)) => println!("HSV ({h}, {s}, {v})"),
    _ => (),
}
```

You can go as deep as the type structure goes.

### Ignoring values

**`_` — wildcard, matches and discards:**

```rust
fn foo(_: i32, y: i32) { // first param ignored entirely
    println!("{y}");
}
```

**Nested `_` — ignore specific parts:**

```rust
match (setting_value, new_setting_value) {
    (Some(_), Some(_)) => println!("can't overwrite"),
    _ => { setting_value = new_setting_value; }
}
```

**`_x` — suppresses unused variable warning but still binds:**

```rust
let _x = 5; // no warning, but _x owns the value
```

Critical distinction — `_x` binds (moves/borrows), `_` does not:

```rust
let s = Some(String::from("hello"));

if let Some(_s) = s { // _s takes ownership of the String — s is moved
    println!("found");
}
println!("{s:?}"); // ERROR: s was moved into _s
```

```rust
let s = Some(String::from("hello"));

if let Some(_) = s { // _ doesn't bind — s is untouched
    println!("found");
}
println!("{s:?}"); // OK
```

**`..` — ignore remaining fields:**

```rust
struct Point { x: i32, y: i32, z: i32 }
let origin = Point { x: 0, y: 0, z: 0 };

match origin {
    Point { x, .. } => println!("x is {x}"), // y and z ignored
}
```

Works in tuples too:

```rust
let numbers = (2, 4, 8, 16, 32);
match numbers {
    (first, .., last) => println!("{first}, {last}"), // 2, 32
}
```

`..` must be unambiguous — you can't use it twice in the same pattern (`(.., second, ..)` is a compile error).

### Match guards

An extra `if` condition on a match arm. Runs only if the pattern already matched:

```rust
let num = Some(4);
match num {
    Some(x) if x % 2 == 0 => println!("{x} is even"),
    Some(x)                => println!("{x} is odd"),
    None                   => (),
}
```

Match guards solve the named-variable shadowing problem — use a guard to compare against an outer variable without shadowing it:

```rust
let x = Some(5);
let y = 10;

match x {
    Some(50)        => println!("Got 50"),
    Some(n) if n == y => println!("Matched, n = {n}"), // n == y compares to outer y (10)
    _               => println!("Default"),
}
```

With `|`, the guard applies to all alternatives — `(4 | 5 | 6) if y`, not `4 | (5 | 6 if y)`:

```rust
match x {
    4 | 5 | 6 if y => println!("yes"), // guard covers all three
    _              => println!("no"),
}
```

### `@` bindings

Bind a value to a name while simultaneously testing it against a pattern. Without `@`, you can test a value or bind it — not both at once.

```rust
enum Message { Hello { id: i32 } }

let msg = Message::Hello { id: 5 };

match msg {
    Message::Hello { id: id @ 3..=7 } => println!("id in range: {id}"), // bind AND test
    Message::Hello { id: 10..=12 }    => println!("id in 10-12"),        // test only, no binding
    Message::Hello { id }             => println!("other id: {id}"),
}
```

The three arms show the contrast clearly:

- `id: id @ 3..=7` — tests that id is in range AND binds it, so `id` is available in the body
- `id: 10..=12` — tests the range, but `id` is not bound — you can't use it in the body
- `id` — binds unconditionally, no test

`@` syntax: `name @ pattern` — the name to bind on the left, the pattern to test on the right:

```rust
id @ 3..=7
//  ^       — name to bind to
//    ^^^^^ — pattern to test against
```

Works with any pattern, not just ranges — `n @ Some(1..=5)` in a nested context works the same way.

---

## Summary

| Syntax | What it does |
|---|---|
| `1`, `'a'`, `true` | Match a specific literal value |
| `x` | Bind any value to `x` (irrefutable) |
| `1 \| 2` | Match either pattern |
| `1..=5` | Match an inclusive range |
| `Point { x, y }` | Destructure a struct |
| `Some(x)`, `Ok(v)` | Destructure an enum variant |
| `(a, b)` | Destructure a tuple |
| `_` | Wildcard — match without binding |
| `_x` | Bind but suppress unused warning |
| `..` | Ignore remaining fields |
| `guard: pat if cond` | Pattern + extra condition |
| `x @ range` | Bind value while testing it |
