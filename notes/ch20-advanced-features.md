# Ch 20 — Advanced Features

A grab-bag of Rust features that don't come up every day but matter a lot when they do. Five topics: unsafe Rust, advanced traits, advanced types, advanced functions/closures, and macros.

---

## 20.1 — Unsafe Rust

The compiler's safety guarantees are conservative. Sometimes you know something is safe that the compiler can't prove. `unsafe` is the escape hatch — you're telling the compiler "trust me, I'll uphold the invariants myself."

`unsafe` doesn't turn off the borrow checker. It only unlocks five specific superpowers:

1. Dereference a raw pointer
2. Call an unsafe function or method
3. Access or modify a mutable static variable
4. Implement an unsafe trait
5. Access fields of a union

### Raw pointers

Two types: `*const T` (immutable) and `*mut T` (mutable). Raw pointers:
- Can ignore borrowing rules (multiple `*mut` to same memory is allowed)
- Aren't guaranteed to point to valid memory
- Can be null
- Don't auto-drop

Creating them is safe. Dereferencing is not.

```rust
let mut num = 5;

let r1 = &raw const num; // *const i32
let r2 = &raw mut num;   // *mut i32

unsafe {
    println!("r1: {}", *r1);
    println!("r2: {}", *r2);
}
```

`&raw const` / `&raw mut` is the modern syntax for creating raw pointers (Rust 2024). You can also cast an arbitrary address — that's where real danger lives:

```rust
let address = 0x012345usize;
let r = address as *const i32; // compiles, but dereferencing is UB
```

### Safe abstractions over unsafe code

The key pattern: use `unsafe` internally but expose a safe public API. The canonical example is `split_at_mut` — the borrow checker can't verify that two slices from the same vec don't overlap, but we know they don't:

```rust
use std::slice;

fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    let ptr = values.as_mut_ptr(); // raw pointer to start of slice

    assert!(mid <= len); // safety check — this is what makes the unsafe below OK

    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),           // [0..mid]
            slice::from_raw_parts_mut(ptr.add(mid), len - mid), // [mid..len]
        )
    }
}
```

The safe version (`&mut values[..mid], &mut values[mid..]`) doesn't compile — the borrow checker sees two mutable borrows of `values` and rejects it, even though they don't actually overlap. The unsafe version works because we're operating on raw pointers instead of references.

The `assert!` is load-bearing — it's what makes this actually safe. Without it, `ptr.add(mid)` could go out of bounds.

### `extern` and FFI

To call C functions from Rust:

```rust
unsafe extern "C" {
    fn abs(input: i32) -> i32;
}

fn main() {
    unsafe {
        println!("{}", abs(-3)); // 3
    }
}
```

`"C"` is the ABI — how the function call is laid out at the binary level. C is the most common. The `extern` block declares the signature; the actual implementation comes from a linked C library.

If you've verified a C function is safe to call, you can mark it `safe` inside the block and drop the `unsafe` at the call site:

```rust
unsafe extern "C" {
    safe fn abs(input: i32) -> i32; // Rust trusts you on this
}

fn main() {
    println!("{}", abs(-3)); // no unsafe needed
}
```

To expose a Rust function to C:

```rust
#[unsafe(no_mangle)]       // don't mangle the name — C needs to find it by exact name
pub extern "C" fn call_from_c() {
    println!("called from C!");
}
```

`no_mangle` is `unsafe` because name mangling exists for a reason (avoiding symbol conflicts). You're opting out of that guarantee.

### Mutable static variables

Constants are inlined everywhere. Statics have a fixed memory address — the same address every time. Accessing a mutable static is unsafe because there's no protection against data races:

```rust
static mut COUNTER: u32 = 0;

/// SAFETY: must only be called from a single thread at a time
unsafe fn add_to_count(inc: u32) {
    unsafe { COUNTER += inc; }
}

fn main() {
    unsafe {
        // SAFETY: single-threaded
        add_to_count(3);
        println!("COUNTER: {}", *(&raw const COUNTER));
    }
}
```

The `SAFETY:` comment convention is important — document exactly what invariants the caller must uphold for the unsafe code to actually be safe.

### Unsafe traits

