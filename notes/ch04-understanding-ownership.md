# Ch 4 — Understanding Ownership

Ownership is Rust's answer to memory management — no garbage collector, no manual malloc/free. The rules are checked at compile time with zero runtime cost.

---

## Stack vs Heap

- **Stack** — LIFO, fixed-size data, fast. The CPU always knows the top. Entire frame popped at scope end.
- **Heap** — allocator finds a free chunk and returns a pointer. Slower (pointer lookup required). Must be explicitly managed.

Stack frames are popped as a unit at scope end — no individual "free" per variable. Heap memory must be managed — this is what ownership solves.

### Why Heap Management Is Hard (the C problem)

Three classic bugs ownership makes impossible:
- **Memory leak** — allocate, never free. Program slowly eats memory.
- **Double free** — free the same memory twice. Can corrupt the allocator or be exploited.
- **Dangling pointer** — use memory after freeing it. May now point to something else entirely.

---

## Ownership Rules

1. Each value has an **owner**
2. Only **one owner** at a time
3. When the owner goes out of scope, the value is **dropped** (freed)

Every `}` is effectively a `drop()` call for values owned in that scope. Rust calls `drop` automatically — this is RAII (from C++).

---

## String vs &str

- **String literals (`&str`)** — fixed, immutable, baked into the binary at compile time. Just a pointer + length into read-only memory.
- **`String`** — heap-allocated, growable, mutable.

```rust
let s = "hello";                   // &str — immutable, stack pointer into binary
let mut s = String::from("hello"); // String — heap allocated
s.push_str(", world!");
println!("{s}"); // hello, world!
```

`String::from` is an *associated function* — called on the type, not an instance.

### String Memory Layout

A `String` on the stack holds three fields:
- `ptr` — heap address of the first byte
- `len` — bytes currently in use
- `capacity` — total bytes allocated (may exceed `len` to avoid frequent reallocations)

```rust
String::with_capacity(n)  // pre-allocate n bytes — avoids repeated reallocate-copy-free cycles
```

---

## Move

```rust
let s1 = String::from("hello");
let s2 = s1; // s1 is moved into s2 — s1 is now invalid
```

- Stack metadata (ptr, len, capacity) is copied to `s2`
- Heap data stays put — nothing is copied
- `s1` is invalidated — only `s2` owns the heap data
- Only one `drop()` happens at scope end — no double free

This is called a **move**, not a shallow copy.

```rust
println!("{s1}"); // compile error: borrow of moved value: `s1`
```

---

## Copy (stack-only types)

```rust
let x = 5;
let y = x; // both x and y are valid — no move
println!("x = {x}, y = {y}");
```

Types that implement the `Copy` trait are trivially duplicated — no ownership transfer. Copying is cheap so Rust just does it.

Types that are `Copy`:
- All integers (`u32`, `i32`, etc.), floats (`f64`, etc.), `bool`, `char`
- Tuples — only if *all* elements are `Copy`. `(i32, i32)` yes, `(i32, String)` no.

A type can't implement both `Copy` and `Drop` — if a type needs cleanup, it can't be freely copied.

---

## Clone

```rust
let s1 = String::from("hello");
let s2 = s1.clone(); // deep copy — new heap allocation, bytes copied

println!("s1 = {s1}, s2 = {s2}"); // both valid, both own separate data
```

Has a real cost (allocation + copy) — seeing `.clone()` is a signal that something expensive may be happening.

---

## Drop Timing

Drop runs at `}` for values that go out of scope. Also runs immediately on reassignment:

```rust
let mut s = String::from("hello");
s = String::from("ahoy"); // "hello" heap data dropped right here
println!("{s}"); // ahoy
```

Contrast with move (`s2 = s1`) — no drop there, just an ownership transfer of the same heap data.

---

## Ownership and Functions

Passing a heap value into a function **moves** it. The value is dropped at the function's `}` unless returned.

```rust
fn main() {
    let s = String::from("hello");
    takes_ownership(s);       // s moved in — no longer valid here

    let x = 5;
    makes_copy(x);            // x copied in — still valid here
    println!("{x}");
}

fn takes_ownership(s: String) {
    println!("{s}");
}  // s dropped here

fn makes_copy(n: i32) {
    println!("{n}");
}  // n dropped, but it's just a stack integer — no heap to free
```

Returning a value moves ownership back to the caller:

```rust
fn gives_back() -> String {
    let s = String::from("yours");
    s  // moved out to caller
}
```

### The Tuple Workaround (pre-references)

To use a heap value in a function without losing it, pass it in and return it back — but this is clunky:

```rust
fn calculate_length(s: String) -> (String, usize) {
    let length = s.len();
    (s, length)  // hand ownership back with the result
}
```

