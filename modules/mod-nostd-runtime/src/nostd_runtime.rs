extern crate alloc;

use crate::nostd_threadpool::ThreadPool;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use rs2_runtime::Runtime;

/// La struttura `NoStdRuntime` avvolge un pool di worker che eseguono task asincroni.
pub struct NoStdRuntime {
    pool: Arc<ThreadPool>, // Il pool di worker che gestisce la coda dei task.
}

impl NoStdRuntime {
    /// Crea una nuova `NoStdRuntime` con un numero specificato di worker.
    ///
    /// # Argomenti
    ///
    /// * `size` - Il numero di worker nel pool.
    ///
    /// # Ritorna
    ///
    /// * Un'istanza di `NoStdRuntime` che avvolge il pool di worker.
    pub fn new(size: usize) -> Self {
        let pool = Arc::new(ThreadPool::new(size));
        for _ in 0..size {
            let pool_clone = Arc::clone(&pool);
            NoStdRuntime::start_worker(pool_clone);
        }
        NoStdRuntime { pool }
    }

    /// Avvia un worker che esegue i task dalla coda.
    fn start_worker(pool: Arc<ThreadPool>) {
        // Simula l'esecuzione in parallelo tramite un loop cooperativo.
        pool.run_worker();
    }

    pub fn execute_all(&self) {
        // Simula worker cooperativi che eseguono i task uno ad uno
        while let Some(task) = self.pool.run_worker() {
            task(); // Esegui il task
        }
    }
}

impl Runtime for NoStdRuntime {
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.execute(job);
    }

    fn block_on<F>(&self, mut future: F) -> F::Output
    where
        F: Future + Send + 'static,
    {
        let mut future = unsafe { Pin::new_unchecked(&mut future) };

        let waker = create_waker();
        let mut context = Context::from_waker(&waker);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => {
                    self.pool.run_worker(); // Esegui un task dalla coda, se presente
                }
            }
        }
    }
}

/// Crea un Waker custom utilizzando una RawWakerVTable minima.
fn create_waker() -> Waker {
    fn noop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(core::ptr::null(), &VTABLE)
    }

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) }
}
