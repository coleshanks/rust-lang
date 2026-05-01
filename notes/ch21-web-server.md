# Ch 21 — Final Project: Multithreaded Web Server

The capstone project. Build a web server from scratch — TCP listener, HTTP parsing, a thread pool, and graceful shutdown. Pulls together: ownership, traits, closures, channels, `Arc<Mutex<T>>`, and `Drop`.

Project lives in `projects/hello/`.

---

## 21.1 — Single-Threaded Server

### TCP and HTTP

Two protocols stacked on each other:
- **TCP** — handles raw bytes over the network, guarantees delivery and ordering
- **HTTP** — rides on top of TCP, defines what those bytes mean (request/response format)

Both are request-response: client sends a request, server sends a response.

### Listening for connections

```rust
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection established!");
    }
}
```

- `TcpListener::bind("127.0.0.1:7878")` — bind to localhost, port 7878. Returns `Result`, hence the `unwrap`.
- `127.0.0.1` — loopback address, your own machine
- Port 7878 — arbitrary, avoids needing root (80/443 require elevated permissions)
- `listener.incoming()` — iterator of `Result<TcpStream>`. Each item is a new connection attempt.
- The `stream` variable represents one open TCP connection. It drops (and closes) when it goes out of scope at the end of each loop iteration.

### Reading the HTTP request

```rust
use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty()) // HTTP headers end at the first blank line
        .collect();

    println!("Request: {http_request:#?}");
}
```

- `BufReader` wraps the stream — buffers reads for efficiency, gives us `.lines()`
- `.lines()` returns an iterator of `Result<String>` — one per line
- `.take_while(|line| !line.is_empty())` — HTTP headers end at a blank line (`\r\n\r\n`), so stop there
- The collected `Vec<String>` is the full header block

### HTTP request format

```
GET / HTTP/1.1\r\n
Host: 127.0.0.1:7878\r\n
User-Agent: Mozilla/5.0\r\n
Accept: text/html\r\n
\r\n
```

- First line: `METHOD path HTTP-VERSION`
- Then headers, one per line
- Blank line signals end of headers
- Optional body after (for POST etc.)

`\r\n` is CRLF — carriage return + line feed. HTTP requires it as a line terminator.

### Writing a response

HTTP response format:

```
HTTP/1.1 200 OK\r\n
Content-Length: 123\r\n
\r\n
<body>
```

Minimal response — just a status line, no body:

```rust
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
}
```

`write_all` takes `&[u8]` — `.as_bytes()` converts the string slice.

### Serving HTML

```rust
use std::fs;

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("hello.html").unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
```

`Content-Length` tells the browser how many bytes to expect. Without it, the browser doesn't know when the body ends.

### Routing: 200 vs 404

Read just the first line (the request line), branch on it:

```rust
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
```

The refactor pulls the only two things that differ (status line and filename) into a tuple, then the read/format/write stays in one place. Clean.

**At this point the server works — one request at a time.** If a request takes a long time, every subsequent connection has to wait.

---

## 21.2 — Turning it Multithreaded

### The problem

Demonstrate the bottleneck by adding a `/sleep` route:

```rust
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1"     => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5)); // blocks for 5s
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    // ... rest of response
}
```

Hit `/sleep` in one tab, then `/` in another — the second one waits. That's the single-threaded problem.

### The solution: thread pool

A **thread pool** is a fixed set of pre-spawned threads that sit idle waiting for work. Incoming jobs are queued via a channel and grabbed by whichever thread is free.

Why not just `thread::spawn` per request? Unbounded spawning under heavy load could create thousands of threads — exhausting system resources and crashing the server. The pool caps it.

### API-first design

Start by writing the `main.rs` as if the API already exists, then build to match:

```rust
use hello::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4); // 4 worker threads

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}
```

`pool.execute` signature mirrors `thread::spawn` — takes a closure that runs on a worker thread.

### `execute` trait bounds

```rust
pub fn execute<F>(&self, f: F)
where
    F: FnOnce() + Send + 'static,
{
```

- `FnOnce()` — called exactly once, no args, no return
- `Send` — can be sent across thread boundaries
- `'static` — no borrowed references (we don't know when the thread will run, so nothing can borrow from the caller's frame)

### The Worker

`ThreadPool` doesn't hold `JoinHandle`s directly — it holds `Worker`s. A `Worker` is just an ID + a thread. The indirection exists because we'll need to do extra bookkeeping during shutdown.

