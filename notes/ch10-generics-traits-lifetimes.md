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

The `<T>` is a type parameter — a placeholder filled in at compile time.

**Zero runtime cost.** The compiler performs *monomorphization*: it reads all the concrete types you use with the generic and generates a separate concrete version for each. The generic code is just a template; by the time the binary runs, everything is fully typed.

**In structs:**
```rust
struct Point<T> {
    x: T,
    y: T,
}
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

// Can also specialize for a concrete type:
impl Point<f32> {
    fn distance_from_origin(&self) -> f32 { ... }
}
```

---

## Traits

A trait defines behavior — a set of method signatures a type must implement to satisfy a contract. Analogous to interfaces in other languages.

**Mental model:** generics say "any type goes here" — trait bounds narrow that to "any type that can do *this*." Together they let you write flexible code that still has compile-time guarantees. Without a bound, you can't call any methods on a generic — you have no guarantee the type supports them.

```rust
pub trait Summary {
    fn summarize(&self) -> String;
}

pub struct Article { pub title: String, pub content: String }

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}: {}", self.title, &self.content[..50])
    }
}
```

**Default implementations:** a trait can provide a default body that types can use or override.

```rust
pub trait Summary {
    fn summarize(&self) -> String {
        String::from("(Read more...)")
    }
}
```

**Orphan rule:** you can implement a trait on a type only if either the trait or the type is defined in your crate. Can't implement `Display` on `Vec<T>` — both are foreign.

**Trait bounds** — constrain generics:

```rust
fn notify<T: Summary>(item: &T) { ... }

// Shorthand with `impl Trait`:
fn notify(item: &impl Summary) { ... }
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
fn make_summarizable() -> impl Summary { ... }
```

Limitation: you can only return one concrete type this way. A function that might return `Article` or `Tweet` based on a condition won't compile with this approach — use trait objects (`Box<dyn Trait>`) for that.

---

## Lifetimes

Lifetimes are annotations that tell the borrow checker how the lifetimes of references relate to each other. They don't change how long something lives — they just describe the relationship so the compiler can verify safety.

**The core problem:** when a function takes multiple references and returns a reference, the compiler can't tell which input the returned reference came from, so it can't verify the return is valid at the call site.

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

`'a` gets resolved to the *overlap* of the two input lifetimes — i.e. the shorter one. The returned reference is only trusted valid for that long.

Why the shorter? The compiler doesn't know at compile time which branch runs, so it can't know whether the return points to `x` or `y`. It has to assume worst case — that the result might point to whichever input dies soonest. Once that input goes out of scope, the result is considered invalid, even if it actually points to the longer-lived one. Conservative, but safe.

The compiler uses this to check callers.

**Lifetime annotations in structs** — required when a struct holds a reference:
```rust
struct Important<'a> {
    part: &'a str,
}
```

This says the struct can't outlive the reference it holds.

**Lifetime elision rules** — the compiler can infer lifetimes in common cases so you don't always have to write them:
1. Each reference parameter gets its own lifetime.
2. If there's exactly one input lifetime, it's applied to all output lifetimes.
3. If one of the inputs is `&self` or `&mut self`, its lifetime is applied to all outputs.

If the rules produce an unambiguous result, no annotations needed. If not, the compiler asks you to be explicit.

**Mental model:** `'a` is always about connecting the return lifetime to something the *caller* owns. The caller passes in refs with lifetimes it controls, and `'a` says "the return is valid within that same scope." You can't return a ref to something created inside the function — that's dropped when the function returns, so the ref is immediately dangling. Return owned data (`String`, not `&str`) in that case.

Owned values don't need lifetimes — whoever owns it drops it, no ambiguity. Lifetimes only exist to reason about references, which are borrowed views into something owned elsewhere.

**`'static`** — the reference lives for the entire duration of the program. String literals are `'static` because they're baked into the binary.

```rust
let s: &'static str = "hello";
```

Be skeptical of error messages suggesting `'static` as a fix — it usually means you're dodging a real design issue.

---

## Putting It Together

Generics, trait bounds, and lifetimes often appear together:

```rust
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
- No runtime cost for any of it
