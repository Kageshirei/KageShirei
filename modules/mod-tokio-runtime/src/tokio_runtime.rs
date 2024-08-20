use rs2_runtime::Runtime;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Runtime as TokioRuntime;

/// The `TokioAdapter` struct wraps a Tokio runtime to implement the `Runtime` trait.
pub struct TokioAdapter {
    runtime: Arc<TokioRuntime>,
}

impl TokioAdapter {
    /// Creates a new `TokioAdapter` with the provided Tokio runtime.
    ///
    /// # Arguments
    ///
    /// * `runtime` - An `Arc` of the Tokio runtime.
    ///
    /// # Returns
    ///
    /// * A `TokioAdapter` instance wrapping the Tokio runtime.
    pub fn new(runtime: Arc<TokioRuntime>) -> Self {
        TokioAdapter { runtime }
    }
}

impl Runtime for TokioAdapter {
    /// Spawns a job on the Tokio runtime.
    ///
    /// The job is executed asynchronously using Tokio's task scheduler.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.runtime.spawn(async { job() });
    }

    /// Blocks on a future using the Tokio runtime's `block_on` method.
    ///
    /// This allows for executing asynchronous operations in a synchronous context.
    fn block_on<F: Future>(&self, f: F) -> F::Output
    where
        F: Send + 'static,
    {
        self.runtime.block_on(f)
    }
}
