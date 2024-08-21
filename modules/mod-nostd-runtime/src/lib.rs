#![no_std]
extern crate alloc;

pub mod nostd_runtime;
pub mod nostd_threadpool; // Nota: alla fine rimuoveremo questo modulo

pub use nostd_runtime::NoStdRuntime; // Nota: alla fine rimuoveremo questo modulo
