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

Note: `query` is shadowed here — `to_lowercase()` returns a `String`, not a `&str`.

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
