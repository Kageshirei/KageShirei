use crate::threadpool::ThreadPool;
use rs2_runtime::Runtime;
use std::future::Future;

/// The `CustomRuntime` struct wraps a custom thread pool to implement the `Runtime` trait.
pub struct CustomRuntime {
    pool: ThreadPool,
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
            pool: ThreadPool::new(size),
        }
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
        self.pool.execute(job);
    }

    /// Blocks on a future.
    ///
    /// Since the custom thread pool is not designed for async operations, this method
    /// is currently unimplemented.
    fn block_on<F: Future>(&self, _f: F) -> F::Output
    where
        F: Send + 'static,
    {
        unimplemented!("Blocking on a future is not implemented in the custom thread pool");
    }
}
