# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is a personal learning repository for working through [The Rust Programming Language Book](https://doc.rust-lang.org/book/). Code examples and notes are organized by chapter under `ch/`.

## Commands

Each chapter project is a standalone Cargo crate. Navigate to the relevant project directory before running commands.

```bash
# Build
cargo build

# Build and run
cargo run

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Compile a standalone .rs file (non-Cargo examples)
rustc main.rs && ./main
```

## Structure

- `ch/` — Chapter-by-chapter code examples, each subdirectory may contain standalone `.rs` files or full Cargo projects
- `projects/` — Longer projects (e.g., the grep tool from Ch. 12, multithreaded web server from Ch. 21)
- `scratch/playground/` — Throwaway experiments
- `rust-lang.md` — Personal notes and chapter roadmap

## Rust Edition

New Cargo projects should use `edition = "2024"` in `Cargo.toml`.

## Learning Workflow

This repo is for working through [The Rust Programming Language Book](https://doc.rust-lang.org/book/) with Claude beside me. The goal is to move at a good pace — not rushed, not slow — doing all examples and projects from the book.

**Session flow:**
1. User says which chapter/section to work on
2. Claude fetches the book page directly (no copy-pasting)
3. Work through examples together, code goes in `ch/`
4. At end of chapter (or natural stopping point), Claude updates `notes/chXX-name.md`

**Role split:**
- User writes all the code and runs all commands (cargo, rustc, etc.) — learning by doing
- Claude is a companion for comprehension, explanation, and discussion
- Claude may edit notes/CLAUDE.md but does NOT run commands or write code unless explicitly asked

**Notes format (`notes/`):**
- One file per chapter, e.g. `notes/ch01-getting-started.md`
- Core concepts for the chapter (not exhaustive — just what matters)
- Things the user flags as interesting or important mid-session
- Keep them tight and scannable

**Claude's responsibilities:**
- Keep notes up to date after each chapter
- Update this CLAUDE.md file as the project evolves (new patterns, progress tracking, anything worth preserving across sessions)
- Track overall progress (see below)

## Progress

- [x] Ch 1 — Getting Started → `notes/ch01-getting-started.md`
- [ ] Ch 2 — Programming a Guessing Game
- [ ] Ch 3 — Common Programming Concepts
- [ ] Ch 4 — Understanding Ownership
- [ ] Ch 5 — Using Structs
- [ ] Ch 6 — Enums and Pattern Matching
- [ ] Ch 7 — Packages, Crates, and Modules
- [ ] Ch 8 — Common Collections
- [ ] Ch 9 — Error Handling
- [ ] Ch 10 — Generic Types, Traits, and Lifetimes
- [ ] Ch 11 — Writing Automated Tests
- [ ] Ch 12 — An I/O Project (grep)
- [ ] Ch 13 — Iterators and Closures
- [ ] Ch 14 — More about Cargo
- [ ] Ch 15 — Smart Pointers
- [ ] Ch 16 — Fearless Concurrency
- [ ] Ch 17 — Async and Await
- [ ] Ch 18 — OOP in Rust
- [ ] Ch 19 — Patterns and Matching
- [ ] Ch 20 — Advanced Features
- [ ] Ch 21 — Multithreaded Web Server
