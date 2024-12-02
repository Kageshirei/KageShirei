use core::future::Future;

/// The `Runtime` trait defines the basic operations required for a runtime environment.
///
/// A runtime is responsible for managing the execution of tasks and futures.
/// This trait is intended to be implemented by different runtime backends, such as Tokio or a
/// custom thread pool.
pub trait Runtime: Send + Sync + 'static {
    /// Spawns a new task on the runtime.
    ///
    /// # Arguments
    ///
    /// * `job` - A closure that will be executed as a task.
    ///
    /// The closure must be `FnOnce`, `Send`, and `'static` to ensure it can be safely
    /// executed within the runtime.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;

    /// Blocks on a future and returns its output.
    ///
    /// # Arguments
    ///
    /// * `f` - The future to block on.
    ///
    /// This function is used to block the current thread until the future is resolved.
    fn block_on<F>(&self, f: F) -> F::Output
    where
        F: Future + Send + 'static;
}
