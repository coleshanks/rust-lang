# Ch 14 — More About Cargo and Crates.io

Covers advanced Cargo features: release profiles, publishing crates, workspaces, installing binaries, and custom commands.

---

## Release Profiles

Cargo has two built-in profiles:

- `cargo build` → `dev` profile (fast compile, no optimization)
- `cargo build --release` → `release` profile (slow compile, optimized binary)

Default `opt-level` settings:

```toml
[profile.dev]
opt-level = 0   # no optimization — fast to compile

[profile.release]
opt-level = 3   # max optimization — fast to run
```

`opt-level` range is 0–3. Higher = longer compile, faster runtime. You can override in `Cargo.toml`:

```toml
[profile.dev]
opt-level = 1   # some optimization during dev
```

---

## Documentation Comments

Use `///` (three slashes) to write doc comments. Supports Markdown. Generates HTML with `cargo doc --open`.

```rust
/// Adds one to the number given.
///
/// # Examples
///
/// ```
/// let answer = my_crate::add_one(5);
/// assert_eq!(6, answer);
/// ```
pub fn add_one(x: i32) -> i32 {
    x + 1
}
```

Common sections: `# Examples`, `# Panics`, `# Errors`, `# Safety`.

Code examples in doc comments are run as tests with `cargo test` — keeps docs from going stale.

Use `//!` to document the containing item (the crate or module itself), placed at the top of `src/lib.rs`:

```rust
//! # My Crate
//!
//! `my_crate` is a collection of utilities...
```

---

## Re-exporting with `pub use`

Internal module structure doesn't have to match the public API. Use `pub use` to re-export items at a higher level:

```rust
// users can write: use art::PrimaryColor;
// instead of:      use art::kinds::PrimaryColor;

pub use self::kinds::PrimaryColor;
pub use self::kinds::SecondaryColor;
pub use self::utils::mix;
```

Re-exports appear on the documentation front page. Lets you restructure internals without breaking user code.

---

## Publishing to Crates.io

### Setup

```bash
cargo login   # paste your API token from crates.io/me/
```

Token stored at `~/.cargo/credentials.toml`. Keep it secret.

### Required metadata in `Cargo.toml`

```toml
[package]
name = "my_crate"          # must be unique on crates.io
version = "0.1.0"
edition = "2024"
description = "A short description shown in search results."
license = "MIT OR Apache-2.0"   # SPDX identifier required
```

### Publishing

```bash
cargo publish
```

**Publishing is permanent** — versions can't be deleted or overwritten. Crates.io is an archive.

### Yanking

Yanking prevents new projects from depending on a version but doesn't break existing ones with that version in `Cargo.lock`:

```bash
cargo yank --vers 1.0.1       # yank
cargo yank --vers 1.0.1 --undo  # undo
```

Yanking does not delete code. If you accidentally publish secrets, yank and immediately rotate the credentials.

### New versions

Update `version` in `Cargo.toml` following semver, then `cargo publish` again.

Semver (`major.minor.patch`):
- **patch** — bug fixes, no API changes (`0.1.0` → `0.1.1`)
- **minor** — new features, backwards compatible (`0.1.0` → `0.2.0`)
- **major** — breaking changes (`0.1.0` → `1.0.0`)

In `Cargo.toml`, `rand = "0.8.5"` means "0.8.5 or any compatible version" — Cargo will accept patch and minor updates but not a new major version.

---

## Workspaces

A workspace is a set of packages that share one `Cargo.lock` and one `target/` directory.

Good for splitting a large project into multiple related crates that you develop together.

### Setup

Top-level `Cargo.toml` (no `[package]`, just `[workspace]`):

```toml
[workspace]
resolver = "3"
members = ["adder", "add_one"]
```

### Structure

```
add/
├── Cargo.lock          ← shared
├── Cargo.toml          ← workspace root
├── target/             ← shared build output
├── adder/
│   ├── Cargo.toml
│   └── src/main.rs
└── add_one/
    ├── Cargo.toml
    └── src/lib.rs
```

### Declaring dependencies between crates

Cargo doesn't assume workspace crates depend on each other — you have to be explicit:

```toml
# adder/Cargo.toml
[dependencies]
add_one = { path = "../add_one" }
```

### External dependencies

Each crate declares its own dependencies. All crates in the workspace share the same `Cargo.lock`, so they all use the same version of any shared dependency — but each crate must still explicitly list what it uses.

### Running and testing

```bash
cargo run -p adder         # run a specific binary
cargo test -p add_one      # test a specific crate
cargo test                 # test everything
```

### Publishing

Each crate must be published separately:

```bash
cargo publish -p add_one
```

---

## Installing Binaries

```bash
cargo install ripgrep       # install a binary crate
cargo search ripgrep        # search crates.io from the terminal
cargo update                # update dependencies within semver constraints
```

`cargo install` puts the binary in `~/.cargo/bin/` — needs to be in `$PATH`.

Only works for crates with binary targets (`src/main.rs` or equivalent). Not for library-only crates.

Not a replacement for system package managers — mainly for Rust dev tools.

---

## Custom Cargo Commands

If a binary named `cargo-something` is in your `$PATH`, you can run it as:

```bash
cargo something
```

Custom commands show up in `cargo --list`. Distribute them via `cargo install`.
