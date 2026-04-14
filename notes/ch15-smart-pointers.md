# Ch 15 — Smart Pointers

Smart pointers are data structures that act like pointers but have additional metadata and capabilities. Unlike regular references (which only borrow), smart pointers typically *own* the data they point to. Implemented using structs that implement the `Deref` and `Drop` traits.

Smart pointers covered: `Box<T>`, `Rc<T>`, `RefCell<T>`, `Weak<T>`.

---

## `Box<T>` — Heap Allocation

Stores data on the heap instead of the stack. The pointer itself lives on the stack; the data lives on the heap.

```rust
let b = Box::new(5);
println!("b = {b}");  // works like a normal value
```

When `b` goes out of scope, both the box and the heap data are freed.

### When to use `Box<T>`

1. **Type size unknown at compile time** — recursive types
2. **Large data, want to transfer ownership** — moves the pointer, not the data
3. **Trait objects** — own a value that implements a trait, don't care about the concrete type

### Recursive types

Without `Box`, the compiler can't determine the size of a recursive type:

```rust
// Doesn't compile — infinite size
enum List {
    Cons(i32, List),
    Nil,
}
```

Fix: break the recursion with a box (known, fixed pointer size):

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}

let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
```

This is called a **cons list** — a linked list from Lisp. In practice, use `Vec<T>` instead.

---

## The `Deref` Trait

Lets you customize the `*` dereference operator. Smart pointers implement this so they can be used like regular references.

```rust
let x = 5;
let y = Box::new(x);

assert_eq!(5, *y);  // * works on Box just like a reference
```

### Implementing `Deref`

```rust
use std::ops::Deref;

struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0  // return reference to inner value
    }
}
```

When you write `*y`, Rust actually runs `*(y.deref())` behind the scenes.

### Deref Coercion

Rust automatically converts `&T` to `&U` when `T: Deref<Target=U>`. This chains as many times as needed at compile time — no runtime cost.

```rust
fn hello(name: &str) { println!("Hello, {name}!"); }

let m = MyBox::new(String::from("Rust"));
hello(&m);  // &MyBox<String> → &String → &str, automatically
```

Without deref coercion you'd write: `hello(&(*m)[..])` — much noisier.

Rules:
- `&T` → `&U` when `T: Deref<Target=U>`
- `&mut T` → `&mut U` when `T: DerefMut<Target=U>`
- `&mut T` → `&U` when `T: Deref<Target=U>` (mut can coerce to immutable, not vice versa)

---

## The `Drop` Trait

Customizes what happens when a value goes out of scope — automatic cleanup. Always called in **reverse order of creation** (LIFO).

```rust
struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping: {}", self.data);
    }
}
```

`Drop` is in the prelude — no import needed.

### Early drop

You can't call `.drop()` directly — Rust would double-free. Use `std::mem::drop` (also in the prelude):

```rust
drop(c);  // forces cleanup now, not at end of scope
```

Common use: releasing a lock early so other code can acquire it.

---

## `Rc<T>` — Reference Counting

Enables **multiple ownership** in single-threaded code. Tracks how many references exist; cleans up when count hits zero.

```rust
use std::rc::Rc;

let a = Rc::new(Cons(5, Rc::new(Nil)));
let b = Cons(3, Rc::clone(&a));  // a now has 2 owners
let c = Cons(4, Rc::clone(&a));  // a now has 3 owners
```

Use `Rc::clone(&a)` — cheap (just increments the count). Don't use `a.clone()` for this, which signals an expensive deep copy.

```rust
println!("{}", Rc::strong_count(&a)); // 3
println!("{}", Rc::weak_count(&a));   // 0 — no weak references yet
```

**Limitation:** `Rc<T>` only gives immutable access. For mutable shared data, combine with `RefCell<T>`.

**Single-threaded only.** For multithreading, use `Arc<T>` (Ch 16).

---

## `RefCell<T>` — Interior Mutability

Moves borrow checking from **compile time** to **runtime**. Lets you mutate data through an immutable reference — useful when you know the code is correct but the compiler can't verify it.

```rust
use std::cell::RefCell;

let data = RefCell::new(vec![1, 2, 3]);

data.borrow_mut().push(4);  // mutable borrow
data.borrow().len();        // immutable borrow
```

- `borrow()` → `Ref<T>` (immutable smart pointer, implements `Deref`)
- `borrow_mut()` → `RefMut<T>` (mutable smart pointer, implements `DerefMut`)

Both types track the active borrows — `RefCell<T>` internally keeps counts of how many `Ref<T>` and `RefMut<T>` values exist. When they go out of scope, the count decrements.

The same rules still apply — one mutable borrow OR many immutable borrows. Violating this **panics at runtime** instead of failing at compile time:

```rust
let data = RefCell::new(5);

let _b1 = data.borrow_mut();
let _b2 = data.borrow_mut(); // panic: already mutably borrowed
```

**Single-threaded only.** For multithreading, use `Mutex<T>`.

### Comparison

| Type | Ownership | Borrow checking | Mutability |
|---|---|---|---|
| `Box<T>` | Single | Compile time | Mutable or immutable |
| `Rc<T>` | Multiple | Compile time | Immutable only |
| `RefCell<T>` | Single | Runtime | Mutable or immutable |

### `Rc<RefCell<T>>` — multiple owners of mutable data

Combining both:

```rust
let value = Rc::new(RefCell::new(5));

let a = Rc::clone(&value);
let b = Rc::clone(&value);

*value.borrow_mut() += 10;  // all clones see the updated value
```

---

## Reference Cycles and `Weak<T>`

`Rc<T>` can create memory leaks if two values point to each other — reference counts never hit zero so memory is never freed.

```
a → b → a  (cycle — both stay at count 2, never dropped)
```

### Solution: `Weak<T>`

Weak references don't express ownership and don't affect the `strong_count`. The value is dropped when `strong_count` hits zero, regardless of `weak_count`.

```rust
use std::rc::{Rc, Weak};

let weak: Weak<i32> = Weak::new(); // empty weak reference — upgrade() always returns None

let strong = Rc::new(5);
let weak = Rc::downgrade(&strong);  // creates Weak<T>

// Access via upgrade() — returns Option<Rc<T>>
match weak.upgrade() {
    Some(val) => println!("{val}"),
    None => println!("value was dropped"),
}
```

### Design pattern: trees

- Parent owns children → `Rc<Node>`
- Child references parent → `Weak<Node>` (no ownership, no cycle)

```rust
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,      // weak — child doesn't own parent
    children: RefCell<Vec<Rc<Node>>>, // strong — parent owns children
}
```

Dropping the parent drops the children. Dropping a child doesn't affect the parent. No cycle, no leak.
