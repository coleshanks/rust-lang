# Ch 10 — Generic Types, Traits, and Lifetimes

Three tools for reducing code duplication and expressing constraints on types and references.

---

## Generics

Generics let you write a function, struct, or enum that works over multiple concrete types without repeating yourself.

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

The `<T>` is a type parameter — a placeholder filled in at compile time. Without `PartialOrd`, the compiler errors: it has no guarantee `T` supports `>`.

**Zero runtime cost.** The compiler performs *monomorphization*: reads all the concrete types used with the generic and generates a separate concrete version for each. Generic code is a template — by the time the binary runs, everything is fully typed.

**In structs:**
```rust
struct Point<T> {
    x: T,
    y: T,  // both must be the same type T
}

struct Point<T, U> {
    x: T,
    y: U,  // can be different types
}

let p = Point { x: 5, y: 4.0 };  // works with T=i32, U=f64
```

**In enums** (you've been using these all along):
```rust
enum Option<T> { Some(T), None }
enum Result<T, E> { Ok(T), Err(E) }
```

**In `impl` blocks:**
```rust
impl<T> Point<T> {
    fn x(&self) -> &T { &self.x }
}

// Specialize for a concrete type — only Point<f32> gets this method:
impl Point<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```

Methods can also introduce their own generic parameters separate from the struct's:

```rust
struct Point<X1, Y1> { x: X1, y: Y1 }

impl<X1, Y1> Point<X1, Y1> {
    fn mixup<X2, Y2>(self, other: Point<X2, Y2>) -> Point<X1, Y2> {
        Point { x: self.x, y: other.y }
    }
}

let p1 = Point { x: 5, y: 10.4 };
let p2 = Point { x: "Hello", y: 'c' };
let p3 = p1.mixup(p2);
println!("{}, {}", p3.x, p3.y); // 5, c
```

---

## Traits

A trait defines behavior — a set of method signatures a type must implement to satisfy a contract. Analogous to interfaces in other languages.

**Mental model:** generics say "any type goes here" — trait bounds narrow that to "any type that can do *this*." Together they let you write flexible code that still has compile-time guarantees. Without a bound, you can't call any methods on a generic — you have no guarantee the type supports them.

```rust
pub trait Summary {
    fn summarize(&self) -> String;
}

pub struct NewsArticle {
    pub headline: String,
    pub author: String,
    pub location: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{}, by {} ({})", self.headline, self.author, self.location)
    }
}
```

`impl TraitName for TypeName { ... }` is the syntax.

**Default implementations** — a trait can provide a default body that types can use without overriding:

```rust
pub trait Summary {
    fn summarize(&self) -> String {
        String::from("(Read more...)")
    }
}

impl Summary for NewsArticle {}  // uses the default
```

Default methods can call other methods in the same trait — including required ones (no default body):

```rust
pub trait Summary {
    fn summarize_author(&self) -> String;  // required — no default

    fn summarize(&self) -> String {        // default calls the required one
        format!("(Read more from {}...)", self.summarize_author())
    }
}

impl Summary for SocialPost {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
    // summarize() gets the default, which calls our summarize_author()
}
```

**Orphan rule:** you can implement a trait on a type only if either the trait or the type is defined in your crate. Can't implement `Display` on `Vec<T>` — both are foreign. This prevents two crates from implementing the same trait on the same type and conflicting.

**Trait bounds** — constrain generics:

```rust
// Full syntax:
fn notify<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}

// `impl Trait` shorthand — same thing:
fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}
```

Key difference: `impl Trait` allows different concrete types per parameter; trait bound syntax enforces the same type:

```rust
fn notify(item1: &impl Summary, item2: &impl Summary) { }  // item1 and item2 can be different types
fn notify<T: Summary>(item1: &T, item2: &T) { }            // item1 and item2 must be the same type
```

Multiple bounds with `+`:
```rust
fn notify(item: &(impl Summary + Display)) { ... }
fn notify<T: Summary + Display>(item: &T) { ... }
```

**`where` clause** — cleaner when bounds get long:
```rust
fn some_fn<T, U>(t: &T, u: &U)
where
    T: Display + Clone,
    U: Clone + Debug,
{ ... }
```

**Returning `impl Trait`** — useful when you don't want to name the concrete return type:
```rust
fn make_summarizable() -> impl Summary {
    SocialPost { /* ... */ }
}
```

Limitation: only one concrete type per function. A function that might return `NewsArticle` or `SocialPost` based on a condition won't compile — use trait objects (`Box<dyn Trait>`) for that (ch18).

**Conditional method implementations** — implement methods only for types satisfying certain bounds:

```rust
use std::fmt::Display;

struct Pair<T> {
    x: T,
    y: T,
}

impl<T> Pair<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Display + PartialOrd> Pair<T> {
    fn cmp_display(&self) {
        if self.x >= self.y {
            println!("Largest: x = {}", self.x);
        } else {
            println!("Largest: y = {}", self.y);
        }
    }
}
// Only Pair<T> where T: Display + PartialOrd gets cmp_display
```

**Blanket implementations** — implement a trait for any type satisfying a bound. The standard library does this extensively:

```rust
// std does this: implement ToString for anything that implements Display
impl<T: Display> ToString for T {
    // ...
}

let s = 3.to_string();     // works — i32 implements Display
let s = true.to_string();  // works — bool implements Display
```

---

## Lifetimes

Lifetimes are annotations that tell the borrow checker how the lifetimes of references relate to each other. They don't change how long something lives — they describe relationships so the compiler can verify safety.

**The core problem:** when a function takes multiple references and returns one, the compiler can't tell which input the returned reference came from, so it can't verify the return is valid at the call site.

```rust
fn longest(x: &str, y: &str) -> &str { // compile error: missing lifetime specifier
    if x.len() > y.len() { x } else { y }
}
```

The fix — annotate the relationship:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

`'a` is resolved to the *overlap* of the two input lifetimes — i.e. the shorter one. The returned reference is only trusted valid for that long. The compiler has to assume worst case — it doesn't know at compile time which branch runs, so it can't know whether the return points to `x` or `y`.

```rust
// Valid — result used within the shorter lifetime (string2):
let string1 = String::from("long string");
{
    let string2 = String::from("xyz");
    let result = longest(string1.as_str(), string2.as_str());
    println!("{result}"); // fine — result, string2, string1 all alive here
}

// Invalid — result used after string2 is dropped:
let string1 = String::from("long string");
let result;
{
    let string2 = String::from("xyz");
    result = longest(string1.as_str(), string2.as_str());
}
println!("{result}"); // compile error: string2 dropped while result still in use
```

If you always return from one specific parameter, you only need to annotate that one:

```rust
fn longest<'a>(x: &'a str, y: &str) -> &'a str {
    x  // always returns x, so y's lifetime is irrelevant
}
```

You can't return a reference to something created inside the function — it's dropped on return:

```rust
fn longest<'a>(x: &str, y: &str) -> &'a str {
    let result = String::from("really long string");
    result.as_str()  // compile error: cannot return reference to local variable
}
// Fix: return an owned String instead of &str
```

**Lifetime annotations in structs** — required when a struct holds a reference:

```rust
struct ImportantExcerpt<'a> {
    part: &'a str,
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().unwrap();
    let i = ImportantExcerpt { part: first_sentence };
    // i cannot outlive novel — that's what 'a expresses
}
```

**Lifetime elision rules** — the compiler infers lifetimes in common cases so you don't always write them:

1. Each reference parameter gets its own lifetime.
2. If there's exactly one input lifetime, it's applied to all output lifetimes.
3. If one of the inputs is `&self` or `&mut self`, its lifetime is applied to all outputs.

Example — `first_word` compiles with no annotations because elision resolves it unambiguously:

```rust
fn first_word(s: &str) -> &str {  // no annotations needed
    // compiler applies rule 1: s gets 'a
    // compiler applies rule 2: return gets 'a
    // result: fn first_word<'a>(s: &'a str) -> &'a str
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' { return &s[0..i]; }
    }
    &s[..]
}
```

`longest` can't be elided because it has two input lifetimes and no `self` — rule 2 and 3 don't apply, so the compiler asks you to be explicit.

**`'static`** — the reference lives for the entire program duration. String literals are `'static` because they're baked into the binary:

```rust
let s: &'static str = "hello";  // lives forever — stored in binary
```

Be skeptical of error messages suggesting `'static` as a fix — usually means you're dodging a real design issue. Fix the underlying lifetime relationship instead.

**Mental model:** `'a` is about connecting the return lifetime to something the *caller* owns. The caller passes in refs with lifetimes it controls, and `'a` says "the return is valid within that same scope." Owned values don't need lifetimes — whoever owns it drops it, no ambiguity.

---

## Putting It Together

Generics, trait bounds, and lifetimes often appear together:

```rust
use std::fmt::Display;

fn longest_with_announcement<'a, T>(
    x: &'a str,
    y: &'a str,
    ann: T,
) -> &'a str
where
    T: Display,
{
    println!("Announcement: {ann}");
    if x.len() > y.len() { x } else { y }
}
```

- `'a` ties the return lifetime to the inputs
- `T: Display` constrains what can be passed as `ann`
- No runtime cost for any of it — all resolved at compile time
