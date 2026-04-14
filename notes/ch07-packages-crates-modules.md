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

```bash
cargo new myproject        # binary by default (src/main.rs)
cargo new myproject --lib  # library (src/lib.rs)
# to get both: create src/lib.rs manually after cargo new
```

---

## Modules

`mod` creates a new branch in the module tree. Can nest as deep as needed.

```rust
// src/lib.rs
mod front_of_house {
    mod hosting {
        fn add_to_waitlist() {}
        fn seat_at_table() {}
    }

    mod serving {
        fn take_order() {}
        fn serve_order() {}
        fn take_payment() {}
    }
}
```

Module tree for the above:

```
crate
 └── front_of_house
     ├── hosting
     │   ├── add_to_waitlist
     │   └── seat_at_table
     └── serving
         ├── take_order
         ├── serve_order
         └── take_payment
```

The root of the tree is always the implicit `crate` module. Sibling modules share a parent. The module tree mirrors a directory tree.

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

`mod` is **not** `#include` — it tells Rust where to find a module. Declare it once; reference it from elsewhere via paths.

---

## Separating Modules into Files

File structure must mirror the module hierarchy:

```
src/
├── main.rs           ← declares: pub mod garden;
├── garden.rs         ← declares: pub mod vegetables;
└── garden/
    └── vegetables.rs ← defines: pub struct Asparagus {}
```

```rust
// src/main.rs
use crate::garden::vegetables::Asparagus;
pub mod garden;

fn main() {
    let plant = Asparagus {};
}
```

```rust
// src/garden.rs
pub mod vegetables;
```

```rust
// src/garden/vegetables.rs
#[derive(Debug)]
pub struct Asparagus {}
```

A module file (`foo.rs`) is the branch. A `foo/` directory only appears if `foo` has submodules that need their own files.

---

## Privacy

Private by default — everything is hidden from outside its module unless marked `pub`.

Rules:
- Parent modules **cannot** access private items in child modules
- Child modules **can** access items in parent modules (and ancestors) regardless of visibility

```rust
mod front_of_house {
    pub mod hosting {       // module is accessible
        pub fn add_to_waitlist() {}  // function is accessible
    }
}
```

Without `pub` on both the module and the function, the inner function is still unreachable from outside. `pub mod` alone doesn't expose the contents.

**Structs vs enums:**
- `pub struct` does NOT make fields public — each field needs `pub` individually
- `pub enum` makes ALL variants public automatically

```rust
pub struct Breakfast {
    pub toast: String,        // accessible from outside
    seasonal_fruit: String,   // private — only code in this module can touch it
}

// Private fields require a public constructor — can't build the struct otherwise
impl Breakfast {
    pub fn summer(toast: &str) -> Breakfast {
        Breakfast {
            toast: String::from(toast),
            seasonal_fruit: String::from("peaches"),
        }
    }
}

pub enum Appetizer {
    Soup,   // public automatically
    Salad,  // public automatically
}
```

Private fields let you change internals without breaking callers — important for library design.

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
        cook_order();
        super::deliver_order(); // call function in parent module
    }
    fn cook_order() {}
}

fn deliver_order() {}
```

---

## `use` — Symlinks for Paths

Brings a path into scope so you don't have to write it in full every time.

```rust
use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist(); // no full path needed
}
```

**Important:** `use` only applies to the scope where it's declared. Child modules don't inherit it:

```rust
use crate::front_of_house::hosting;

mod customer {
    pub fn eat_at_restaurant() {
        hosting::add_to_waitlist(); // compile error: unresolved import
        // fix: move `use` inside this module, or use `super::hosting`
    }
}
```

**Idiomatic conventions:**
- Functions → bring the parent module into scope, call `parent::fn()`
- Structs/enums/other → bring the full path to the item

```rust
// function — stop one level up
use crate::front_of_house::hosting;
hosting::add_to_waitlist();

// struct — full path
use std::collections::HashMap;
let mut map = HashMap::new();
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
use std::io::{self, Write}; // `self` includes std::io itself
```

**Glob** — import everything public (use sparingly):
```rust
use std::collections::*;
```

---

## Summary

- **Package** — Cargo project with a `Cargo.toml`
- **Crate** — binary (has `main`) or library (no `main`), one compilation unit
- **Module** — branch in the tree, maps to a file or inline block
- **Path** — how you navigate the tree (`::` = `/`, `super` = `../`, `crate` = root)
- **`pub`** — opens visibility; private by default
- **`use`** — shortcut to a path, scope-local
- **`pub use`** — shortcut that's also part of your public API
