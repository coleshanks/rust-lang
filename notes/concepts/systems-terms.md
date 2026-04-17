# Systems Terms — Latency, Bandwidth, and Related Concepts

These words come up constantly in programming, networking, and embedded work. They get used loosely in CS contexts, but you can understand them more precisely because of the signals background.

---

## Bandwidth

**In signals (ELEC 301):** The range of frequencies a channel can carry — literally Hz to Hz. A channel with 20 Hz to 20 kHz bandwidth can carry signals across that entire frequency range. Wider band = more information capacity. This is the physical, measurable thing.

**In CS/networking:** Borrowed from signals theory. Means the *maximum data throughput* of a link — how much data can move through it per unit time. Usually measured in bits per second (bps, Mbps, Gbps).

The connection is real: Shannon's theorem (信号と情報理論) says channel capacity is directly proportional to bandwidth in Hz. Wider frequency range → more capacity → higher data rate. CS people stripped out the Hz part and kept just the "capacity" sense.

**Examples:**
- A gigabit ethernet port has 1 Gbps bandwidth
- Your home internet might have 100 Mbps download bandwidth
- A USB 3.0 cable has ~5 Gbps bandwidth

**Analogy:** Bandwidth is the width of a pipe. Wider pipe = more water can flow at once.

---

## Latency

**What it is:** The time delay between sending something and receiving it — or between requesting something and getting a response. Measured in milliseconds (ms), microseconds (µs), or nanoseconds (ns) depending on context.

**Examples:**
- Ping to a server in Japan from Japan: ~5ms
- Ping to a server in the US from Japan: ~120ms
- RAM access: ~100ns
- SSD read: ~100µs
- Spinning disk read: ~5ms
- L1 cache hit: ~1ns

**Analogy:** Latency is how long it takes for the first drop of water to arrive after you turn on the tap. Doesn't matter how wide the pipe is.

---

## Bandwidth vs Latency

They're independent. You can have:
- **High bandwidth, high latency** — a satellite internet link. Huge data rate, but the signal has to travel to space and back (~600ms round trip). Streaming works fine; interactive stuff feels laggy.
- **Low bandwidth, low latency** — a serial connection to a microcontroller. Very slow, but responsive. Your RP2040 UART link will be like this.
- **High bandwidth, low latency** — local gigabit LAN. Fast and responsive.
- **Low bandwidth, high latency** — a slow distant connection. Worst of both.

**Why this matters for async:** async/await is designed for I/O-bound work — things where you're waiting on latency (waiting for a response), not limited by bandwidth (not trying to push more data through). You spend most of your time waiting, so the runtime fills that time with other work.

---

## Throughput

Closely related to bandwidth but slightly different:
- **Bandwidth** = the theoretical maximum capacity of a link
- **Throughput** = the actual data rate you achieve in practice

Throughput ≤ bandwidth, always. Things that eat into it: protocol overhead, congestion, retransmission, CPU bottlenecks.

**Example:** A 1 Gbps ethernet port might only sustain 940 Mbps throughput due to TCP/IP overhead.

---

## Jitter

Variation in latency over time. If your ping is usually 20ms but sometimes spikes to 80ms, that's jitter.

Matters a lot for:
- Real-time audio/video (Zoom, VoIP) — irregular packet arrival causes choppy audio
- Games — inconsistent frame timing feels worse than consistently high latency
- Embedded systems — if you need something to happen every 10ms exactly, jitter is a problem

---

## Round-Trip Time (RTT)

The time for a message to go out and a response to come back. A ping measures RTT.

One-way latency = RTT / 2 (approximately, assuming symmetric paths).

---

## Blocking vs Non-Blocking (I/O)

**Blocking I/O:** Your thread sits and waits until the data arrives. Simple to write, wastes CPU time during the wait.

**Non-blocking I/O:** You make the request, then your thread goes off and does other things. You get notified (or check) when the data is ready. More complex, but efficient when you have many concurrent operations.

Async/await is syntactic sugar over non-blocking I/O — the compiler generates the state machine so you can write code that *looks* blocking but actually yields control during waits.

---

## Buffer

Temporary storage used to hold data in transit between two places running at different speeds or rates.

**Examples:**
- A serial receive buffer on a microcontroller — bytes arrive from UART and sit in a buffer until your code reads them. If your code is too slow, the buffer fills and bytes get dropped.
- A network socket buffer — incoming packets queue up until your program reads them.
- Audio buffer — a small chunk of audio queued up to smooth over timing variation.

**Buffer overflow** (in this context, not the security sense) — the buffer fills up and incoming data starts getting dropped because there's nowhere to put it.

---

## Summary Table

| Term | One-line definition | Unit |
|---|---|---|
| Bandwidth | Max data capacity of a link | bps / Mbps / Gbps |
| Latency | Delay from send to receive | ms / µs / ns |
| Throughput | Actual data rate achieved in practice | bps (always ≤ bandwidth) |
| Jitter | Variation in latency over time | ms |
| RTT | Time for request + response round trip | ms |
| Buffer | Temporary storage between mismatched speeds | bytes |
