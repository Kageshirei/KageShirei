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
}

impl ThreadPool {
    pub fn new(_size: usize) -> ThreadPool {
        ThreadPool {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut tasks = self.tasks.lock();
        tasks.push(Box::new(f));
    }

    pub fn run_worker(&self) -> Option<Box<dyn FnOnce() + Send + 'static>> {
        self.tasks.lock().pop()
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
