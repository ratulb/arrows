---
layout: page
title: Arrows
---

**An actor framework in Rust with message durability and ingestion-order processing.**

Arrows pairs the actor model with an embedded SQLite backing store, giving you **at-least-once delivery**, **ordered message processing**, and **automatic actor recovery** across restarts — without external infrastructure.

---

## Features

- **Durable messages** — every message is persisted to SQLite before delivery.
- **Ingestion-order guarantees** — messages are delivered in the exact order they entered the system; out-of-sequence messages wait until prior messages are consumed.
- **Actor recovery** — producers and actor state survive restarts; actors resume where they left off.
- **Remote messaging** — send messages to actors on other nodes via IP:port addressing.
- **Text + binary payloads** — messages carry either text or arbitrary binary blobs.
- **Batched sends** — group multiple messages to multiple actors in a single `send!` call.
- **Panic tolerance** — actors tolerate up to 3 panics before eviction.
- **Embedded** — single-binary deployment with SQLite bundled.

---

## Quick start

```bash
git clone https://github.com/ratulb/arrows
cd arrows
cargo build --release
```

Open three terminals:

```bash
# Terminal 1 — register actors
./register.sh

# Terminal 2 — start the listener
./server.sh

# Terminal 3 — send messages
./send.sh
```

The actors begin receiving and processing messages immediately.

---

## Architecture overview

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Actor 1   │     │   Actor 2    │     │   Actor N   │
│  (local)    │     │  (remote)    │     │  (local)    │
└──────┬──────┘     └──────┬───────┘     └──────┬──────┘
       │                   │                     │
       └───────────────────┼─────────────────────┘
                           │
                    ┌──────┴──────┐
                    │  Messenger  │
                    │  (routing)  │
                    └──────┬──────┘
                           │
              ┌────────────┴────────────┐
              │   SQLite backing store  │
              │   (message persistence) │
              └─────────────────────────┘
```

1. **Define an actor** — implement `Actor::receive()` for message handling, plus `post_start()` / `pre_shutdown()` lifecycle hooks.
2. **Register a producer** — implement `Producer::produce()` to create actor instances. Producers are serialized and stored in SQLite.
3. **Define an instance** — call `define_actor!("name", producer)` to register an actor. It starts receiving messages immediately.
4. **Send messages** — `send!("name", msg1, msg2)` dispatches to local or remote actors. Messages persist before delivery.

---

## API overview

### Actor trait

```rust
pub trait Actor: Any + Send + Sync {
    /// Required — handle an incoming message
    fn receive(&mut self, mail: Mail) -> Option<Mail>;

    /// Optional — called after actor is created or restored
    fn post_start(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Msg::from_text("Start up signal received").into())
    }

    /// Optional — called before shutdown or eviction
    fn pre_shutdown(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Msg::from_text("Shutdown signal received").into())
    }
}
```

### Producer trait

```rust
#[typetag::serde]
pub trait Producer {
    /// Create a new actor instance
    fn produce(&mut self) -> Box<dyn Actor>;
}
```

### Message model

| Type | Description |
|------|-------------|
| `Msg` | A single message with text or binary content, from/to addresses, and a unique ID |
| `Mail::Trade(Msg)` | A single-message envelope delivered to the actor |
| `Mail::Bulk(Vec<Msg>)` | A batched envelope for multiple messages |
| `Mail::Blank` | An empty envelope (no-op) |

### Address model

```rust
let addr = Addr::new("my_actor");                         // local
let remote = Addr::remote("my_actor", "10.0.0.1:7171");   // remote
```

Addresses embed the node's IP and port, making remote routing transparent.

---

## Example

```rust
use arrows::{Actor, Mail, Msg, Producer, define_actor, send};
use serde::{Deserialize, Serialize};

struct MyActor;

impl Actor for MyActor {
    fn receive(&mut self, incoming: Mail) -> Option<Mail> {
        match incoming {
            Mail::Trade(msg) => println!("Got: {:?}", msg.as_text()),
            Mail::Bulk(msgs) => {
                for msg in &msgs {
                    println!("Got: {:?}", msg.as_text());
                }
            }
            Mail::Blank => {}
        }
        Some(Msg::from_text("Acknowledged").into())
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct MyProducer;

#[typetag::serde]
impl Producer for MyProducer {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(MyActor)
    }
}

define_actor!("my_actor", MyProducer::default());

let m1 = Msg::from_text("Hello!");
let m2 = Msg::from_text("Hello again!");
send!("my_actor", (m1, m2));
```

---

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `LISTEN_ADDR` | auto-detect | `IP:PORT` to bind the listener |
| `PORT` | `7171` | Listener port |
| `DB_PATH` | `/tmp` | SQLite database directory |
| `db_buff_size` | `1` | Buffer size before flush |

```bash
cargo run --bin arrows -- -i user --addr 127.0.0.1:8181 -d /tmp/mydb
```

---

## License

AGPL-3.0-or-later.
