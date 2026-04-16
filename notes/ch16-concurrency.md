# Ch 16 — Fearless Concurrency

Rust's ownership and type system catch most concurrency bugs at **compile time** rather than runtime. The goal: write concurrent code without fear of data races or use-after-free errors.

Two related concepts:
- **Concurrent** — parts of a program execute independently (may interleave)
- **Parallel** — parts execute literally at the same time (multiple cores)

**Concurrency vs parallelism on real hardware:** A single core executes one instruction at a time — parallelism is physically impossible on one core. What feels like simultaneous execution is the OS scheduler context-switching between processes so fast it seems seamless. True parallelism requires multiple cores. Modern CPUs do both: multiple cores running things genuinely in parallel, each core context-switching between competing processes. Hardware interrupts (keyboard, network, disk) add another layer — the CPU can be pulled away from whatever it's doing mid-instruction to handle an event, then resume. The result feels smooth even though it's orchestrated chaos underneath.

---

## 16.1 — Threads

### Common concurrency bugs

**Race condition** — two threads access shared data in an unpredictable order, producing different results depending on timing:

```
Thread 1 reads counter (0)
Thread 2 reads counter (0)   ← both read before either writes
Thread 1 writes counter + 1 (1)
Thread 2 writes counter + 1 (1)  ← should be 2, but both started from 0
```

Result: you expected `2`, you got `1`. Happens non-deterministically — sometimes it works fine.

**Deadlock** — two threads are each waiting for a lock the other holds, so neither can proceed:

```
Thread 1 holds lock A, waiting for lock B
Thread 2 holds lock B, waiting for lock A
→ both blocked forever
```

Rust prevents race conditions at compile time via ownership. Deadlocks are a runtime problem — Rust can't save you from them.

**Heisenbugs** — bugs that only appear under specific timing conditions, disappear when you add logging or run in debug mode (because that changes the timing). Notoriously hard to reproduce and fix. Classic concurrency problem in languages without compile-time guarantees.

---

### The basics

Rust uses a **1:1 thread model** — one OS thread per language thread. Spawn a thread with `thread::spawn`, passing a closure:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    thread::spawn(|| {
        for i in 1..10 {
            println!("spawned thread: {i}");
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..5 {
        println!("main thread: {i}");
        thread::sleep(Duration::from_millis(1));
    }
    // spawned thread may be cut off when main exits
}
```

**Important:** when the main thread ends, all spawned threads are killed immediately — even if they haven't finished.

### JoinHandle — waiting for threads

`thread::spawn` returns a `JoinHandle<T>`. Call `.join()` to block until the thread finishes:

```rust
let handle = thread::spawn(|| {
    for i in 1..10 {
        println!("spawned: {i}");
        thread::sleep(Duration::from_millis(1));
    }
});

for i in 1..5 {
    println!("main: {i}");
    thread::sleep(Duration::from_millis(1));
}

handle.join().unwrap(); // wait here until spawned thread is done
```

Where you call `join()` matters:
- After main loop → threads interleave, then main waits for spawned to finish
- Before main loop → spawned thread runs fully first, then main loop runs

### `move` closures with threads

Spawned threads often need data from the parent scope. You can't just borrow it — the compiler can't prove the reference stays valid for the thread's lifetime. Use `move` to transfer ownership:

```rust
let v = vec![1, 2, 3];

// Without move — won't compile:
// thread::spawn(|| println!("{v:?}"));

// With move — v is owned by the spawned thread:
let handle = thread::spawn(move || {
    println!("{v:?}");
});

handle.join().unwrap();
// v is no longer accessible here — it was moved
```

Why Rust forces this: if you could borrow across threads, this would be valid:

```rust
let v = vec![1, 2, 3];
let handle = thread::spawn(|| println!("{v:?}")); // borrows v
drop(v); // use-after-free! spawned thread still has a ref
handle.join().unwrap();
```

`move` makes it explicit — the thread owns the data, so there's no ambiguity.

---

## 16.2 — Message Passing with Channels

> "Do not communicate by sharing memory; instead, share memory by communicating." — Go docs

A **channel** has two ends:
- **transmitter (`tx`)** — sends data
- **receiver (`rx`)** — receives data

`mpsc` = **multiple producer, single consumer** — many senders, one receiver.

### Basic channel usage

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let msg = String::from("hello");
        tx.send(msg).unwrap();
        // msg is moved into send() — can't use it here anymore
    });

    let received = rx.recv().unwrap();
    println!("Got: {received}");
}
```

- `tx.send(val)` — takes ownership of `val`, returns `Result`
- `rx.recv()` — **blocks** until a message arrives, returns `Result`
- `rx.try_recv()` — returns immediately (useful if the thread has other work to do)

### Sending multiple values

The receiver can be iterated — it blocks waiting for each value and stops when the channel closes (tx dropped):

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let msgs = vec!["one", "two", "three", "four"];
        for msg in msgs {
            tx.send(msg).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    }); // tx dropped here — channel closes

    for received in rx { // iterates until channel closes
        println!("Got: {received}");
    }
}
```

### Multiple producers

Clone `tx` to create multiple senders into the same receiver:

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone(); // second sender

    thread::spawn(move || {
        tx.send(String::from("from thread 1")).unwrap();
    });

    thread::spawn(move || {
        tx2.send(String::from("from thread 2")).unwrap();
    });

    for received in rx {
        println!("Got: {received}");
    }
}
// output order is non-deterministic
```

