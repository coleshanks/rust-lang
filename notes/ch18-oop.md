# Ch 18 — OOP Features of Rust

Rust isn't a traditional OOP language, but it supports many OOP concepts — sometimes in different ways, sometimes better. This chapter walks through the classic OOP checklist and shows how Rust approaches each one.

---

## 18.1 — What Is OOP?

### Objects: data + behavior

OOP languages define objects as things that bundle data and the methods that operate on it. Rust does this with structs/enums + `impl` blocks. Different name, same idea.

### Encapsulation

Hiding implementation details behind a public API. Rust does this with `pub` — everything is private by default.

```rust
pub struct AveragedCollection {
    list: Vec<i32>,   // private
    average: f64,     // private
}

impl AveragedCollection {
    pub fn add(&mut self, value: i32) {
        self.list.push(value);
        self.update_average();
    }

    pub fn remove(&mut self) -> Option<i32> {
        let result = self.list.pop();
        match result {
            Some(value) => {
                self.update_average();
                Some(value)
            }
            None => None,
        }
    }

    pub fn average(&self) -> f64 {
        self.average
    }

    fn update_average(&mut self) { // private — internal only
        let total: i32 = self.list.iter().sum();
        self.average = total as f64 / self.list.len() as f64;
    }
}
```

External code can only call `add`, `remove`, and `average`. The `list` and `average` fields are hidden — callers can't desync them by accident. Classic encapsulation.

### Inheritance

**Rust does not have inheritance.** You can't define a struct that inherits fields and method implementations from a parent struct.

Inheritance bundles behavior into a type hierarchy — subclasses get everything from the parent whether it makes sense or not. A `FlyingAnimal` parent breaks down the moment you need a penguin. You end up with workarounds or methods that exist on a type but shouldn't.

Rust flips this: behavior is defined separately as traits and attached to whatever types make sense. A `Fly` trait, a `Swim` trait, a `Speak` trait — a penguin implements `Swim` and `Speak` but not `Fly`. No hierarchy, no inherited baggage. Types opt into exactly the behavior they need.

```rust
trait Fly  { fn fly(&self); }
trait Swim { fn swim(&self); }

struct Duck;
struct Penguin;

impl Fly  for Duck    { fn fly(&self)  { println!("flap flap"); } }
impl Swim for Duck    { fn swim(&self) { println!("paddle"); } }
impl Swim for Penguin { fn swim(&self) { println!("zoom"); } }
// Penguin doesn't implement Fly — and that's fine, no workaround needed
```

What Rust offers in place of inheritance:
- **Default trait method implementations** — a trait can provide a default body that implementors can override. Similar to inheriting a method you can selectively override.
- **Generics + trait bounds** — static polymorphism (resolved at compile time)
- **Trait objects** — dynamic polymorphism (resolved at runtime)

This approach is called **bounded parametric polymorphism**. Behavior is distributed across types via traits rather than inherited down a hierarchy.

### `self` in trait methods

Trait methods (and regular `impl` methods) don't always need `&self`. Three forms:

```rust
trait Example {
    fn read(&self);           // borrows the instance immutably — can read fields
    fn mutate(&mut self);     // borrows the instance mutably — can modify fields
    fn standalone();          // no instance at all — associated function (like a constructor)
}

struct Counter { count: u32 }

impl Example for Counter {
    fn read(&self) {
        println!("{}", self.count); // read-only access to fields
    }

    fn mutate(&mut self) {
        self.count += 1; // can modify fields
    }

    fn standalone() {
        println!("no instance needed");
    }
}

// calling them:
let mut c = Counter { count: 0 };
c.read();           // dot syntax — instance on the left
c.mutate();         // dot syntax — instance on the left
Counter::standalone(); // double colon — no instance
```

`&self` is just the most common because most methods need to read the struct's data. Use `&mut self` when you need to change it. Leave `self` off entirely for things that don't need an instance (constructors, factory functions, utilities).

---

## 18.2 — Trait Objects

### The problem

A `Vec<T>` can only hold one type. If you're building a GUI library with buttons, checkboxes, and dropdowns — all of which need to be drawn — you can't put them in the same vec with generics unless they're all the same type.

### The solution: `Box<dyn Trait>`

A **trait object** is a pointer to some type that implements a given trait, where the exact type is unknown at compile time and looked up at runtime.

```rust
pub trait Draw {
    fn draw(&self);
}

pub struct Screen {
    pub components: Vec<Box<dyn Draw>>, // any type that implements Draw
}

impl Screen {
    pub fn run(&self) {
        for component in self.components.iter() {
            component.draw(); // runtime dispatch — calls the right draw() for each type
        }
    }
}
```