A trait is `unsafe` when at least one of its methods has an invariant the compiler can't verify. `Send` and `Sync` are the key examples — they're automatically derived for types composed of `Send`/`Sync` parts, but if your type has a raw pointer, you need to opt in manually:

```rust
struct MyType(*mut u8); // raw pointer — Send/Sync not auto-derived

unsafe impl Send for MyType {} // you're asserting: yes, safe to send across threads
unsafe impl Sync for MyType {} // you're asserting: yes, safe to share across threads
```

### Unions

Like a struct but all fields share the same memory — only one is valid at a time. Mostly used for FFI with C unions. Accessing a field is unsafe because Rust doesn't track which field is currently active:

```rust
union MyUnion {
    f1: u32,
    f2: f32,
}

let u = MyUnion { f1: 1 };
unsafe {
    println!("{}", u.f1); // fine — we initialized f1
    // println!("{}", u.f2); // UB — would reinterpret the bytes as f32
}
```

### Miri

Miri is a Rust tool that interprets your program and catches undefined behavior at runtime — use-after-free, invalid pointer reads, uninitialized memory, etc. It's a dynamic tool (only catches bugs in code paths that actually run), not a static checker.

```bash
rustup +nightly component add miri
cargo +nightly miri run
cargo +nightly miri test
```

Good practice: run Miri on any crate that uses `unsafe`. It won't catch everything, but it catches a lot.

**Embedded note:** Raw pointers and `unsafe` are central to embedded Rust. Memory-mapped I/O registers are accessed via raw pointers to specific hardware addresses. Embassy abstracts most of this away, but understanding it matters for lower-level work on the Pico.

---

## 20.2 — Advanced Traits

### Associated types

Associated types attach a type placeholder to a trait. The implementor specifies the concrete type once; the trait's methods can then use it without any type annotations at the call site.

```rust
pub trait Iterator {
    type Item; // placeholder — each implementor fills this in

    fn next(&mut self) -> Option<Self::Item>; // uses the placeholder
}

impl Iterator for Counter {
    type Item = u32; // concrete type — Counter produces u32s

    fn next(&mut self) -> Option<Self::Item> { ... }
}
```

**Associated types vs generics:** if `Iterator` used a generic (`Iterator<T>`), you could implement it multiple times for the same type — `impl Iterator<u32> for Counter`, `impl Iterator<String> for Counter`, etc. Then `counter.next()` would be ambiguous. With an associated type, there's only one `Item` per implementor — `counter.next()` unambiguously returns `Option<u32>`.

Use associated types when there should be exactly one implementation per type. Use generics when multiple implementations make sense.

### Default type parameters and operator overloading

Generic type parameters can have defaults: `<T=SomeDefault>`. The most common use is operator overloading via `std::ops`:

```rust
// Add trait from std — Rhs defaults to Self
trait Add<Rhs=Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}

// Adding a Point to a Point — uses default Rhs=Self
impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

// Adding Meters to Millimeters — custom Rhs
impl Add<Meters> for Millimeters {
    type Output = Millimeters;
    fn add(self, other: Meters) -> Millimeters {
        Millimeters(self.0 + (other.0 * 1000))
    }
}
```

Default type params have two use cases:
1. Extend a type without breaking existing code — add a new type param with a default, old impls don't need to change
2. Allow customization without requiring it — `Add<Rhs=Self>` works for the common case, custom `Rhs` available when you need it

### Disambiguating methods with the same name

Multiple traits can define methods with the same name. Rust doesn't error — it just needs you to be explicit when calling them:

```rust
trait Pilot  { fn fly(&self); }
trait Wizard { fn fly(&self); }

struct Human;

impl Pilot  for Human { fn fly(&self) { println!("captain speaking"); } }
impl Wizard for Human { fn fly(&self) { println!("Up!"); } }
impl Human            { fn fly(&self) { println!("*waving arms*"); } }
```

```rust
let person = Human;
person.fly();           // Human's own method — default
Pilot::fly(&person);    // Pilot's fly — explicit trait + self passed manually
Wizard::fly(&person);   // Wizard's fly
```

For associated functions (no `self`), the syntax is more explicit — **fully qualified syntax**:

