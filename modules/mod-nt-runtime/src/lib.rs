#![no_std]
extern crate alloc;

pub mod channel;
pub mod nt_threadpool;

use channel::{channel, Receiver, Sender};
use nt_threadpool::ThreadPool;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll, Waker};
use rs2_runtime::Runtime;
use spin::Mutex;

// Garantisce che NoStdRuntime implementi Send e Sync
unsafe impl Send for NoStdRuntime {}
unsafe impl Sync for NoStdRuntime {}

pub struct NoStdRuntime {
    pool: Arc<Mutex<ThreadPool>>,   // The thread pool
    shutdown_flag: Arc<AtomicBool>, // Flag to signal shutdown
}

impl NoStdRuntime {
    /// Creates a new `NoStdRuntime` with the specified number of worker threads.
    pub fn new(size: usize) -> Self {
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let pool = ThreadPool::new(size, Arc::clone(&shutdown_flag));
        NoStdRuntime {
            pool: Arc::new(Mutex::new(pool)),
            shutdown_flag,
        }
    }

    /// Shuts down the thread pool, ensuring all workers have completed their jobs.
    pub fn shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
        let mut pool = self.pool.lock();
        pool.shutdown();
    }
}

impl Runtime for NoStdRuntime {
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let pool = self.pool.lock();
        pool.execute(Box::new(job));
    }

    fn block_on<F>(&self, mut future: F) -> F::Output
    where
        F: Future + Send + 'static,
    {
        let (tx, rx): (Sender<()>, Receiver<()>) = channel();

        let mut future = unsafe { Pin::new_unchecked(&mut future) };

        let waker = Waker::from(Arc::new(SimpleWaker {
            tx: Mutex::new(Some(tx)),
        }));
        let mut context = Context::from_waker(&waker);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => {
                    let _ = rx.recv();
                }
            }
        }
    }
}

struct SimpleWaker {
    tx: Mutex<Option<Sender<()>>>,
}

impl Wake for SimpleWaker {
    fn wake(self: Arc<Self>) {
        if let Some(tx) = self.tx.lock().take() {
            let _ = tx.send(());
        }
    }
}
