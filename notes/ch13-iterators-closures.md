# Ch 13 — Closures and Iterators

Two closely related features borrowed from functional programming. Both are zero-cost abstractions — expressive, high-level code that compiles down to the same thing as hand-written low-level code.

---

## Closures

Anonymous functions that can capture values from the scope they're defined in. Unlike regular functions, closures have access to variables outside their own body.

```rust
let only_borrows = || println!("{list:?}");
```

`||` is the closure syntax — parameters go inside the pipes. Body follows, either as a single expression or a `{}` block.

A real example — `unwrap_or_else` takes a closure that captures `self`:

```rust
impl Inventory {
    fn giveaway(&self, user_preference: Option<ShirtColor>) -> ShirtColor {
        user_preference.unwrap_or_else(|| self.most_stocked())
    }
}
```

The closure `|| self.most_stocked()` captures an immutable reference to `self`. A regular function couldn't do this — closures are unique in that they can close over their environment.

### Type Inference

Closures don't require type annotations — the compiler infers them from context:

```rust
fn  add_one_v1   (x: u32) -> u32 { x + 1 }  // function
let add_one_v2 = |x: u32| -> u32 { x + 1 };  // closure, annotated
let add_one_v3 = |x|             { x + 1 };  // closure, inferred
let add_one_v4 = |x|               x + 1  ;  // closure, single expression
```

Once a closure is called with a specific type, that type is locked in. Calling it with a different type afterwards is a compile error:

```rust
let example_closure = |x| x;
let s = example_closure(String::from("hello")); // type locked to String
let n = example_closure(5);                     // compile error: expected String
```

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
println!("{list:?}"); // ok after closure is done — [1, 2, 3, 7]
```

Can't use `list` between defining and calling `borrows_mutably` — the mutable borrow is active during that window.

**Move** — force the closure to take ownership, usually for threading:
```rust
use std::thread;

let list = vec![1, 2, 3];
thread::spawn(move || println!("{list:?}")).join().unwrap();
```

`move` is required when passing a closure to a new thread because the thread might outlive the current scope — Rust can't guarantee borrows would still be valid.

---

## The `Fn` Traits

Closures automatically implement one or more of three traits depending on what they do with captured values. These matter when writing functions or types that accept closures as parameters.

### `FnOnce`
- Can be called **at most once**
- The closure moves a captured value out of its body — it's gone after the first call
- All closures implement this at minimum

### `FnMut`
- Can be called **multiple times**
- Mutates captured values but doesn't move them out
- Required by things like `sort_by_key`, which calls the closure once per element

### `Fn`
- Can be called **multiple times**, even concurrently
- Doesn't move or mutate — only borrows immutably (or captures nothing)

The trait a closure gets depends on what it does with captured values:
- Moves a captured value out → `FnOnce` only
- Mutates captured values → `FnMut` (also `FnOnce`)
- Only reads, or captures nothing → `Fn` (also `FnMut` and `FnOnce`)

Hierarchy: `Fn ⊂ FnMut ⊂ FnOnce`. A function requiring `FnMut` accepts `Fn` or `FnMut`. A function requiring `FnOnce` accepts any closure.

**`unwrap_or_else` takes `FnOnce`** — only called once if `None`:
```rust
pub fn unwrap_or_else<F>(self, f: F) -> T
where
    F: FnOnce() -> T
```

**`sort_by_key` requires `FnMut`** — called once per element:
```rust
list.sort_by_key(|r| r.width);
```

This fails — closure moves a value out, making it `FnOnce` only:
```rust
list.sort_by_key(|r| {
    sort_operations.push(value); // moves `value` — FnOnce only, not FnMut
    r.width
});
```

Fix — mutate instead of move:
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
    // all other methods have default implementations built on next
}
```

`next` returns `Some(item)` until the sequence is exhausted, then `None`. The iterator must be `mut` to call `next` directly — it changes internal state to track position. (`for` loops handle this automatically.)

