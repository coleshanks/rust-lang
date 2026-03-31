# Ch 4 — Understanding Ownership

## Stack vs Heap

- **Stack** — LIFO, fixed-size data, no pointer needed (CPU always knows the top). Fast.
- **Heap** — flat address space, allocator finds a free chunk and returns a pointer. Slower (pointer lookup required).
- Stack frames are popped as a unit at scope end — no individual "free" per variable.
- Heap memory must be explicitly managed — this is what Rust's ownership solves.

## Why Heap Management Is Hard (the C problem)

Three classic bugs:
- **Memory leak** — allocate, never free. Program slowly eats memory.
- **Double free** — free the same memory twice. Can corrupt the allocator or be exploited.
- **Dangling pointer** — keep using memory after freeing it. May now point to something else entirely.

Rust's ownership system makes all three structurally impossible.

## Ownership Rules

1. Each value has an **owner**
2. Only **one owner** at a time
3. When the owner goes out of scope, the value is **dropped** (freed)

Every `}` is effectively a `drop()` call for values owned in that scope.

## String vs &str

- **String literals (`&str`)** — fixed, immutable, baked directly into the binary at compile time. Just a pointer + length into read-only memory.
- **`String`** — heap-allocated, growable, mutable. Created with `String::from("hello")`.

`String::from` is an *associated function* (called on the type, not an instance) — makes sense since it's creating the instance.

## String Memory Layout

A `String` on the stack holds three fields:
- `ptr` — heap address of the first byte (e.g. `0xF5...`)
- `len` — bytes currently in use
- `capacity` — bytes allocated (may exceed `len` to avoid frequent reallocations)

The actual character data lives on the heap, one byte per ASCII char (UTF-8 encoded).

`String::with_capacity(n)` pre-allocates `n` bytes upfront — useful when you know roughly how large a string will grow (avoids repeated reallocate-copy-free cycles).

## Move

```rust
let s1 = String::from("hello");
let s2 = s1; // s1 is moved into s2
```

- The stack metadata is copied to `s2`, but the heap data stays put
- `s1` is **invalidated** — Rust considers it no longer valid
- Only `s2` owns the heap data, so only one `drop()` happens at scope end — no double free

This is called a **move**, not a shallow copy.

## Copy (stack-only types)

```rust
let x = 5;
let y = x; // both x and y are valid
```

- Types like `i32`, `bool`, `char`, `f64`, tuples of Copy types — implement the `Copy` trait
- Copying is cheap, so Rust just duplicates the value — no ownership transfer
- Both variables remain valid after assignment

Types with `Copy` built in:
- All integers (`u32`, `i32`, etc.), floats (`f64`, etc.), `bool`, `char`
- Tuples — only if *all* elements are `Copy`. `(i32, i32)` yes, `(i32, String)` no.

A type can't implement both `Copy` and `Drop` — if a type needs cleanup on drop, Rust won't let it be freely copied everywhere.

## Clone

```rust
let s2 = s1.clone();
```

- Explicitly deep-copies the heap data — new allocation, bytes copied over
- Both `s1` and `s2` are valid, each owning separate heap data
- Has a real cost (allocation + copy) — use deliberately

## Drop timing

- Drop runs at `}` for values that go out of scope
- But also runs immediately on reassignment:
  ```rust
  let mut s = String::from("hello");
  s = String::from("ahoy"); // "hello" heap data dropped right here
  ```
- Contrast with move (`s2 = s1`) — no drop there, just ownership transfer of the same data

## Ownership and Functions

- Passing a heap value into a function **moves** ownership into that function
- The value is dropped at the function's `}` unless returned
- Returning a value moves ownership back to the caller

```rust
fn takes_ownership(s: String) { ... } // s dropped here
fn gives_back(s: String) -> String { s } // ownership transferred to caller
```

- `i32` and other `Copy` types just copy into the function — original still valid after the call

## The Tuple Workaround (pre-references)

To use a heap value in a function without losing it, you can pass it in and return it back out in a tuple — but this is clunky:

```rust
fn calculate_length(s: String) -> (String, usize) {
    let length = s.len();
    (s, length)
}
```

References (4.2) solve this properly.
