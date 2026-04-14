# Ch 8 — Common Collections

Collections store multiple values on the heap — size can grow or shrink at runtime, unlike arrays. The three most common: `Vec<T>`, `String`, and `HashMap<K, V>`.

---

## Vectors — `Vec<T>`

Stores multiple values of the same type in contiguous memory. `T` is a generic placeholder filled in by annotation or inference.

```rust
let v: Vec<i32> = Vec::new();  // explicit type needed — no initial values to infer from
let v = vec![1, 2, 3];         // type inferred from values
```

**Adding elements:**
```rust
let mut v = Vec::new();
v.push(5);
v.push(6);
```

Pre-allocate capacity when you know roughly how large it'll get (avoids repeated reallocation):
```rust
let mut v = Vec::with_capacity(10);
```

**Reading elements — two ways:**
```rust
let v = vec![1, 2, 3, 4, 5];

let third: &i32 = &v[2];           // panics if out of bounds
let third: Option<&i32> = v.get(2); // returns None if out of bounds

// out of bounds behavior:
let x = &v[100];    // panics at runtime
let x = v.get(100); // returns None — safe to match on
```

Use `v[i]` when you're certain the index is valid. Use `.get()` when it might not be.

**Borrow checker gotcha:**
```rust
let mut v = vec![1, 2, 3, 4, 5];
let first = &v[0];  // immutable borrow of v

v.push(6);          // compile error: cannot borrow `v` as mutable
                    // because it is also borrowed as immutable

println!("{first}");
```

`push` may reallocate the entire vector if it's out of capacity, which would invalidate `first`. The borrow checker catches this at compile time. Fix: don't hold a reference across a `push`.

**Iterating:**
```rust
let v = vec![100, 32, 57];

for i in &v {
    println!("{i}");  // immutable — read only
}

let mut v = vec![100, 32, 57];

for i in &mut v {
    *i += 50;  // dereference required to modify the value
}
```

**Storing multiple types with enums:**

A vec must hold one type — but that type can be an enum whose variants carry different data:

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

The vec is `Vec<SpreadsheetCell>` — all the same type. The variety lives inside the enum variants.

Vectors are dropped and all elements freed when they go out of scope.

---

## Strings

Two string types:
- `&str` — string slice, borrowed, usually a literal baked into the binary
- `String` — owned, heap-allocated, growable

```rust
let s = String::new();             // empty String
let s = String::from("hello");     // from a literal
let s = "hello".to_string();       // same — to_string() works on any type implementing Display
```

**Updating:**
```rust
let mut s = String::from("foo");
s.push_str("bar"); // appends a &str — doesn't take ownership of "bar"
s.push('!');       // appends a single char (single quotes)
```

`push_str` takes a `&str` so the argument remains valid after the call:
```rust
let mut s1 = String::from("foo");
let s2 = "bar";
s1.push_str(s2);
println!("{s2}"); // s2 still valid — push_str didn't take ownership
```

**Concatenation with `+`:**
```rust
let s1 = String::from("Hello, ");
let s2 = String::from("world!");
let s3 = s1 + &s2; // s1 is moved here — can no longer use s1
```

The `+` operator uses this signature under the hood:
```rust
fn add(self, s: &str) -> String
```

`self` means `s1` is consumed (moved in). `&s2` is a `&String` that Rust coerces to `&str` via deref coercion. The result is a new `String` owned by `s3`. Looks like a copy, actually a move + append.

For combining more than two strings, `format!` is cleaner and doesn't take ownership of anything:
```rust
let s1 = String::from("tic");
let s2 = String::from("tac");
let s3 = String::from("toe");

let s = format!("{s1}-{s2}-{s3}"); // returns a String, all three still valid
```

**Why you can't index strings (`s[0]`):**

Rust strings are UTF-8. Characters are 1–4 bytes — a byte index doesn't map cleanly to a character. Rust refuses to guess what you mean.

```rust
let s1 = String::from("hello");
let h = s1[0]; // compile error: `String` cannot be indexed by `{integer}`
```

Three ways to view the same string data:
- **Bytes** — raw `u8` values
- **Scalar values** (`char`) — Unicode code points, what `.chars()` gives you
- **Grapheme clusters** — what a human sees as "letters" (external crate needed)

```rust
for c in "Зд".chars() { println!("{c}"); }  // З then д
for b in "Зд".bytes() { println!("{b}"); }  // 208, 151, 208, 180
```

Slicing by byte range works but panics if you cut across a character boundary:
```rust
let s = &"Здравствуйте"[0..4]; // "Зд" — each Cyrillic char is 2 bytes
let s = &"Здравствуйте"[0..1]; // panic: byte index 1 is not a char boundary
```

Useful string methods:
```rust
s.contains("world")           // bool
s.replace("foo", "bar")       // returns new String
s.split_whitespace()          // iterator over words
s.trim()                      // strip leading/trailing whitespace
```

Strings are more complex than they look — Rust makes you deal with that upfront rather than silently producing wrong results.

---

## Hash Maps — `HashMap<K, V>`

Key-value store. Not in the prelude — must `use`. No built-in macro like `vec!`.

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();
scores.insert(String::from("Blue"), 10);
scores.insert(String::from("Yellow"), 50);
```

All keys must be the same type, all values must be the same type.

**Accessing:**
```rust
let team_name = String::from("Blue");
let score = scores.get(&team_name).copied().unwrap_or(0);
// .get()        → Option<&V>
// .copied()     → Option<V>  (for Copy types like i32)
// .unwrap_or(0) → V, defaulting to 0 if key not found
```

**Iterating — order not guaranteed:**
```rust
for (key, value) in &scores {
    println!("{key}: {value}");
}
```

**Ownership:** `String` keys and values are moved into the map — they can't be used after `insert`. `Copy` types like `i32` are copied in, originals remain valid.

```rust
let field_name = String::from("color");
let field_value = String::from("blue");
map.insert(field_name, field_value);
// field_name and field_value are no longer valid here
```

**Updating:**
```rust
// Overwrite existing value
scores.insert(String::from("Blue"), 25);

// Insert only if key doesn't already exist
scores.entry(String::from("Blue")).or_insert(50);
// .entry() returns an Entry enum
// .or_insert() inserts the value if absent, returns &mut V either way

// Update based on existing value — classic word count:
let text = "hello world wonderful world";
let mut map: HashMap<&str, i32> = HashMap::new();

for word in text.split_whitespace() {
    let count = map.entry(word).or_insert(0); // &mut i32
    *count += 1;                               // dereference to modify
}

println!("{map:?}"); // {"hello": 1, "world": 2, "wonderful": 1}
```

`or_insert` returns a `&mut V` — you need `*` to get through the reference and modify the actual value. The mutable reference goes out of scope at the end of each loop iteration, so the borrow checker is happy.

**Hashing:** default hasher is SipHash — DoS resistant, not the fastest. Can swap in a different hasher implementing `BuildHasher` if performance matters (several available on crates.io).
