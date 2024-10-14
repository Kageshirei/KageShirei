use std::{future::Future, sync::Arc};

use kageshirei_runtime::Runtime;
use tokio::runtime::{Builder, Handle, Runtime as TokioRuntime};

/// A wrapper around Tokio's runtime to implement the `Runtime` trait.
pub struct TokioRuntimeWrapper {
    runtime: Arc<TokioRuntime>,
}

impl TokioRuntimeWrapper {
    /// Creates a new `TokioRuntimeWrapper` with a specified number of worker threads.
    ///
    /// # Arguments
    ///
    /// * `num_threads` - The number of worker threads to be used by the runtime.
    ///
    /// # Returns
    ///
    /// * A new instance of `TokioRuntimeWrapper`.
    pub fn new(num_threads: usize) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(num_threads)
            .enable_all()
            .build()
            .unwrap();
        TokioRuntimeWrapper {
            runtime: Arc::new(runtime),
        }
    }

    pub fn handle(&self) -> Handle { self.runtime.handle().clone() }
}

impl Runtime for TokioRuntimeWrapper {
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.runtime.spawn(async move { job() });
    }

    fn block_on<F: Future>(&self, f: F) -> F::Output
    where
        F: Send + 'static,
    {
        self.runtime.block_on(f)
    }
}

impl Clone for TokioRuntimeWrapper {
    fn clone(&self) -> Self {
        TokioRuntimeWrapper {
            runtime: Arc::clone(&self.runtime),
        }
    }
}