---

## 16.3 — Shared State with Mutex

Channels are great, but sometimes you want multiple threads reading/writing the same data directly.

### Mutex — mutual exclusion

A `Mutex<T>` wraps data and enforces that only **one thread can access it at a time**. A thread must acquire the **lock** before accessing, and release it when done.

```rust
use std::sync::Mutex;

fn main() {
    let m = Mutex::new(5);

    {
        let mut num = m.lock().unwrap(); // acquire lock, get MutexGuard<T>
        *num = 6;
    } // MutexGuard dropped here — lock released automatically

    println!("m = {m:?}"); // Mutex { data: 6, .. }
}
```

`m.lock()` returns a `MutexGuard<T>` which:
- Implements `Deref` — so you can use it like a reference to the inner value
- Implements `Drop` — automatically releases the lock when it goes out of scope

### Sharing a Mutex across threads — `Arc<T>`

`Mutex<T>` alone can't be shared across threads — only one owner. `Rc<T>` won't work either because it's not thread-safe. Use `Arc<T>` (atomic reference counting):

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter); // clone the Arc, not the Mutex
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        }); // lock released when num goes out of scope
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap()); // 10
}
```

### `Arc<T>` vs `Rc<T>`

| | `Rc<T>` | `Arc<T>` |
|---|---|---|
| Thread safe | No | Yes |
| Performance | Faster | Slight overhead (atomic ops) |
| Use case | Single-threaded | Multi-threaded |

`Arc` = **Atomic** reference counted. The atomic ops guarantee the reference count is updated safely across threads.

### Deadlocks

`Mutex<T>` can deadlock — thread 1 holds lock A waiting for lock B, thread 2 holds lock B waiting for lock A. Neither can proceed. This is the multithreaded equivalent of the reference cycle problem from ch15. Rust doesn't prevent it at compile time.

### `Mutex<T>`/`Arc<T>` mirrors `RefCell<T>`/`Rc<T>`

| Situation | Interior mutability | Multiple ownership |
|---|---|---|
| Single-threaded | `RefCell<T>` | `Rc<T>` |
| Multi-threaded | `Mutex<T>` | `Arc<T>` |

---

## 16.4 — `Send` and `Sync` Traits

These are **marker traits** — no methods, just signal properties to the compiler.

### `Send`

A type is `Send` if **ownership can safely be transferred between threads**.

- Almost all Rust types are `Send`
- `Rc<T>` is **not** `Send` — its reference count is not atomic, so transferring it across threads could corrupt the count
- `Arc<T>` is `Send`
- Any type composed entirely of `Send` types is automatically `Send`

### `Sync`

A type is `Sync` if **it's safe to be referenced from multiple threads simultaneously**.

Formally: `T: Sync` if `&T: Send` — a shared reference to it can be sent to another thread.

- Primitive types: `Sync`
- `Mutex<T>`: `Sync` (safe to share)
- `Rc<T>`: **not** `Sync`
- `RefCell<T>`: **not** `Sync` (runtime borrow checking isn't thread-safe)

### Manual implementation

You can manually implement `Send` and `Sync` but it requires `unsafe`. Don't do this unless you really know what you're doing — see the Rustonomicon for the details.

In practice: if your type is composed of `Send`/`Sync` types, you get these traits for free. If the compiler says something isn't `Send` or `Sync`, that's a real warning to take seriously.

---

## Summary

| Tool | Purpose | Thread safe |
|---|---|---|
| `thread::spawn` | Create a new OS thread | — |
| `JoinHandle::join` | Wait for thread to finish | — |
| `mpsc::channel` | Send data between threads (message passing) | Yes |
| `Mutex<T>` | Shared mutable data (one thread at a time) | Yes (with `Arc`) |
| `Arc<T>` | Shared ownership across threads | Yes |
| `Send` | Type can be transferred between threads | marker trait |
| `Sync` | Type can be referenced from multiple threads | marker trait |