```rust
let mut v1_iter = v1.iter();
assert_eq!(v1_iter.next(), Some(&1));
assert_eq!(v1_iter.next(), Some(&2));
assert_eq!(v1_iter.next(), Some(&3));
assert_eq!(v1_iter.next(), None);
```

### Three Flavors of Iteration

```rust
v.iter()       // yields &T — immutable references, v still usable after
v.iter_mut()   // yields &mut T — mutable references
v.into_iter()  // yields T — takes ownership, v consumed
```

---

## Consuming Adapters

Methods that call `next` internally and use up the iterator. After calling one, the iterator is gone.

```rust
let v1 = vec![1, 2, 3];
let total: i32 = v1.iter().sum(); // consumes the iterator — v1_iter unusable after
assert_eq!(total, 6);
```

Other consuming adapters: `count()`, `last()`, `max()`, `min()`, `any()`, `all()`, `find()`, `position()`.

---

## Iterator Adapters

Methods that return a new iterator — transform without consuming. Lazy until something downstream consumes the chain.

### `map`
Applies a closure to each element:
```rust
let v2: Vec<_> = vec![1, 2, 3].iter().map(|x| x + 1).collect();
// v2 = [2, 3, 4]
```

### `filter`
Keeps elements where the closure returns `true`. Note: `iter()` yields `&&T` when chained, so you often need to dereference:
```rust
let v = vec![1, 2, 3, 4, 5];
let evens: Vec<_> = v.iter().filter(|x| *x % 2 == 0).collect(); // *x deref
// or destructure in the pattern:
let evens: Vec<_> = v.iter().filter(|&x| x % 2 == 0).collect(); // &x pattern
```

### `enumerate`
Wraps each item with its index, yielding `(usize, item)` tuples:
```rust
for (i, val) in v.iter().enumerate() {
    println!("{i}: {val}");
}
```

### `zip`
Pairs two iterators together, yielding tuples until either runs out:
```rust
let a = vec![1, 2, 3];
let b = vec!["one", "two", "three"];
let paired: Vec<_> = a.iter().zip(b.iter()).collect();
// [(1, "one"), (2, "two"), (3, "three")]
```

### `collect`
Consumes an iterator and gathers results into a collection. Needs a type annotation — can produce different collection types:
```rust
let v2: Vec<_> = iter.collect();
let s: HashSet<_> = iter.collect();

// or with turbofish:
let v2 = iter.collect::<Vec<_>>();
```

### Chaining

All lazy until consumed:
```rust
let result: Vec<_> = vec![1, 2, 3, 4, 5]
    .iter()
    .map(|x| x * 2)
    .filter(|x| x > &4)
    .collect();
// [6, 8, 10]
```

---

## Creating a Custom Iterator

Implement the `Iterator` trait by defining `next`. Everything else (map, filter, etc.) comes for free as default methods:

```rust
struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

// all iterator methods now work:
let sum: u32 = Counter::new()
    .zip(Counter::new().skip(1))  // pair (1,2), (2,3), (3,4), (4,5)
    .map(|(a, b)| a * b)          // 2, 6, 12, 20
    .filter(|x| x % 3 == 0)      // 6, 12
    .sum();                       // 18
```

---

## Applying This to minigrep

### 1. Remove `clone()` in `Config::build`

Before — borrows a slice, so has to clone:
```rust
fn build(args: &[String]) -> Result<Config, &'static str> {
    let query = args[1].clone();
    let file_path = args[2].clone();
    ...
}
```

After — takes ownership of the iterator directly, no cloning needed:
```rust
fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
    args.next(); // skip binary name

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

`impl Iterator<Item = String>` — any type implementing `Iterator` that yields `String`s. Pass `env::args()` directly instead of collecting into a `Vec` first.

### 2. Clean up `search` with iterator adapters

Before — mutable accumulator:
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

After — no mutable state, clear intent:
```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines().filter(|line| line.contains(query)).collect()
}
```

---

## Performance

Iterators are a **zero-cost abstraction** — they compile down to the same machine code as hand-written loops. No overhead from the abstraction at runtime.

> "What you don't use, you don't pay for. What you do use, you couldn't hand code any better." — Bjarne Stroustrup