```rust
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {id} got a job; executing.");
                job();
            }
        });

        Worker { id, thread }
    }
}
```

Each worker spins up a thread that loops forever: lock the receiver, grab a job, execute it. The `lock()` + `recv()` pattern means only one worker grabs each job.

### Why `Arc<Mutex<Receiver>>`

The channel receiver (`mpsc::Receiver`) needs to be shared across all worker threads — but it's not `Clone` and it's not `Sync`. Solution:

- `Mutex<Receiver>` — only one thread can receive at a time (prevents multiple workers grabbing the same job)
- `Arc<Mutex<Receiver>>` — multiple owners (one per worker), reference counted, safe to send across threads

`Arc::clone(&receiver)` gives each worker its own handle to the same underlying mutex.

### Why not `while let` for the receive loop

```rust
// Problematic version
while let Ok(job) = receiver.lock().unwrap().recv() {
    job(); // lock is still held during job execution!
}
```

With `while let`, the `MutexGuard` from `.lock()` lives until the end of the `while let` expression — which in this case is the end of the loop body. That means the mutex is held while the job runs, blocking all other workers.

The `let job = receiver.lock().unwrap().recv().unwrap();` version drops the guard immediately after `recv()` returns, so other workers can grab the next job while this one is executing.

### Full ThreadPool implementation

```rust
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>; // type alias — any one-shot closure

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size); // pre-allocate exact capacity

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}
```

`type Job = Box<dyn FnOnce() + Send + 'static>` — type alias for the trait object. Makes the channel type readable.

---

## 21.3 — Graceful Shutdown

When the server stops, worker threads are still in their `loop`, blocked on `recv()`. They'll be killed abruptly. Graceful shutdown means:
1. Signal workers to stop waiting for new jobs
2. Let any in-progress job finish
3. Join all threads before the process exits

### The signal: drop the sender

An `mpsc` channel closes when all senders are dropped. When the channel closes, `recv()` returns `Err` instead of blocking. Workers check for this and break.

To drop the sender on shutdown, wrap it in `Option` so we can `.take()` it:

```rust
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>, // Option so we can take() it
}
```

`execute` now unwraps it:

```rust
pub fn execute<F>(&self, f: F)
where
    F: FnOnce() + Send + 'static,
{
    let job = Box::new(f);
    self.sender.as_ref().unwrap().send(job).unwrap();
}
```

### Worker thread: wrapped in Option too

To call `join()`, we need ownership of the `JoinHandle` — we can't get that from a `&mut`. Wrapping it in `Option` lets us `.take()` it out:

```rust
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>, // Option so we can take() to join
}
```

### Updated worker loop: handle channel close

```rust
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break; // sender dropped — channel closed — exit loop
                    }
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}
```

### `Drop` for ThreadPool

```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take()); // drop sender → closes channel → workers get Err → break

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap(); // wait for thread to finish
            }
        }
    }
}
```

Order matters:
1. Drop the sender first — this signals workers to stop
2. Then join — waits for each worker to actually finish its current job and exit the loop

If you join before dropping the sender, workers are still waiting on `recv()` and `join()` blocks forever.

### Testing graceful shutdown

Limit the server to 2 requests so it shuts down cleanly:

```rust
for stream in listener.incoming().take(2) {
    ...
}
println!("Shutting down.");
// pool drops here → Drop runs → workers finish and join
```

Expected output:

```
Worker 0 got a job; executing.
Worker 3 got a job; executing.
Shutting down.
Shutting down worker 0
Worker 1 disconnected; shutting down.
Worker 2 disconnected; shutting down.
Worker 3 disconnected; shutting down.
Worker 0 disconnected; shutting down.
Shutting down worker 1
Shutting down worker 2
Shutting down worker 3
```

Workers that weren't busy get the disconnect message and exit. Workers that were busy finish their job first, then exit. All threads join before the process ends.

---

## Complete Final Code

### `src/lib.rs`

