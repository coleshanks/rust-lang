# Ch 13 — Closures and Iterators

Two closely related features borrowed from functional programming. Both are zero-cost abstractions — expressive, high-level code that compiles down to the same thing as hand-written low-level code.

---

## Closures

Anonymous functions that can capture values from the scope they're defined in. Unlike regular functions, closures have access to variables outside their own body.

```rust
let only_borrows = || println!("{list:?}");
```

`||` is the closure syntax — parameters go inside the pipes. Body follows, either as a single expression or a `{}` block.

### Type Inference

Closures don't require type annotations — the compiler infers them from context. You can add them, but don't have to:

```rust
fn  add_one_v1   (x: u32) -> u32 { x + 1 }  // function
let add_one_v2 = |x: u32| -> u32 { x + 1 };  // closure, annotated
let add_one_v3 = |x|             { x + 1 };  // closure, inferred
let add_one_v4 = |x|               x + 1  ;  // closure, single expression
```

Once a closure is called with a specific type, that type is locked in. Calling it with a different type afterwards is a compiler error.

### Capturing the Environment

Closures capture in the least restrictive way possible — the compiler figures out what's needed:

**Immutable borrow** — when the closure only reads:
```rust
let list = vec![1, 2, 3];
let only_borrows = || println!("{list:?}");
only_borrows();
println!("{list:?}"); // still accessible — only borrowed
```

**Mutable borrow** — when the closure mutates:
```rust
let mut list = vec![1, 2, 3];
let mut borrows_mutably = || list.push(7);
borrows_mutably();
// can't use list here while mutable borrow is active
println!("{list:?}"); // ok after closure is done
```

**Move** — force the closure to take ownership, usually for threading:
```rust
thread::spawn(move || println!("{list:?}")).join().unwrap();
```

`move` is required when passing a closure to a new thread, because the thread might outlive the current scope — Rust can't guarantee references would still be valid.

---

## The `Fn` Traits

Closures automatically implement one or more of three traits depending on what they do with captured values. These traits matter when you're writing functions or types that accept closures as parameters.

### `FnOnce`
- Can be called **at most once**
- The closure moves a captured value out of its body, so it can't be called again
- All closures implement this at minimum

### `FnMut`
- Can be called **multiple times**
- Mutates captured values but doesn't move them out
- Required by things like `sort_by_key`, which calls the closure once per element

### `Fn`
- Can be called **multiple times**, even concurrently
- Doesn't move or mutate captured values — only borrows immutably

The trait a closure gets depends on what it does with captured values:
- Moves a captured value out of its body → `FnOnce` only (can't run again, value is gone)
- Mutates captured values but keeps them → `FnMut` (and `FnOnce`)
- Only reads captured values (or captures nothing) → `Fn` (and `FnMut` and `FnOnce`)

Every closure implements at least `FnOnce`. The hierarchy: `Fn` is the most restrictive (safest), `FnOnce` the least. A function that requires `FnMut` accepts anything that is `FnMut` or `Fn`. A function that requires `FnOnce` accepts any closure.

**Common example — `unwrap_or_else` takes `FnOnce`:**
```rust
// from std lib:
pub fn unwrap_or_else<F>(self, f: F) -> T
where
    F: FnOnce() -> T
```

It only calls the closure once (if the value is `None`), so `FnOnce` is enough.

**`sort_by_key` requires `FnMut`:**
```rust
list.sort_by_key(|r| r.width); // called once per element — must be FnMut
```

This fails because the closure moves a value out (making it `FnOnce`, not `FnMut`):
```rust
list.sort_by_key(|r| {
    sort_operations.push(value); // moves `value` — FnOnce only
    r.width
});
```

Fix: mutate instead of move:
```rust
list.sort_by_key(|r| {
    num_sort_operations += 1; // mutable borrow, not a move — FnMut ok
    r.width
});
```

---

## Iterators

Iterators let you process a sequence of items one at a time. In Rust they're **lazy** — no work happens until you actually consume the iterator.

```rust
let v1 = vec![1, 2, 3];
let v1_iter = v1.iter(); // nothing happens yet

for val in v1_iter {    // consumed here
    println!("{val}");
}
```

### The `Iterator` Trait

All iterators implement this trait. You only have to define `next`:

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

`next` returns `Some(item)` until the sequence is exhausted, then `None`. The iterator must be `mut` to call `next` — it changes internal state to track position. (`for` loops handle this automatically.)

### Three flavors of iteration

```rust
v.iter()       // &T — immutable references
v.iter_mut()   // &mut T — mutable references
v.into_iter()  // T — takes ownership, yields owned values
```

---

## Consuming Adapters

Methods that call `next` internally and use up the iterator. After calling one, the iterator is gone.

```rust
let v1 = vec![1, 2, 3];
let total: i32 = v1.iter().sum(); // consumes the iterator
// v1_iter is no longer usable here
assert_eq!(total, 6);
```

---

## Iterator Adapters

Methods that return a new iterator — they don't consume, they transform. Lazy until consumed.

### `map`
Applies a closure to each element:
```rust
let v2: Vec<_> = v1.iter().map(|x| x + 1).collect();
// v2 = [2, 3, 4]
```

### `filter`
Keeps elements where the closure returns `true`:
```rust
let evens: Vec<_> = v1.iter().filter(|x| *x % 2 == 0).collect();
```

You can chain adapters — they're all lazy until something consumes the chain:
```rust
v1.iter()
    .map(|x| x + 1)
    .filter(|x| x % 2 == 0)
    .collect::<Vec<_>>();
```

### `collect`
Consumes an iterator and gathers results into a collection. Requires a type annotation because it can produce different collection types:
```rust
let v2: Vec<_> = iter.collect();          // Vec
let s: HashSet<_> = iter.collect();       // HashSet
```

---

## Applying This to minigrep (Ch 13-03)

Two improvements using iterators:

### 1. Remove `clone()` in `Config::build`

Before — borrows a slice, so has to clone:
```rust
fn build(args: &[String]) -> Result<Config, &'static str> {
    let query = args[1].clone();
    let file_path = args[2].clone();
    ...
}
```

After — takes ownership of the iterator directly:
```rust
fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
    args.next(); // skip program name

    let query = match args.next() {
        Some(arg) => arg,
        None => return Err("Didn't get a query string"),
    };
    let file_path = match args.next() {
        Some(arg) => arg,
        None => return Err("Didn't get a file path"),
    };
    ...
}
```

`impl Iterator<Item = String>` means "any type that implements `Iterator` and yields `String`s." Pass `env::args()` directly from `main` instead of collecting into a `Vec` first.

### 2. Clean up `search` with iterator adapters

Before:
```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}
```

After:
```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines().filter(|line| line.contains(query)).collect()
}
```

Same result, no mutable state, clearer intent.

---

## Performance

Iterators are a **zero-cost abstraction** — they compile down to the same machine code as hand-written loops. The benchmark in the book shows `for` loop vs iterator implementations running in essentially identical time.

> "What you don't use, you don't pay for. What you do use, you couldn't hand code any better." — Bjarne Stroustrup