References solve this properly.

---

## References and Borrowing

`&` creates a reference — use a value without taking ownership. Called **borrowing**. When the reference goes out of scope, the value isn't dropped (you don't own it).

```rust
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);  // lend s1, keep ownership
    println!("'{s1}' has length {len}");
}

fn calculate_length(s: &String) -> usize {
    s.len()
}  // s goes out of scope — nothing dropped, we never owned it
```

### Immutable References

Default. Read-only. As many `&T` as you want at the same time.

```rust
fn change(s: &String) {
    s.push_str(", world"); // compile error: cannot borrow as mutable
}
```

### Mutable References (`&mut`)

Lets you modify borrowed data. **Only one `&mut` can exist at a time** — and no other references (`&` or `&mut`) can overlap while it's active.

```rust
fn main() {
    let mut s = String::from("hello");
    change(&mut s);
    println!("{s}"); // hello, world
}

fn change(s: &mut String) {
    s.push_str(", world");
}
```

Two `&mut` at once is a compile error:

```rust
let mut s = String::from("hello");
let r1 = &mut s;
let r2 = &mut s; // error: cannot borrow `s` as mutable more than once
println!("{r1}, {r2}");
```

Can't mix `&` and `&mut` that overlap in usage:

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;
let r3 = &mut s; // error: can't borrow mutably while immutable borrows are active
println!("{r1}, {r2}, {r3}");
```

But this is fine — the immutable borrows' last use is before the mutable borrow:

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{r1} and {r2}"); // r1, r2 done here

let r3 = &mut s;           // fine — r1 and r2 are no longer in use
println!("{r3}");
```

Rust tracks the **last point of use**, not scope end — called Non-Lexical Lifetimes (NLL). A reference's lifetime ends at its last usage, not at the closing `}`.

### Dangling References

Rust prevents returning a reference to a local variable — the data would be dropped before the reference is used:

```rust
fn dangle() -> &String {   // compile error: missing lifetime specifier
    let s = String::from("hello");
    &s   // s is dropped when this function returns — reference would be invalid
}
```

Solution: return the owned value directly:

```rust
fn no_dangle() -> String {
    let s = String::from("hello");
    s   // ownership moved to caller — valid
}
```

### Reference Rules

1. Any number of immutable references (`&T`) **or** exactly one mutable reference (`&mut T`) — never both at once
2. References must always point to valid data

---

## Slices

A slice is a reference to a contiguous sequence of elements — no ownership.

```rust
let s = String::from("hello world");
let hello = &s[0..5];  // "hello"
let world = &s[6..11]; // "world"
```

`[start..end]` — exclusive on the right. `[0..5]` gives indices 0–4.

Shortcuts:
```rust
&s[..5]  // from start to 5 (exclusive)
&s[3..]  // from 3 to end
&s[..]   // whole string — same as &s but type &str
```

### Why Slices Over Indices

Returning a bare `usize` index is fragile — if the string is modified, the index is stale and Rust can't help:

```rust
fn first_word(s: &String) -> usize {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' { return i; }
    }
    s.len()
}

fn main() {
    let mut s = String::from("hello world");
    let word = first_word(&s); // word = 5
    s.clear();                 // s is now ""
    // word is still 5 — meaningless, and Rust didn't catch it
}
```

With a slice, the borrow checker catches it:

```rust
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' { return &s[0..i]; }
    }
    &s[..]
}

fn main() {
    let mut s = String::from("hello world");
    let word = first_word(&s);
    s.clear(); // compile error: can't mutably borrow while `word` (immutable borrow) is in use
    println!("{word}");
}
```

The returned `&str` is tied to `s` — the borrow checker enforces that `s` can't be modified while `word` is alive.

### `&str` vs `&String` in Parameters

Prefer `&str` — works with both `String` (via `&s` or `&s[..]`) and string literals. No reason to restrict to `&String`.

```rust
fn first_word(s: &str) -> &str { ... }

let my_string = String::from("hello world");
first_word(&my_string);      // &String coerces to &str automatically
first_word(&my_string[..]);  // explicit slice
first_word("hello world");   // string literals are already &str
```

### Array Slices

Same idea, different type:

```rust
let a = [1, 2, 3, 4, 5];
let slice = &a[1..3]; // type: &[i32]

assert_eq!(slice, &[2, 3]);
```

---

## When to Pass What

| Signature | Use when |
|---|---|
| `fn f(s: String)` | function should own and consume the value |
| `fn f(s: &String)` | borrow for reading (but prefer `&str`) |
| `fn f(s: &str)` | borrow a string for reading — accepts both `String` and literals |
| `fn f(s: &mut String)` | borrow for modification without taking ownership |
