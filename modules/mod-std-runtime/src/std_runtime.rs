use crate::threadpool::ThreadPool;
use rs2_runtime::Runtime;
use std::{
    future::Future,
    pin::Pin,
    sync::{mpsc, Arc, Mutex},
    task::{Context, Poll, Waker},
};

/// The `CustomRuntime` struct wraps a custom thread pool to implement the `Runtime` trait.
#[derive(Debug, Clone)]
pub struct CustomRuntime {
    pool: Arc<Mutex<ThreadPool>>, // Wrap the ThreadPool in a Mutex inside an Arc.
}

impl CustomRuntime {
    /// Creates a new `CustomRuntime` with the specified number of worker threads.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of worker threads in the thread pool.
    ///
    /// # Returns
    ///
    /// * A `CustomRuntime` instance wrapping the custom thread pool.
    pub fn new(size: usize) -> Self {
        CustomRuntime {
            pool: Arc::new(Mutex::new(ThreadPool::new(size))),
        }
    }

    /// Shuts down the thread pool, ensuring all workers have completed their jobs.
    pub fn shutdown(&self) {
        let mut pool = self.pool.lock().unwrap(); // Lock the Mutex to get mutable access.
        pool.shutdown();
    }
}

impl Runtime for CustomRuntime {
    /// Spawns a job on the custom thread pool.
    ///
    /// The job is executed by one of the worker threads in the pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let pool = self.pool.lock().unwrap();
        pool.execute(job);
    }

    /// Blocks on a future until it completes, polling it in the current thread.
    fn block_on<F>(&self, mut future: F) -> F::Output
    where
        F: Future + Send + 'static,
    {
        // Create a simple channel to signal completion
        let (tx, rx) = mpsc::channel();

        // Wrap the future in a Pin
        let mut future = unsafe { Pin::new_unchecked(&mut future) };

        // Create a custom waker that sends a signal on the channel when woken up
        let waker = Waker::from(Arc::new(SimpleWaker {
            tx: Mutex::new(Some(tx)),
        }));
        let mut context = Context::from_waker(&waker);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => {
                    // Wait for the signal to be sent
                    let _ = rx.recv();
                }
            }
        }
    }
}

/// Struct for a simple custom Waker that sends a signal on a channel
struct SimpleWaker {
    tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl std::task::Wake for SimpleWaker {
    fn wake(self: Arc<Self>) {
        if let Some(tx) = self.tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
    }
}
