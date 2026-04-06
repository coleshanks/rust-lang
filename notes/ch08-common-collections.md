# Ch 8 — Common Collections

Collections store multiple values on the heap — size can grow or shrink at runtime, unlike arrays.

---

## Vectors — `Vec<T>`

Stores multiple values of the same type in contiguous memory. `T` is a generic placeholder — filled in by annotation or inference (full generics in ch10).

```rust
let v: Vec<i32> = Vec::new();   // explicit type needed, empty
let v = vec![1, 2, 3];          // type inferred from values
```

**Adding elements:**
```rust
let mut v = Vec::new();
v.push(5);
v.push(6);
```

**Reading elements — two ways:**
```rust
let third: &i32 = &v[2];           // panics if out of bounds
let third: Option<&i32> = v.get(2); // returns None if out of bounds
```

Use `v[i]` when you're certain the index is valid. Use `.get()` when it might not be — lets you handle the missing case with `match` instead of crashing.

**Borrow checker gotcha:**
```rust
let first = &v[0];  // immutable borrow
v.push(6);          // ERROR — mutable borrow while immutable borrow active
```
`push` might reallocate the whole vector if it's out of capacity, which would invalidate `first`. Borrow checker catches this. Fix: don't hold a reference across a `push`.

**Iterating:**
```rust
for i in &v {
    println!("{i}");
}

for i in &mut v {
    *i += 50;  // dereference needed to modify
}
```

**Storing multiple types with enums:**

A vec must be one type — but that type can be an enum whose variants carry different data:

```rust
enum SpreadsheetCell {
    Int(i32),
    Float(f64),
    Text(String),
}

let row = vec![
    SpreadsheetCell::Int(3),
    SpreadsheetCell::Text(String::from("blue")),
    SpreadsheetCell::Float(10.12),
];
```

The vec is `Vec<SpreadsheetCell>` — all the same type. The variety lives inside the enum.

Vectors are dropped and all elements freed when they go out of scope.

---

## Strings

Two string types:
- `&str` — string slice, borrowed, usually a literal baked into the binary
- `String` — owned, heap-allocated, growable

```rust
let s = String::new();
let s = String::from("hello");
let s = "hello".to_string();
```

**Updating:**
```rust
let mut s = String::from("foo");
s.push_str("bar");   // append a string slice
s.push('!');         // append a single char
```

**Concatenation:**
```rust
let s3 = s1 + &s2;  // s1 is moved, s2 is borrowed
```

For combining more than two strings, `format!` is cleaner and doesn't take ownership:
```rust
let s = format!("{s1}-{s2}-{s3}");
```

**Why you can't index strings (`s[0]`):**

Rust strings are UTF-8. Characters can be 1–4 bytes — a byte index doesn't map cleanly to a character. Rust refuses to guess what you mean, so indexing is not allowed.

Three ways to look at the same string data:
- **Bytes** — raw numbers
- **Scalar values** (`char`) — Unicode code points, what `.chars()` gives you
- **Grapheme clusters** — what a human would call "letters" (requires an external crate)

```rust
for c in "Зд".chars() { println!("{c}"); }   // З, д
for b in "Зд".bytes() { println!("{b}"); }   // 208, 151, 208, 180
```

Slicing by byte range works but panics if you cut across a character boundary:
```rust
let s = &"Здравствуйте"[0..4]; // "Зд" — each Cyrillic char is 2 bytes
```

Strings are more complex than they look — Rust forces you to deal with that upfront.

---

## Hash Maps — `HashMap<K, V>`

Key-value store. Same concept as lookup tables, dictionaries, objects in other languages. Not in the prelude — must `use`.

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();
scores.insert(String::from("Blue"), 10);
scores.insert(String::from("Yellow"), 50);
```

All keys must be the same type, all values must be the same type.

**Accessing:**
```rust
let score = scores.get(&team_name).copied().unwrap_or(0);
// .get() returns Option<&V>
// .copied() turns Option<&i32> into Option<i32>
// .unwrap_or(0) returns the value or 0 if None
```

**Iterating — no guaranteed order:**
```rust
for (key, value) in &scores {
    println!("{key}: {value}");
}
```

**Ownership:** `String` keys/values are moved into the map. Copy types (`i32`) are copied.

**Updating:**
```rust
// Overwrite
scores.insert(String::from("Blue"), 25);

// Insert only if key doesn't exist
scores.entry(String::from("Blue")).or_insert(50);

// Update based on existing value
let count = map.entry(word).or_insert(0);
*count += 1;
```

`entry().or_insert()` returns a mutable reference to the value — dereference with `*` to modify it.

Default hasher is SipHash — DoS resistant, not the fastest. Can swap in a different hasher if performance matters.
