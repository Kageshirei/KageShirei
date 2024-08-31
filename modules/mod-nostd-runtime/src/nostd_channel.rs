use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

pub struct Channel<T> {
    buffer: Mutex<Vec<T>>,
    capacity: usize,
}

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel {
        buffer: Mutex::new(Vec::new()),
        capacity: 32, // Dimensione fissa, può essere modificata
    });

    (
        Sender {
            channel: Arc::clone(&channel),
        },
        Receiver { channel },
    )
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            channel: Arc::clone(&self.channel),
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), ()> {
        let mut buffer = self.channel.buffer.lock();
        if buffer.len() < self.channel.capacity {
            buffer.push(value);
            Ok(())
        } else {
            Err(()) // Buffer pieno
        }
    }
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        let mut buffer = self.channel.buffer.lock();
        if !buffer.is_empty() {
            Some(buffer.remove(0))
        } else {
            None // Buffer vuoto
        }
    }
}

// Implement the Iterator trait for the Receiver struct
impl<T> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.recv()
    }
}
