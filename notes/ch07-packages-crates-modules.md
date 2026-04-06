# Ch 7 — Packages, Crates, and Modules

## The Module System as a File Tree

This is the mental model that makes ch7 click. Rust's module system maps almost 1:1 to navigating a filesystem in the terminal.

| Filesystem | Rust module system |
|---|---|
| `/` (root) | `crate` |
| directory | `mod` |
| `/` path separator | `::` |
| `../` (go up) | `super::` |
| symlink | `use` |
| file permissions | `pub` / private |
| `ls` a dir to see contents | look inside a `mod` block or its `.rs` file |

**Example:**
```
crate::front_of_house::hosting::add_to_waitlist()
```
is basically:
```
/front_of_house/hosting/add_to_waitlist
```

---

## Crates

The unit the compiler works with — one compilation unit.

**Binary crate** — compiles to an executable. Has a `main` function. Something you run (CLI tools, servers, etc.).

**Library crate** — no `main`, no executable. Provides reusable code other programs depend on (like `rand`).

A package can have both. Cargo conventions:
- `src/main.rs` → binary crate root
- `src/lib.rs` → library crate root
- `src/bin/*.rs` → additional binary crates

---

## Packages

A bundle of one or more crates with a `Cargo.toml`. Rules:
- At most **one** library crate
- Any number of binary crates
- Must have at least one crate

`cargo new` → binary by default (`src/main.rs`)
`cargo new --lib` → library (`src/lib.rs`)
To get both, just create `src/lib.rs` manually after `cargo new`.

---

## rustc vs Cargo

- **rustc** — the actual compiler. Takes `.rs` files and compiles them.
- **Cargo** — build tool and package manager. Manages dependencies, project structure, calls `rustc` for you with the right flags. You almost never call `rustc` directly.

---

## Modules

`mod` creates a new branch in the module tree. Can nest as deep as you want.

Three ways to define a module:

**Inline:**
```rust
mod front_of_house {
    mod hosting {
        fn add_to_waitlist() {}
    }
}
```

**Separate file (modern, idiomatic):**
```rust
// in lib.rs or main.rs
mod front_of_house;
// Rust looks for src/front_of_house.rs
```

**Separate directory (older style, still works):**
```rust
mod front_of_house;
// Rust looks for src/front_of_house/mod.rs
```

Prefer the `foo.rs` style over `foo/mod.rs` — the latter gives you a bunch of files all named `mod.rs` which is confusing in an editor.

A module file (`foo.rs`) is the branch. Everything defined directly in it — structs, fns, enums, etc. — hangs off that node. The associated `foo/` directory only appears if `foo` has submodules that need their own files. No submodules = just a file, no directory needed.

When a module lives in a directory with submodules, the directory needs a `foo.rs` at the same level (or `foo/mod.rs`) that declares the submodules:

```
src/
├── main.rs           ← declares: pub mod garden;
├── garden.rs         ← declares: pub mod vegetables;
└── garden/
    └── vegetables.rs ← defines: pub struct Asparagus {}
```

---

## Privacy

Private by default — everything is hidden from outside its module unless marked `pub`.

Rules:
- Parent modules **cannot** access private items in child modules
- Child modules **can** access items in parent modules (and ancestors) regardless of visibility

`pub` opens things up:
```rust
pub mod hosting {       // module is accessible
    pub fn add_to_waitlist() {}  // function is accessible
}
```

Without `pub` on both, the inner function is still unreachable from outside.

**Structs vs enums:**
- `pub struct` does NOT make fields public — mark each field individually
- `pub enum` makes ALL variants public automatically

```rust
pub struct Breakfast {
    pub toast: String,        // accessible
    seasonal_fruit: String,   // private — only code in this module can touch it
}

pub enum Appetizer {
    Soup,   // public (automatically)
    Salad,  // public (automatically)
}
```

This matters for library design — private fields let you change internals without breaking callers.

---

## Paths

Two kinds:

**Absolute** — starts from crate root:
```rust
crate::front_of_house::hosting::add_to_waitlist();
```

**Relative** — starts from current module:
```rust
front_of_house::hosting::add_to_waitlist();
```

**`super::`** — go up one level (like `../`):
```rust
mod back_of_house {
    fn fix_incorrect_order() {
        super::deliver_order(); // calls function in parent module
    }
}
```

---

## `use` — Symlinks for Paths

Brings a path into scope so you don't have to write it in full every time. Think of it as a symlink.

```rust
use crate::front_of_house::hosting;
hosting::add_to_waitlist(); // no full path needed
```

**Idiomatic conventions:**
- Functions → bring the parent module into scope, call `parent::fn()`
- Structs/enums/other → bring the full path

```rust
// function — stop one level up
use crate::front_of_house::hosting;
hosting::add_to_waitlist();

// struct — full path
use std::collections::HashMap;
HashMap::new();
```

**`as`** — alias for name conflicts:
```rust
use std::fmt::Result;
use std::io::Result as IoResult;
```

**`pub use`** — re-export. Makes the imported item part of your public API:
```rust
pub use crate::front_of_house::hosting;
```
External code can now use `your_crate::hosting::...` instead of the full internal path. Useful for reshaping your public API without changing internal structure.

**Nested paths** — combine imports from the same root:
```rust
use std::{cmp::Ordering, io};
use std::io::{self, Write}; // 'self' includes the module itself
```

**Glob** — import everything public (use sparingly):
```rust
use std::collections::*;
```

---

## Separating Modules into Files

`mod foo;` (semicolon, no braces) tells Rust to find the module in a file. Only declare a module once — other files reference it via paths.

```rust
// src/lib.rs
mod front_of_house;          // load from src/front_of_house.rs
pub use crate::front_of_house::hosting;
```

```rust
// src/front_of_house.rs
pub mod hosting;             // load from src/front_of_house/hosting.rs
```

```rust
// src/front_of_house/hosting.rs
pub fn add_to_waitlist() {}
```

File structure must mirror the module hierarchy.

---

## Summary

- **Package** — Cargo project with a `Cargo.toml`
- **Crate** — binary (has `main`) or library (no `main`), one per compilation unit
- **Module** — branch in the tree, maps to a file or inline block
- **Path** — how you navigate the tree (`::` = `/`, `super` = `../`, `crate` = root)
- **`pub`** — opens visibility, private by default
- **`use`** — symlink to a path, reduces repetition
- **`pub use`** — symlink that's also part of your public API
