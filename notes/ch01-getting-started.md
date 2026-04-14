# Ch 1 — Getting Started

## Installation

Rust is managed via `rustup` — handles installs, updates, and toolchain switching.

```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

rustc --version        # verify install
rustup update          # update to latest stable
rustup doc             # open local offline docs
rustup self uninstall  # remove Rust entirely
```

---

## Hello, World

```rust
fn main() {
    println!("Hello, world!");
}
```

```bash
rustc main.rs   # compile
./main          # run
```

Key details:
- `main` is the entry point — always runs first in an executable
- `println!` — the `!` means it's a **macro**, not a function. Macros generate code at compile time.
- Rust is **ahead-of-time compiled** — you can hand someone the binary and they don't need Rust installed

`rustc` is fine for single files but you'd use Cargo for anything real.

---

## Cargo

Cargo is Rust's build system and package manager. Handles building, dependency downloading, and more.

```bash
cargo new hello_cargo   # creates project with src/, Cargo.toml, .gitignore
cargo init              # same but in the current directory (for existing projects)
```

### Cargo.toml

Project manifest — config and dependencies. Uses TOML format.

```toml
[package]
name = "hello_cargo"
version = "0.1.0"
edition = "2024"

[dependencies]
# crates go here, e.g.:
# rand = "0.8"
```

Source lives in `src/`. Top-level is for config, docs, licenses.

### Key Commands

| Command | What it does |
|---|---|
| `cargo build` | Compile (debug build, output in `target/debug/`) |
| `cargo run` | Build + run in one step |
| `cargo check` | Verify it compiles without producing a binary — fast feedback |
| `cargo build --release` | Optimized build (`target/release/`) |

`cargo check` is faster than `cargo build` — use it constantly while writing code.

---

## The `target/` Folder

Cargo dumps build artifacts, metadata, and incremental build state here. It's large and fully reproducible.

- `target/debug/` — unoptimized, what `cargo build` produces, for development
- `target/release/` — optimized, what `cargo build --release` produces, what you'd ship
- In `.gitignore` by default — keep it that way, don't push it

---

## Worth Remembering

- `Cargo.lock` tracks exact dependency versions — auto-managed, don't edit manually
- `edition = "2024"` is the current Rust edition — use it for new projects
- To share a binary: hand someone the single file from `target/release/`, not the whole folder
