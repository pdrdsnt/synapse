
<small><i>This README contains ChatGPT-generated content.</i></small>
# ReceiverFunnel

A small Rust utility that merges multiple async `Stream`s into a single receiving channel (a "funnel"). It is useful when you want to collect events coming from many independent sources and process them from one single `UnboundedReceiver`.

This crate/file demonstrates a minimal implementation using `tokio`, `tokio-stream` and `futures`.

---

## Key features

* Start a background task that listens to control messages (add / remove subscriptions) and to all active streams.
* Add subscriptions dynamically; each subscription is registered under a generated `u32` id.
* Remove subscriptions by id.
* Single output `UnboundedReceiver<T>` that receives items produced by any active stream.

---

## Main types

* `ReceiverFunnel<T, S>` – the funnel handle. Generic over the item type `T` and the stream type `S`.

  * `start() -> (Self, UnboundedReceiver<T>)` — spawns the background task and returns the funnel handle plus a receiver to read items.
  * `add_subscription(&self, receiver: S) -> Option<u32>` — adds a stream and returns its id.
  * `remove_subscription(&self, id: u32)` — removes a stream by id.

* `Control<T, S>` — internal enum used to tell the background task to add or remove subscriptions.

---

## Usage example

```rust
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::main]
async fn main() {
    // Start the funnel; here we choose `i32` as the item type and wrap mpsc receivers
    let (funnel, mut out_rx) = ReceiverFunnel::<i32, UnboundedReceiverStream<i32>>::start();

    // Create a producer channel and a stream adapter
    let (tx1, rx1) = unbounded_channel::<i32>();
    let stream1 = UnboundedReceiverStream::new(rx1);

    // Add the stream to the funnel
    let id1 = funnel.add_subscription(stream1).expect("failed to add subscription");

    // Send one value from producer side
    tx1.send(123).expect("send failed");

    // Receive from the funnel's single receiver
    if let Some(item) = out_rx.recv().await {
        println!("received from funnel: {}", item);
    }

    // Remove subscription when no longer needed
    funnel.remove_subscription(id1);
}
```

Note: the example above uses `tokio_stream::wrappers::UnboundedReceiverStream` to convert a `tokio::sync::mpsc::UnboundedReceiver<T>` into a `Stream<Item = T>` that matches the funnel's generic `S`.

---

## Cargo.toml (dependencies)

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
Futures = "0.3"
tokio-stream = "0.1"
```

(Replace `Futures` with `futures` if you prefer the lowercase crate name; the implementation requires the `Stream` trait from `futures`.)

---

## Implementation notes & caveats

* `ReceiverFunnel::start` spawns a background `tokio::spawn` task. Make sure you run inside a Tokio runtime.
* The background task uses `tokio_stream::StreamMap<u32, S>` so each subscription must be `Unpin + Send + 'static`.
* `used: RefCell<HashSet<u32>>` tracks which ids are in use on the handle side. The internal background task also tracks `active_ids`.
* `subscriptions` (an `Arc<Mutex<Vec<Subscription>>>`) is used internally to keep a small registry — it can be extended to store metadata per subscription.
* The current implementation generates ids by scanning from `0` upward and checking the local `used` set. This is simple but could be replaced with a more efficient id allocator if you expect many add/removes.
* When `remove_subscription` is called the id is removed from both the handle's `used` set and the background `StreamMap`.

---

## License

MIT — feel free to copy and adapt.

---