```rust
trait Animal { fn baby_name() -> String; }
struct Dog;

impl Dog    { fn baby_name() -> String { String::from("Spot") } }
impl Animal for Dog { fn baby_name() -> String { String::from("puppy") } }

Dog::baby_name()              // "Spot" — Dog's own function
<Dog as Animal>::baby_name()  // "puppy" — Animal's function
```

`<Type as Trait>::function(args)` — the full form. Use it any time the compiler can't figure out which version you want.

### Supertraits

A trait that requires another trait to also be implemented. Use `: OtherTrait` in the trait definition:

```rust
use std::fmt;

trait OutlinePrint: fmt::Display { // requires Display
    fn outline_print(&self) {
        let output = self.to_string(); // can call to_string() because Display is guaranteed
        let len = output.len();
        println!("{}", "*".repeat(len + 4));
        println!("* {output} *");
        println!("{}", "*".repeat(len + 4));
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl OutlinePrint for Point {} // no extra code needed — outline_print has a default body
```

If you try `impl OutlinePrint for Point` without `impl Display for Point`, the compiler errors. Supertraits are a compile-time guarantee: within `OutlinePrint`'s default methods, you can freely call any `Display` method.

### Newtype pattern (with traits)

The **orphan rule**: you can only implement a trait for a type if either the trait or the type is defined in your crate. You can't implement `Display` for `Vec<String>` — both are external.

Workaround: wrap the type in a new struct (a **newtype**) and implement the trait on the wrapper:

```rust
use std::fmt;

struct Wrapper(Vec<String>); // tuple struct — zero cost at runtime

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", ")) // self.0 accesses the inner Vec
    }
}

let w = Wrapper(vec![String::from("hello"), String::from("world")]);
println!("w = {w}"); // w = [hello, world]
```

The wrapper is erased at compile time — no runtime cost. The downside: `Wrapper` doesn't automatically have `Vec`'s methods. You either implement them manually or implement `Deref` to get transparent access.

---

## 20.3 — Advanced Types

### Newtype pattern (for type safety and abstraction)

Two more uses beyond the trait orphan workaround:

**Type safety:** distinct types prevent accidental mixing.

```rust
struct Meters(u32);
struct Millimeters(u32);

fn take_meters(m: Meters) { ... }

take_meters(Millimeters(5)); // compile error — wrong type
take_meters(5);              // compile error — not a Meters
```

**Abstraction:** hide implementation details behind a public API.

```rust
struct People(HashMap<i32, String>);

impl People {
    pub fn add(&mut self, name: String) { ... } // public
    // internal: assigns i32 IDs — users don't know or care
}
```

### Type aliases

`type` creates a synonym for an existing type — not a new type. The compiler treats them identically; no extra type safety.

```rust
type Kilometers = i32;

let x: i32 = 5;
let y: Kilometers = 5;
println!("{}", x + y); // fine — same type under the hood
```

The real use: cutting down repetition for long types.

```rust
type Thunk = Box<dyn Fn() + Send + 'static>;

fn takes_long_type(f: Thunk) { ... }
fn returns_long_type() -> Thunk { Box::new(|| ()) }
```

Standard library uses this pattern for `Result` — `std::io` defines `type Result<T> = std::result::Result<T, std::io::Error>` so every I/O function can write `Result<usize>` instead of `Result<usize, std::io::Error>`.

### The never type `!`

`!` is the return type for functions that never return. Called the **never type** or **empty type** — you can't actually create a value of type `!`.

```rust
fn bar() -> ! {
    panic!("never returns");
}
```

Why it matters in practice: `!` coerces into any other type. This is what makes these valid:

```rust
// In a match, all arms must have the same type.
// continue has type !, which coerces to u32, so the match resolves to u32.
let guess: u32 = match guess.trim().parse() {
    Ok(num) => num,
    Err(_) => continue, // ! coerces to u32
};

// Same for panic! — used in unwrap:
match self {
    Some(val) => val,   // T
    None => panic!(...), // ! coerces to T
}

// An infinite loop with no break has type !
loop {
    // runs forever — never produces a value
}
```

The key insight: `!` can become any type precisely because there's no code path that actually produces a value of `!`. The coercion is safe because that branch never actually returns.

### Dynamically sized types (DSTs)

Most types have a size known at compile time. **Dynamically sized types** (DSTs) don't — their size is only known at runtime.

