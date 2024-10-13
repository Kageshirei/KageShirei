use alloc::sync::Arc;
use core::{
    fmt,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::Future;
use mod_nostd::nostd_mpsc;
use rs2_runtime::Runtime;
use spin::Mutex;

use crate::nostd_nt_threadpool::NoStdThreadPool;

/// The `NoStdNtRuntime` struct provides a custom implementation of the `Runtime` trait,
/// using a thread pool to manage the execution of jobs. This runtime is designed to work
/// in a `no_std` environment, relying on custom synchronization primitives and a custom MPSC channel.
#[derive(Clone)]
pub struct NoStdNtRuntime {
    pool: Arc<Mutex<NoStdThreadPool>>, /* The thread pool is protected by a Mutex for safe concurrent access and is
                                        * wrapped in an Arc for shared ownership. */
}

impl fmt::Debug for NoStdNtRuntime {
    /// Custom implementation of the `Debug` trait for `NoStdNtRuntime`.
    /// It omits details of the thread pool for simplicity, providing a basic structure output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoStdNtRuntime")
            .field("pool", &"ThreadPool { ... }") // Omitting the internal state of the ThreadPool
            .finish()
    }
}

impl NoStdNtRuntime {
    /// Creates a new `NoStdNtRuntime` with a specified number of worker threads in the thread pool.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of worker threads to be spawned in the thread pool.
    ///
    /// # Returns
    ///
    /// * A new `NoStdNtRuntime` instance that wraps the thread pool.
    pub fn new(size: usize) -> Self {
        NoStdNtRuntime {
            pool: Arc::new(Mutex::new(NoStdThreadPool::new(size))),
        }
    }

    /// Shuts down the thread pool, ensuring all worker threads complete their tasks before termination.
    ///
    /// This method locks the Mutex around the thread pool to ensure safe access during the shutdown process.
    pub fn shutdown(&self) {
        let mut pool = self.pool.lock(); // Acquire a lock on the thread pool to safely shut it down.
        pool.shutdown();
    }
}

impl Runtime for NoStdNtRuntime {
    /// Spawns a job to be executed by the thread pool.
    ///
    /// The job is a closure that is executed by one of the worker threads in the pool.
    ///
    /// # Arguments
    ///
    /// * `job` - A closure that implements `FnOnce` and `Send`, representing the task to be executed.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let pool = self.pool.lock(); // Acquire a lock on the thread pool.
        pool.execute(job); // Send the job to be executed by the thread pool.
    }

    /// Blocks the current thread until the provided future is resolved.
    ///
    /// This method repeatedly polls the future until it is ready, using a custom waker
    /// to signal progress. It allows synchronous code to await the result of an asynchronous task.
    ///
    /// # Arguments
    ///
    /// * `future` - The future to be polled until completion.
    ///
    /// # Returns
    ///
    /// * The output of the future once it is completed.
    fn block_on<F>(&self, mut future: F) -> F::Output
    where
        F: Future + Send + 'static,
    {
        // Create a custom MPSC channel to signal when the future is ready
        let (tx, rx) = nostd_mpsc::channel();

        // Pin the future to the stack, ensuring it cannot be moved in memory
        let mut future = unsafe { Pin::new_unchecked(&mut future) };

        // Create a custom waker that sends a signal on the channel when the future is ready
        let waker = Waker::from(Arc::new(SimpleWaker {
            tx: Mutex::new(Some(tx)),
        }));
        let mut context = Context::from_waker(&waker);

        loop {
            // Poll the future to see if it is ready or still pending
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output, // The future is ready; return its output.
                Poll::Pending => {
                    // Wait for the signal that the future has made progress
                    let _ = rx.recv();
                },
            }
        }
    }
}

/// A simple custom waker that uses an MPSC channel to signal when a future is ready.
///
/// This struct is used within the `block_on` method to notify the runtime that the future
/// has made progress and should be polled again.
struct SimpleWaker {
    tx: Mutex<Option<nostd_mpsc::Sender<()>>>, /* The sender half of the MPSC channel, wrapped in a Mutex for safe
                                                * access. */
}

impl alloc::task::Wake for SimpleWaker {
    /// Sends a signal on the channel to indicate that the future has made progress and should be polled.
    ///
    /// This method is called when the waker is woken up, typically because the future is ready to make progress.
    fn wake(self: Arc<Self>) {
        if let Some(tx) = self.tx.lock().take() {
            let _ = tx.send(());
        }
    }
}
