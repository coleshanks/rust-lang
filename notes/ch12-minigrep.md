# Ch 12 — An I/O Project: minigrep

Building a CLI tool that searches a file for a string — a simplified `grep`. Pulls together concepts from Ch 7–11.

```bash
cargo run -- searchstring filename.txt
```

---

## Reading Command Line Args

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
}
```

- `env::args()` returns an iterator over the CLI args
- `.collect()` needs a type annotation — it can't infer what collection you want
- `args[0]` is always the binary name; actual args start at `args[1]`
- Use `env::args_os()` instead if args might contain invalid Unicode

The `--` in `cargo run -- foo bar` separates cargo's args from the program's args.

---

## Reading a File

```rust
use std::fs;

let contents = fs::read_to_string(file_path)
    .expect("Should have been able to read the file");
```

- `fs::read_to_string` opens the file and returns `Result<String>`
- `.expect()` is fine early on; gets replaced with proper error handling later

---

## Refactoring for Modularity

### The problem with a big `main`

- Hard to test
- Multiple responsibilities mixed together
- Config vars and logic vars all in one scope

### Pattern for binary crates

- `main.rs` — thin: parse args, call `run`, handle errors
- `lib.rs` — all real logic: `Config`, `run`, `search`

### Config struct

Group related config into a struct instead of loose variables:

```rust
pub struct Config {
    pub query: String,
    pub file_path: String,
}
```

### `Config::build` returning `Result`

`new` by convention never fails — use `build` when it can:

```rust
impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        Ok(Config {
            query: args[1].clone(),
            file_path: args[2].clone(),
        })
    }
}
```

Handle in `main` with `unwrap_or_else`:

```rust
let config = Config::build(&args).unwrap_or_else(|err| {
    eprintln!("Problem parsing arguments: {err}");
    process::exit(1);
});
```

`process::exit(1)` terminates immediately with a nonzero exit code — no unwinding, no destructors. Cleaner than `panic!` for user-facing errors because it doesn't print the ugly "thread panicked" message.

### `run` function

Extract all the program logic from `main` into `run`:

```rust
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;
    for line in search(&config.query, &contents) {
        println!("{line}");
    }
    Ok(())
}
```

- `Box<dyn Error>` — a trait object that can hold any error type. Flexible when you don't know the exact error type at compile time.
- `?` propagates the error up instead of panicking
- `Ok(())` — returning unit wrapped in Ok, just to satisfy the `Result` return type

Handle in `main` with `if let`:

```rust
if let Err(e) = run(config) {
    eprintln!("Application error: {e}");
    process::exit(1);
}
```

Use `if let` here (not `unwrap_or_else`) because you only care about the error case — there's no value to unwrap from success.

---

## The search Function (TDD)

TDD cycle: write a failing test → run it to confirm it fails for the right reason → implement just enough to pass → refactor → repeat.

Write the test first, then implement:

```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}
```

**Lifetime `'a` on `contents` and return type** — tells the compiler the returned slices point into `contents`, not `query`. The compiler needs this to verify the returned references are valid at the call site.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";
        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }
}
```

---

## Environment Variables

Add an `IGNORE_CASE` env var for case-insensitive search:

```rust
pub ignore_case: bool,

// in Config::build:
let ignore_case = env::var("IGNORE_CASE").is_ok();
```

- `env::var()` returns `Result` — `.is_ok()` is true if the variable is set, regardless of its value
- Usage: `IGNORE_CASE=1 cargo run -- query file.txt`
- The variable name (`IGNORE_CASE`) is not special — it's just what your code looks for. OS-level vars like `PATH` and `HOME` are set automatically; yours are custom conventions for your program.
- Setting inline (`IGNORE_CASE=1 cargo run ...`) only applies to that one command. `export IGNORE_CASE=1` persists for the session.

Case-insensitive search implementation:

```rust
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase(); // shadows original, now a String
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }
    results
}
```

- `query` is shadowed here — `to_lowercase()` returns a `String`, not a `&str`
- Method chaining goes left to right: `line.to_lowercase()` runs first, returns a `String`, then `.contains(&query)` is called on that
- `&query` in `.contains(&query)` coerces the `String` back to `&str` for the comparison
- The original `line` (not the lowercased version) is pushed to results — you compare lowercase but return the real line

---

## stderr vs stdout

- `println!` → stdout
- `eprintln!` → stderr

Use `eprintln!` for error messages so they still show on screen when stdout is redirected:

```bash
cargo run > output.txt          # errors still visible, successful output goes to file
cargo run -- to poem.txt > output.txt  # results go to file, nothing on screen
```

---

## Final Structure

```
minigrep/
├── Cargo.toml
├── poem.txt
└── src/
    ├── main.rs   ← parse args, call run, handle top-level errors
    └── lib.rs    ← Config, run, search, search_case_insensitive, tests