```rust
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

// The public API — main.rs only sees ThreadPool, not Worker or Job.
pub struct ThreadPool {
    workers: Vec<Worker>,
    // Option so we can call .take() in Drop to get ownership and drop the sender,
    // which closes the channel and signals workers to stop.
    sender: Option<mpsc::Sender<Job>>,
}

// Type alias for a heap-allocated closure:
// - FnOnce() — called exactly once, no args, no return value
// - Send     — can be moved across thread boundaries
// - 'static  — no borrowed references (we don't know when the thread will run it)
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // Create a single mpsc channel. All workers share one receiver end.
        // The sender end stays in ThreadPool; workers hold clones of the receiver via Arc.
        let (sender, receiver) = mpsc::channel();

        // Wrap receiver in Arc<Mutex<>> so multiple workers can share it safely:
        // - Mutex  — only one worker locks and receives at a time (no double-grabbing a job)
        // - Arc    — multiple owners (one per worker), reference counted
        let receiver = Arc::new(Mutex::new(receiver));

        // Pre-allocate exactly `size` slots — no reallocations as we push workers.
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // Give each worker its own Arc handle to the shared receiver.
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender), // wrapped in Some so Drop can .take() it
        }
    }

    // Mirror of thread::spawn's signature — takes any closure satisfying the same bounds.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Box the closure to make it a trait object (uniform size for the channel).
        let job = Box::new(f);

        // Send the job down the channel. Whichever worker is free will pick it up.
        // as_ref() because we can't move out of self; unwrap() because sender is always
        // Some while the pool is alive.
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Step 1: drop the sender. This closes the channel.
        // Workers blocked on recv() will now get Err(_) instead of blocking forever.
        // Order matters — must do this BEFORE joining, otherwise join blocks forever.
        drop(self.sender.take());

        // Step 2: join every worker thread — wait for each one to finish its current
        // job and exit its loop before the process ends.
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            // .take() pulls the JoinHandle out of the Option, giving us ownership.
            // We need ownership to call .join() — can't do it from a shared reference.
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// Internal type — not pub. Main.rs doesn't need to know about Workers.
struct Worker {
    id: usize,
    // Option so Drop can call .take() to get ownership of the JoinHandle for .join().
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // Lock the mutex, grab one job from the channel, then immediately
                // release the lock. The MutexGuard drops at the end of this statement —
                // NOT at the end of the loop body — so other workers can grab the next
                // job while this one is executing.
                //
                // (Using `while let Ok(job) = receiver.lock()...recv()` would hold the
                // lock for the entire job execution, blocking all other workers.)
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job(); // run the closure
                    }
                    Err(_) => {
                        // recv() returns Err when the sender has been dropped and the
                        // channel is empty — means the pool is shutting down.
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
```

### `src/main.rs`

```rust
use hello::ThreadPool;
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    // Bind to localhost port 7878. Returns Result — unwrap panics if port is in use.
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Spin up a pool of 4 worker threads. Jobs are queued via a channel inside the pool.
    let pool = ThreadPool::new(4);

    // .take(2) limits to 2 requests so the server shuts down after handling them.
    // This lets Drop run and graceful shutdown be observed. Remove for a real server.
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        // Hand the connection off to a worker thread via the pool.
        // The closure captures `stream` and moves it onto whichever thread picks it up.
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    // Reached after .take(2) exhausts. Pool drops here — Drop signals workers and joins.
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    // Wrap the stream in a BufReader for line-by-line reading.
    let buf_reader = BufReader::new(&stream);

    // Read only the first line — the HTTP request line (e.g. "GET / HTTP/1.1").
    // We don't need the headers for this server.
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // Route based on the request line. Pull out only the two things that differ
    // per route — status line and filename — then handle read/format/write once.
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),

        // Simulates a slow request — blocks this worker thread for 5 seconds.
        // In a single-threaded server this would block ALL requests. With the pool,
        // other workers keep serving while this one sleeps.
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }

        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    // HTTP response format: status line, then headers, then blank line, then body.
    // Content-Length tells the browser exactly how many bytes to expect in the body.
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // write_all takes &[u8] — as_bytes() converts the String, unwrap surfaces any I/O error.
    stream.write_all(response.as_bytes()).unwrap();
}
```

---

## Summary

| Concept | How it's used here |
|---|---|
| `TcpListener` | Binds to a port and accepts incoming TCP connections |
| `BufReader` + `.lines()` | Reads HTTP headers line by line from the stream |
| `write_all` | Sends the HTTP response as bytes |
| Thread pool | Fixed set of worker threads — prevents unbounded spawning |
| `mpsc::channel` | Queue of jobs from main thread to workers |
| `Arc<Mutex<Receiver>>` | Share one receiver across multiple worker threads safely |
| `FnOnce + Send + 'static` | Trait bounds for closures that can be sent to another thread |
| `Option<T>` on fields | Enables `.take()` to get ownership for `join()` / `drop()` |
| `Drop` trait | Signals workers and joins threads on shutdown |
| Channel close → `Err` | How workers know there's no more work coming |