`Box<dyn Draw>` is the trait object syntax:
- `Box<T>` — heap pointer, required because the compiler doesn't know the size at compile time
- `dyn` — signals dynamic dispatch (method resolved at runtime via vtable)
- `Draw` — the trait being required

### Implementing types

```rust
pub struct Button {
    pub width: u32,
    pub height: u32,
    pub label: String,
}

impl Draw for Button {
    fn draw(&self) {
        // draw a button
    }
}
```

A user of the library can define their own type:

```rust
struct SelectBox {
    width: u32,
    height: u32,
    options: Vec<String>,
}

impl Draw for SelectBox {
    fn draw(&self) {
        // draw a select box
    }
}
```

And mix them freely:

```rust
let screen = Screen {
    components: vec![
        Box::new(SelectBox {
            width: 75,
            height: 10,
            options: vec![
                String::from("Yes"),
                String::from("Maybe"),
                String::from("No"),
            ],
        }),
        Box::new(Button {
            width: 50,
            height: 10,
            label: String::from("OK"),
        }),
    ],
};

screen.run();
```

`Screen` doesn't care what concrete type each component is — only that it implements `Draw`. This is duck typing: if it implements `draw()`, it works.

If you try to put a type that doesn't implement `Draw` into the vec, it's a compile error.

### Trait objects vs. generics

```rust
// Generics — all components must be the same type T
pub struct Screen<T: Draw> {
    pub components: Vec<T>,
}

// Trait objects — components can be any mix of types
pub struct Screen {
    pub components: Vec<Box<dyn Draw>>,
}
```

| | Generics (`impl Trait` / `T: Trait`) | Trait objects (`dyn Trait`) |
|---|---|---|
| Dispatch | Static (compile time) | Dynamic (runtime via vtable) |
| Performance | Faster — can inline | Slight overhead — can't inline |
| Flexibility | Homogeneous collections | Heterogeneous collections |
| Use when | Types known at compile time | Types unknown / user-extensible |

### Dynamic dispatch and the vtable

When you call a method on `Box<dyn Draw>`, Rust looks up the method in a **vtable** — a table of function pointers specific to the concrete type. This is the runtime cost of `dyn`. It's small but real, and it prevents inlining.

---

## 18.3 — OOP Design Patterns

### The state pattern

A classic OOP pattern where a value's behavior changes based on internal state. States are represented as objects; the value delegates behavior to whatever state it currently holds.

Blog post workflow: Draft → PendingReview → Published. Only published posts return content.

```rust
use blog::Post;

fn main() {
    let mut post = Post::new();

    post.add_text("I ate a salad for lunch today");
    assert_eq!("", post.content()); // draft — empty

    post.request_review();
    assert_eq!("", post.content()); // pending — still empty

    post.approve();
    assert_eq!("I ate a salad for lunch today", post.content()); // published
}
```

### Implementation

Starting point — the struct and its first state:

```rust
pub struct Post {
    state: Option<Box<dyn State>>, // current state as a trait object
    content: String,
}

impl Post {
    pub fn new() -> Post {
        Post {
            state: Some(Box::new(Draft {})), // always starts as Draft
            content: String::new(),
        }
    }
}

trait State {}

struct Draft {}
impl State for Draft {}
```

- `state: Option<Box<dyn State>>` — three wrappers stacked, each solving a different problem:
  - `dyn State` — "some type that implements `State`, unknown at compile time." Could be `Draft`, `PendingReview`, `Published`. Compiler can't know the size.
  - `Box<dyn State>` — since size is unknown, can't store it on the stack directly. `Box` puts it on the heap and gives us a fixed-size pointer. `Post` always knows how big `state` is — it's just a pointer regardless of which state is inside.
  - `Option<Box<dyn State>>` — wraps it in `Option` so it can temporarily be `None`. Required for the `take()` trick during transitions: `take()` pulls the value out leaving `None`, giving you ownership of the old state to call the transition method on, then you put the new state back:
  ```rust
  if let Some(s) = self.state.take() { // state is now None, s owns old state
      self.state = Some(s.request_review()); // new state goes back in
  }
  ```
  Without `Option`, Rust won't let you move out of a struct field — there'd be no way to get ownership of the current state to transition it.
- `Draft {}` is an **empty struct** — no fields, just a type. State structs don't hold data; they exist purely to represent which state we're in. The actual content lives in `Post`.
- `impl State for Draft {}` is empty too — `State` has no required methods yet at this stage.
- `Post::new()` enforces that every post starts as `Draft`. You can't construct a `Post` directly in any other state.