```

`main.rs` stays thin and untestable-by-design. All logic moves to `lib.rs` where it can be unit tested.

---

## main.rs vs lib.rs — Responsibility Split

| | `main.rs` | `lib.rs` |
|---|---|---|
| **Purpose** | Binary entry point | Library — all real logic |
| **Contains** | arg parsing, `Config::build` call, `run` call, top-level error handling | `Config`, `run`, `search`, tests |
| **Testable?** | No — `main` can't be unit tested | Yes — everything here can be tested |
| **Reusable?** | No | Yes — other crates can `use minigrep::...` |
| **Should be** | So thin you can verify it by reading | Where all the interesting code lives |

### When lib.rs gets big — split into submodules

```
src/
├── main.rs
├── lib.rs       ← declares modules, re-exports public API
├── config.rs    ← Config struct and build logic
├── search.rs    ← search / search_case_insensitive
└── output.rs    ← formatting, printing
```

Each submodule is declared in `lib.rs` with `mod config;` etc. The rule: if it needs a test, it belongs in `lib.rs` or a submodule of it — not in `main.rs`.

---

## Full Code Walkthrough

A line-by-line breakdown of the finished project.

---

### `main.rs`

```rust
use std::env;
use std::fs;
use std::process;
use std::error::Error;
use minigrep::search;
use minigrep::search_case_insensitive;
```

Imports. `env` for CLI args and env vars, `fs` for reading files, `process` for `exit()`, `Error` for the trait object in `run`'s return type. The two `minigrep::` imports pull in the public functions from `lib.rs` — the crate name matches the package name in `Cargo.toml`.

---

```rust
fn main() {
    let args: Vec<String> = env::args().collect();
```

`env::args()` returns an iterator over the command-line arguments as `String`s. `.collect()` needs a type annotation (`Vec<String>`) because it can't infer what kind of collection you want. `args[0]` is always the binary name — actual user args start at `args[1]`.

---

```rust
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
```

`Config::build` returns a `Result`. If it's `Ok`, `unwrap_or_else` unwraps the value and assigns it to `config`. If it's `Err`, the closure runs — prints the error to stderr (`eprintln!`), then exits the process with code `1`. The closure takes the error value as `err`, which is the `&'static str` returned from `build`.

`eprintln!` instead of `println!` means error messages go to stderr, so they still show up even when stdout is redirected to a file.

---

```rust
    println!("Searching for {}", config.query);
    println!("In file {}", config.file_path);

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1)
    }
}
```

`run` returns `Result<(), Box<dyn Error>>`. `if let Err(e)` only matches the error case — there's nothing to do on success (it returns `()`), so `if let` is cleaner than `unwrap_or_else` here. Same pattern: print to stderr and exit on failure.

---

```rust
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}
```

Groups the three config values into one struct instead of passing them as separate variables. All fields are `pub` so `lib.rs` can access them. `query` and `file_path` are owned `String`s — `build` clones them out of `args`. `ignore_case` is a plain `bool` read from the environment.

---

```rust
fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;
```

Takes ownership of `config`. `fs::read_to_string` opens the file and returns its full contents as a `String`, or an error. The `?` operator propagates any error up to the caller (main) instead of panicking. `Box<dyn Error>` as the error type means "any type that implements the `Error` trait" — flexible enough to hold whatever `read_to_string` might return.

---

```rust
    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{line}");
    }

    Ok(())
}
```

The `if/else` is an expression — it evaluates to whichever function's return value matches the condition, and that gets bound to `results`. Both branches return `Vec<&str>`, so the types line up. Then just iterate and print. `Ok(())` returns unit wrapped in `Ok` to satisfy the `Result<(), _>` return type — nothing meaningful to return on success.

---

```rust
impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config { query, file_path, ignore_case })
    }
}
```

Takes a slice of `String`s (not a `Vec` — slices are more flexible). Returns `Result` so it can report failure without panicking. The error type is `&'static str` — a string literal baked into the binary, valid for the entire program lifetime.

`args[1]` and `args[2]` are cloned into owned `String`s because `Config` needs to own its data independently of `args`.

`env::var("IGNORE_CASE")` returns `Ok` if the variable is set (to anything), `Err` if it isn't. `.is_ok()` converts that to a plain `bool`. The variable name is not special — it's just what this program looks for.

---

### `lib.rs`

```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}
```

The lifetime `'a` connects `contents` to the return type. It tells the compiler: the `&str` slices in the returned `Vec` are borrowed from `contents`, not from `query`. Without this, the compiler can't verify the returned references are valid after the function returns.

`contents.lines()` gives an iterator over the lines of the string (splitting on newlines). `line.contains(query)` checks for a substring match. The matching lines are pushed as slices — they point directly into `contents`, no allocation.

---

```rust
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }
    results
}
```

Same structure as `search`, with two differences:

1. `let query = query.to_lowercase()` — shadows the parameter with a new `String`. Type changes from `&str` to `String`. Both sides of the comparison need to be the same case, so you lowercase both.
2. `line.to_lowercase().contains(&query)` — chains left to right: `to_lowercase()` on `line` returns a temporary `String`, then `.contains(&query)` runs on that. `&query` coerces the `String` back to `&str` for the comparison.

The original `line` (not the lowercased version) is what gets pushed — you want to return the real text, just matched case-insensitively.

---

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() { ... }

    #[test]
    fn case_insensitive() { ... }
}
```

`#[cfg(test)]` means this module only compiles when running `cargo test` — it doesn't ship in the binary. `use super::*` imports everything from the parent module (`lib.rs`) into the test scope. Two tests: one verifies `search` doesn't match across cases (`"duct"` matches `"productive"` but not `"Duct"`), the other verifies `search_case_insensitive` matches regardless of case (`"rUsT"` matches both `"Rust:"` and `"Trust me."`).
