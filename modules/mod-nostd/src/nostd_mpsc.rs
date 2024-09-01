use alloc::{sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

/// A simple implementation of a multiple-producer, single-consumer (MPSC) channel
/// with a fixed-size buffer, designed for use in a `no_std` environment.
///
/// The `Channel` struct encapsulates the shared state between the sender and receiver,
/// including the buffer, capacity, and atomic flags for data availability and space availability.
#[derive(Debug)]
pub struct NoStdChannel<T> {
    buffer: Mutex<Vec<T>>, // A mutex-protected vector that serves as the buffer for the channel.
    capacity: usize,       // The maximum number of items the buffer can hold.
    available: AtomicBool, // Indicates if there is data available for the receiver.
    space_available: AtomicBool, // Indicates if there is space available for the sender.
}

/// The `Sender` struct represents the sending side of the channel. It allows
/// multiple producers to send messages to a single consumer.
#[derive(Debug)]
pub struct Sender<T> {
    channel: Arc<NoStdChannel<T>>, // An atomic reference-counted pointer to the shared `Channel`.
}

/// The `Receiver` struct represents the receiving side of the channel. It allows
/// a single consumer to receive messages from multiple producers.
pub struct Receiver<T> {
    channel: Arc<NoStdChannel<T>>, // An atomic reference-counted pointer to the shared `Channel`.
}

/// Creates a new MPSC channel with a fixed-size buffer and returns a `Sender` and `Receiver` pair.
///
/// # Returns
/// * `(Sender<T>, Receiver<T>)` - A pair of `Sender` and `Receiver` structs that represent the ends of the channel.
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(NoStdChannel {
        buffer: Mutex::new(Vec::new()),
        capacity: 32, // Fixed size for the buffer; can be adjusted as needed.
        available: AtomicBool::new(false),
        space_available: AtomicBool::new(true),
    });

    (
        Sender {
            channel: Arc::clone(&channel),
        },
        Receiver { channel },
    )
}

impl<T> Clone for Sender<T> {
    /// Clones the `Sender`, allowing multiple producers to send messages to the same channel.
    ///
    /// # Returns
    /// * `Sender<T>` - A new `Sender` instance that shares access to the same channel.
    fn clone(&self) -> Self {
        Sender {
            channel: Arc::clone(&self.channel),
        }
    }
}

impl<T> Clone for Receiver<T> {
    /// Clones the `Receiver`, allowing multiple consumers to receive messages from the same channel.
    /// However, in typical usage, only one `Receiver` should be active at a time to prevent data races.
    ///
    /// # Returns
    /// * `Receiver<T>` - A new `Receiver` instance that shares access to the same channel.
    fn clone(&self) -> Self {
        Receiver {
            channel: Arc::clone(&self.channel),
        }
    }
}

impl<T> Sender<T> {
    /// Sends a value into the channel. If the buffer is full, the sender will block until space is available.
    ///
    /// # Arguments
    /// * `value` - The value to be sent into the channel.
    ///
    /// # Returns
    /// * `Result<(), ()>` - Returns `Ok(())` if the value was successfully sent, or `Err(())` if the buffer is full.
    pub fn send(&self, value: T) -> Result<(), ()> {
        loop {
            {
                let mut buffer = self.channel.buffer.lock();
                if buffer.len() < self.channel.capacity {
                    buffer.push(value);
                    self.channel.available.store(true, Ordering::Release); // Notify that there is an item available
                    return Ok(());
                }
            }
            // If the buffer is full, wait until space becomes available
            while !self.channel.space_available.load(Ordering::Acquire) {}
        }
    }
}

impl<T> Receiver<T> {
    /// Receives a value from the channel. If the buffer is empty, the receiver will block until data is available.
    ///
    /// # Returns
    /// * `Option<T>` - Returns `Some(T)` with the received value, or `None` if the channel is closed and empty.
    pub fn recv(&self) -> Option<T> {
        loop {
            {
                let mut buffer = self.channel.buffer.lock();
                if let Some(item) = buffer.pop() {
                    self.channel.space_available.store(true, Ordering::Release); // Notify that there is space available
                    return Some(item);
                } else if Arc::strong_count(&self.channel) == 1 {
                    // If the buffer is empty and all senders have been dropped, terminate
                    return None;
                }
            }
            // If the buffer is empty, wait until data becomes available
            while !self.channel.available.load(Ordering::Acquire) {
                // Check if all senders have been dropped while waiting
                if Arc::strong_count(&self.channel) == 1 {
                    return None;
                }
            }
        }
    }
}

/// Implements the `Iterator` trait for the `Receiver` struct, allowing it to be used in for loops
/// and other iterator-based constructs.
///
/// # Returns
/// * `Option<T>` - The next item in the channel, or `None` if the channel is empty and closed.
impl<T> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.recv()
    }
}
