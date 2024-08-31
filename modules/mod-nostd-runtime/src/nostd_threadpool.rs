use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use mod_win32::nt_time::delay;
use rs2_communication_protocol::communication_structs::task_output::TaskOutput;
use rs2_communication_protocol::metadata::Metadata;
use spin::Mutex;

pub struct ThreadPool {
    tasks: Arc<Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>>, // La coda di task
    shutdown_flag: Arc<Mutex<bool>>,                            // Flag per segnalare il shutdown
}

impl ThreadPool {
    pub fn new(_size: usize) -> ThreadPool {
        ThreadPool {
            tasks: Arc::new(Mutex::new(Vec::new())),
            shutdown_flag: Arc::new(Mutex::new(false)), // Flag di shutdown inizialmente falso
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut tasks = self.tasks.lock();
        tasks.push(Box::new(f));
    }

    /// Esegue i task rimanenti fino a quando non sono tutti completati o viene richiesto il shutdown.
    // pub fn run_worker(&self) -> Option<Box<dyn FnOnce() + Send + 'static>> {
    //     loop {
    //         let task = self.tasks.lock().pop(); // Recupera un task dalla coda
    //         if task.is_some() {
    //             return task; // Ritorna il task da eseguire
    //         } else {
    //             let is_shutdown = *self.shutdown_flag.lock();
    //             if is_shutdown {
    //                 break; // Esce dal loop se è stato richiesto il shutdown e non ci sono task da eseguire
    //             }
    //         }
    //     }
    //     None
    // }

    pub fn run_worker(&self) -> Option<Box<dyn FnOnce() + Send + 'static>> {
        self.tasks.lock().pop()
    }

    pub fn has_tasks(&self) -> bool {
        !self.tasks.lock().is_empty()
    }

    /// Avvia il processo di shutdown della thread pool.
    /// Completa tutti i task rimanenti prima di terminare.
    pub fn shutdown(&self) {
        *self.shutdown_flag.lock() = true; // Imposta il flag di shutdown a `true`

        // Esegue tutti i task rimanenti nella coda.
        while let Some(task) = self.run_worker() {
            task(); // Esegui il task
        }
    }
}

pub fn task_type_a(metadata: Metadata) -> TaskOutput {
    delay(1);
    let mut output = TaskOutput::new();
    output.with_metadata(metadata);
    output.output = Some("Result from task type A".to_string());
    output
}

pub fn task_type_b(metadata: Metadata) -> TaskOutput {
    delay(3);
    let mut output = TaskOutput::new();
    output.with_metadata(metadata);
    output.output = Some("Result from task type B".to_string());
    output
}