`str` is the canonical DST. You can't have a bare `str` variable:

```rust
let s1: str = "hello"; // error — how many bytes? compiler doesn't know at compile time
```

You always use `str` behind a pointer — most commonly `&str`, which is a fat pointer: (address, length).

```rust
let s: &str = "hello"; // two words: pointer + length — fixed size, safe
```

Same applies to trait objects:

```rust
let d: dyn Draw = ...; // error — unknown size
let d: &dyn Draw = ...; // fat pointer: (data pointer, vtable pointer) — fixed size
let d: Box<dyn Draw> = ...; // also fine — Box is a fixed-size pointer
```

**Golden rule: DSTs must always live behind a pointer** (`&`, `Box`, `Rc`, etc.).

### `Sized` and `?Sized`

Every generic function implicitly requires `Sized`:

```rust
fn generic<T>(t: T) { ... }
// is really:
fn generic<T: Sized>(t: T) { ... }
```

To accept unsized types, use `?Sized` (read: "maybe sized"). The parameter must then be behind a pointer since its size is unknown:

```rust
fn generic<T: ?Sized>(t: &T) { ... } // T may or may not be Sized — &T is always Sized
```

`?Sized` only exists for the `Sized` trait. You can't write `?Clone` or similar.

---

## 20.4 — Advanced Functions and Closures

### Function pointers

Functions themselves have a type: `fn`. Lowercase `fn` is a type (a function pointer), distinct from the uppercase `Fn`/`FnMut`/`FnOnce` closure traits.

```rust
fn add_one(x: i32) -> i32 { x + 1 }

fn do_twice(f: fn(i32) -> i32, arg: i32) -> i32 {
    f(arg) + f(arg)
}

fn main() {
    println!("{}", do_twice(add_one, 5)); // 12
}
```

`fn` is a type, not a trait bound — you write `f: fn(i32) -> i32`, not `f: impl Fn(i32) -> i32`.

Function pointers implement all three closure traits (`Fn`, `FnMut`, `FnOnce`) — so you can pass a function anywhere a closure is accepted. The reverse isn't always true (closures can capture environment; function pointers can't).

**Prefer `impl Fn(...)` in most cases** — it accepts both closures and function pointers. Use `fn` only when interfacing with C/FFI that doesn't have closures.

Practical pattern — enum variant constructors are function pointers:

```rust
enum Status { Value(u32), Stop }

// Status::Value is fn(u32) -> Status — use it directly as a fn pointer
let statuses: Vec<Status> = (0u32..20).map(Status::Value).collect();
```

### Returning closures

Closures don't have a nameable concrete type — each closure is its own unique type. You can't return them directly.

**Option 1: `impl Fn`** — works when returning a single closure type:

```rust
fn returns_closure() -> impl Fn(i32) -> i32 {
    |x| x + 1
}
```

**Problem with `impl Fn`**: each call site generates a distinct opaque type. Two functions both returning `impl Fn(i32) -> i32` return *different* types — you can't put them in the same `Vec`:

```rust
// This fails — opaque types don't unify
let handlers = vec![returns_closure(), returns_initialized_closure(123)];
```

**Option 2: `Box<dyn Fn>`** — erases the concrete type, puts a uniform fat pointer in the vec:

```rust
fn returns_closure() -> Box<dyn Fn(i32) -> i32> {
    Box::new(|x| x + 1)
}

fn returns_initialized_closure(init: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x + init) // move captures init into the closure
}

let handlers = vec![returns_closure(), returns_initialized_closure(123)]; // works
```

Rule of thumb:
- Returning one closure, concrete type doesn't matter → `impl Fn(...)`
- Returning different closures that need to coexist → `Box<dyn Fn(...)>`

---

## 20.5 — Macros

