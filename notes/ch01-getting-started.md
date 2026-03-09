# Ch 1 — Getting Started

## Core Concepts

- `rustc` compiles single files directly; Cargo is used for everything real
- Cargo creates the project structure (`src/`, `Cargo.toml`, `.gitignore`) with `cargo new`
- `Cargo.toml` — project config (name, version, edition) + dependencies list
- Source lives in `src/`, top-level is for config/docs

## Key Commands

| Command | What it does |
|---|---|
| `cargo new <name>` | Create a new project |
| `cargo build` | Compile (debug build, output in `target/debug/`) |
| `cargo run` | Build + run in one step |
| `cargo check` | Verify it compiles without producing a binary (fast feedback loop) |
| `cargo build --release` | Optimized build for distribution (`target/release/`) |

## Worth Remembering

- `cargo check` is faster than `cargo build` — use it constantly while writing code
- `Cargo.lock` tracks exact dependency versions (auto-managed, don't edit manually)
- `edition = "2024"` is the current Rust edition

## The `target/` Folder

When you build, Cargo dumps a lot into `target/` — incremental build artifacts, metadata, intermediate files. It's large and entirely reproducible, so:
- It's in `.gitignore` by default and should stay there — don't push it to GitHub
- `target/debug/` is what `cargo build` produces — unoptimized, for development only
- `target/release/` is what `cargo build --release` produces — optimized, this is what you'd ship or share with someone
- If you want to share a binary, hand someone the single executable from `target/release/`, not the folder
