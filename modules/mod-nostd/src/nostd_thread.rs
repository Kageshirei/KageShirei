use alloc::boxed::Box;
use core::{ffi::c_void, mem::size_of, ptr::null_mut};

use kageshirei_win32::{
    ntapi::nt_current_process,
    ntdef::{ObjectAttributes, HANDLE, OBJ_CASE_INSENSITIVE, THREAD_ALL_ACCESS},
    ntstatus::NT_SUCCESS,
};
use mod_agentcore::instance;
use mod_win32::utils::NT_STATUS;

/// A Rust abstraction over the Windows native threading API, specifically using
/// functions from `ntdll.dll` to create and manage threads without using the standard library.
/// This is particularly useful in `no_std` environments where you don't have access to the
/// standard Rust threading facilities.
///
/// This struct holds a handle to the thread created via `NtCreateThreadEx` and provides methods
/// to spawn a new thread and wait for its completion.
///
/// # Safety
/// This implementation involves unsafe code as it interacts directly with low-level Windows APIs
/// and performs raw pointer manipulations. Use this with care.
#[derive(Debug, Clone, Copy)]
pub struct NoStdThread {
    pub thread_handle: HANDLE,
}

unsafe impl Send for NoStdThread {}
unsafe impl Sync for NoStdThread {}

impl NoStdThread {
    /// Spawns a new thread in the current process using the `NtCreateThreadEx` function from
    /// `ntdll.dll`. The thread will execute the closure provided by the caller.
    ///
    /// # Parameters
    /// - `start_routine`: A closure that will be executed by the new thread. The closure must implement `FnOnce() +
    ///   Send + 'static` because it is executed once and must be safely transferable across threads.
    ///
    /// # Returns
    /// - `NoStdThread`: An instance of `NoStdThread` containing the handle to the newly created thread.
    ///
    /// # NtCreateThreadEx
    /// This function is an undocumented Windows API that creates a new thread in the specified process.
    /// It allows more control over thread creation compared to `CreateThread` and is particularly useful
    /// in low-level system programming.
    ///
    /// # Safety
    /// This method is unsafe because it involves raw pointer operations and direct system calls that
    /// can lead to undefined behavior if misused.
    pub fn spawn<F>(start_routine: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        // Box the closure to move it onto the heap and convert it into a raw pointer.
        let boxed_closure = Box::new(start_routine);
        let closure_ptr = Box::into_raw(boxed_closure) as *mut F;

        // This is the entry point for the new thread, which will execute the provided closure.
        unsafe extern "system" fn thread_start_routine<F>(lp_parameter: *mut c_void) -> u32
        where
            F: FnOnce() + Send,
        {
            // Convert the raw pointer back into a Box and then call the closure.
            let closure: Box<F> = Box::from_raw(lp_parameter as *mut F);
            closure();
            0
        }

        // Get a handle to the current process, which is needed to create a thread within it.
        let proc_handle = nt_current_process();
        let mut thread_handle: HANDLE = null_mut();

        // Set up object attributes for the thread. These attributes include settings like
        // case insensitivity for object names.
        let mut obj_attr = ObjectAttributes::new();
        obj_attr.length = size_of::<ObjectAttributes>() as u32;
        obj_attr.attributes = OBJ_CASE_INSENSITIVE;

        // Call NtCreateThreadEx to create the thread and pass it the necessary parameters.
        let status = unsafe {
            instance().ntdll.nt_create_thread_ex.run(
                &mut thread_handle,
                THREAD_ALL_ACCESS,
                &mut obj_attr,
                proc_handle,
                thread_start_routine::<F> as *mut c_void,
                closure_ptr as *mut c_void,
                0,
                0,
                0,
                0,
                null_mut(),
            )
        };

        // Check the status of the thread creation and panic if the thread creation failed.
        if !NT_SUCCESS(status) {
            panic!("Failed to create thread: {}", NT_STATUS(status));
        }

        // Return the NoStdThread instance with the created thread's handle.
        NoStdThread {
            thread_handle,
        }
    }

    /// Waits for the thread to complete execution using the `NtWaitForSingleObject` function from
    /// `ntdll.dll`.
    ///
    /// # Returns
    /// - `Result<(), i32>`: `Ok(())` if the thread completed successfully, or an error code otherwise.
    ///
    /// # NtWaitForSingleObject
    /// This function waits until the specified object is in the signaled state or the time-out interval
    /// elapses. In this case, it waits for the thread to finish execution.
    ///
    /// # Safety
    /// This method is unsafe because it directly interacts with low-level Windows API, which can
    /// lead to undefined behavior if not used correctly.
    pub fn join(self) -> Result<(), i32> {
        let status = unsafe {
            instance()
                .ntdll
                .nt_wait_for_single_object
                .run(self.thread_handle, false, null_mut())
        };

        // Return Ok(()) if the wait was successful, or the error code otherwise.
        if status == 0 {
            Ok(())
        }
        else {
            Err(status)
        }
    }
}