Macros are code that writes code at compile time. They're distinct from functions in two key ways:
- They accept a variable number of arguments (e.g. `println!("hello")` and `println!("{}", x)` work because it's a macro)
- They expand *before* the compiler processes your code, so they can implement traits, generate impl blocks, etc. — things functions can't do

Downside: harder to read and debug, and they must be defined or imported before use.

Two main families: **declarative** (`macro_rules!`) and **procedural** (three varieties).

### Declarative macros — `macro_rules!`

Work like a `match` on source code structure. Each arm has a pattern and a replacement:

```rust
#[macro_export] // make available when crate is imported
macro_rules! vec {
    ( $( $x:expr ),* ) => {  // pattern: comma-separated expressions, zero or more
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x); // repeat this for each matched $x
            )*
            temp_vec
        }
    };
}
```

Pattern syntax:
- `$x:expr` — capture a Rust expression, name it `$x`
- `$( ... ),*` — repeat the pattern, separated by commas, zero or more times
- `$( ... )*` in the body — repeat the replacement for each captured match

`vec![1, 2, 3]` expands to:

```rust
{
    let mut temp_vec = Vec::new();
    temp_vec.push(1);
    temp_vec.push(2);
    temp_vec.push(3);
    temp_vec
}
```

Fragment types beyond `expr`: `ident` (identifier), `ty` (type), `stmt` (statement), `tt` (token tree), and others.

### Procedural macros

Accept a `TokenStream` (raw token soup of your source code), manipulate it, return a new `TokenStream`. More powerful than `macro_rules!` but also more complex.

Three varieties:

**1. Custom `derive`** — `#[derive(YourTrait)]`

Define a trait and a derive macro that auto-implements it. Must live in its own crate with `proc-macro = true`:

```toml
# hello_macro_derive/Cargo.toml
[lib]
proc-macro = true

[dependencies]
syn = "2.0"   # parse TokenStream into an AST
quote = "1.0" # turn AST back into TokenStream
```

```rust
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap(); // parse to DeriveInput
    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident; // the struct/enum name being derived on

    let generated = quote! {
        impl HelloMacro for #name { // #name interpolates the identifier
            fn hello_macro() {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };

    generated.into()
}
```

User code:

```rust
#[derive(HelloMacro)]
struct Pancakes;

Pancakes::hello_macro(); // Hello, Macro! My name is Pancakes!
```

Key tools:
- `syn::parse()` — turns `TokenStream` into a structured AST (`DeriveInput`, `ItemFn`, etc.)
- `quote!` — template syntax for generating Rust code; `#variable` interpolates values
- `stringify!(#name)` — converts the identifier token to a string literal at compile time
- `.into()` — converts `quote!`'s output back to `TokenStream`

**2. Attribute-like macros** — can attach to any item (functions, modules, etc.), not just structs/enums

```rust
#[route(GET, "/")]
fn index() { ... }
```

```rust
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    // attr: the attribute args — GET, "/"
    // item: the thing being annotated — the fn
}
```

**3. Function-like macros** — look like function calls but operate on tokens, more flexible than `macro_rules!`

```rust
let sql = sql!(SELECT * FROM posts WHERE id=1);
```

```rust
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    // parse, validate, transform the SQL tokens
}
```

---

## Summary

| Feature | What it does |
|---|---|
| `unsafe` | Opt out of compiler checks for five specific superpowers |
| Raw pointers `*const T` / `*mut T` | Bypass borrow rules; unsafe to dereference |
| `extern "C"` | Call C functions from Rust (or expose Rust to C) |
| `static mut` | Mutable global — unsafe, watch for data races |
| `unsafe trait` | Trait with invariants the compiler can't verify (`Send`, `Sync`) |
| Associated types | One concrete type per trait implementation — no annotation needed at call site |
| Default type params | `<Rhs=Self>` — customizable but not required |
| Fully qualified syntax | `<Type as Trait>::method()` — disambiguate when names collide |
| Supertraits | `trait A: B` — require B to be implemented alongside A |
| Newtype pattern | Wrap a type in a tuple struct — type safety, orphan rule workaround, abstraction |
| Type alias `type` | Synonym — no new type, just shorter name |
| Never type `!` | Function that never returns; coerces to any type |
| DSTs / `?Sized` | Runtime-sized types must live behind a pointer; `?Sized` relaxes the `Sized` bound |
| `fn` type | Function pointer — implements all `Fn*` traits |
| `impl Fn` vs `Box<dyn Fn>` | Single closure return vs heterogeneous closures |
| `macro_rules!` | Declarative macro — pattern match on source code |
| Procedural macros | Transform `TokenStream` at compile time; three varieties: derive, attribute, function-like |
