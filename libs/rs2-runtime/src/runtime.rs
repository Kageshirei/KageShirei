use std::future::Future;

/// The `Runtime` trait defines a generic interface for runtime environments
/// that can spawn tasks and block on futures.
///
/// This trait is designed to be implemented by different runtime backends,
/// such as a Tokio-based runtime or a custom thread pool.
pub trait Runtime {
    /// Spawns a job on the runtime.
    ///
    /// # Arguments
    ///
    /// * `job` - A closure representing the job to be executed.
    ///
    /// The job must be `Send` and `'static` to ensure it can be safely executed
    /// across threads and outlive its caller.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;

    /// Blocks on a future until it completes.
    ///
    /// # Arguments
    ///
    /// * `f` - The future to block on.
    ///
    /// This method is primarily used to execute asynchronous operations in a synchronous
    /// context. The future must be `Send` and `'static` for thread safety.
    fn block_on<F: Future>(&self, f: F) -> F::Output
    where
        F: Send + 'static;
}