Full implementation:

impl Post {
    pub fn new() -> Post {
        Post {
            state: Some(Box::new(Draft {})), // always starts as Draft
            content: String::new(),
        }
    }

    pub fn add_text(&mut self, text: &str) {
        self.content.push_str(text); // doesn't depend on state
    }

    pub fn content(&self) -> &str {
        self.state.as_ref().unwrap().content(self) // delegates to state
    }

    pub fn request_review(&mut self) {
        if let Some(s) = self.state.take() { // take() moves state out, leaves None
            self.state = Some(s.request_review()) // state returns its successor
        }
    }

    pub fn approve(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.approve())
        }
    }
}

trait State {
    fn request_review(self: Box<Self>) -> Box<dyn State>; // takes ownership via Box
    fn approve(self: Box<Self>) -> Box<dyn State>;
    fn content<'a>(&self, post: &'a Post) -> &'a str {
        "" // default — most states return empty
    }
}

struct Draft {}
impl State for Draft {
    fn request_review(self: Box<Self>) -> Box<dyn State> { Box::new(PendingReview {}) }
    fn approve(self: Box<Self>) -> Box<dyn State> { self } // no-op
}

struct PendingReview {}
impl State for PendingReview {
    fn request_review(self: Box<Self>) -> Box<dyn State> { self } // no-op
    fn approve(self: Box<Self>) -> Box<dyn State> { Box::new(Published {}) }
}

struct Published {}
impl State for Published {
    fn request_review(self: Box<Self>) -> Box<dyn State> { self }
    fn approve(self: Box<Self>) -> Box<dyn State> { self }
    fn content<'a>(&self, post: &'a Post) -> &'a str {
        &post.content // only Published actually returns content
    }
}
```

Key mechanics:
- `self.state.take()` — moves the state value out of the `Option`, leaving `None` temporarily. Required because we need ownership to call the state transition method.
- `self: Box<Self>` — method takes ownership of the boxed state, consuming it. The old state is dropped, the new state is returned.
- `content` default implementation returns `""` — only `Published` overrides it.

### The Rust-idiomatic alternative: encode states as types

Instead of runtime state objects, make each state its own type. Invalid transitions become compile errors.

```rust
pub struct Post { content: String }       // only reachable after approval
pub struct DraftPost { content: String }
pub struct PendingReviewPost { content: String }

impl Post {
    pub fn new() -> DraftPost {            // Post::new() returns a DraftPost, not Post
        DraftPost { content: String::new() }
    }
    pub fn content(&self) -> &str { &self.content }
}

impl DraftPost {
    pub fn add_text(&mut self, text: &str) { self.content.push_str(text); }
    pub fn request_review(self) -> PendingReviewPost { // consumes DraftPost
        PendingReviewPost { content: self.content }
    }
}

impl PendingReviewPost {
    pub fn approve(self) -> Post { // consumes PendingReviewPost, returns Post
        Post { content: self.content }
    }
}
```

Usage:

```rust
fn main() {
    let mut post = Post::new();           // DraftPost
    post.add_text("I ate a salad");

    let post = post.request_review();     // PendingReviewPost — shadows old post
    let post = post.approve();            // Post

    assert_eq!("I ate a salad", post.content());
}
```

- Calling `content()` on a `DraftPost` is a **compile error** — the method doesn't exist on that type
- No `Option::take()`, no runtime checks, no way to get into an invalid state
- State transitions consume the old value — you physically can't use a `DraftPost` after calling `request_review()`

### Tradeoffs

| | OOP state pattern | Type-encoded states |
|---|---|---|
| Invalid states | Caught at runtime | Caught at compile time |
| Encapsulation | High — transitions hidden behind methods | Lower — types are visible |
| Adding new states | Add new struct + trait impl | Add new struct, update transitions |
| Rust idiomatic | Less so | More so |

The type-encoded approach is more Rust-idiomatic — it uses the ownership system to make invalid states literally unrepresentable. The OOP approach is fine too and maps more directly to how you'd write it in other languages.

---

## Summary

| Concept | Rust equivalent |
|---|---|
| Objects | Structs/enums + `impl` blocks |
| Encapsulation | `pub` / private by default |
| Inheritance | Not supported — use traits with default methods |
| Polymorphism (static) | Generics + trait bounds |
| Polymorphism (dynamic) | `Box<dyn Trait>` trait objects |
| State pattern | Trait objects OR type-encoded states |
