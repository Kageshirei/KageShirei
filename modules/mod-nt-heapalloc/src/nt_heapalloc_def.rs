use kageshirei_win32::ntdef::HANDLE;

pub const NTDLL_HASH: u32 = 0x1edab0ed;

pub const RTL_CREATE_HEAP_H: usize = 0xe1af6849;
pub const RTL_ALLOCATE_HEAP_H: usize = 0x3be94c5a;
pub const RTL_FREE_HEAP_H: usize = 0x73a9e4d7;
pub const RTL_DESTROY_HEAP_H: usize = 0xceb5349f;
pub const RTL_REALLOCATE_HEAP_H: usize = 0xaf740371;

/// Type definition for the RtlCreateHeap function.
///
/// Creates a heap with the specified attributes. The heap can be used to allocate and manage memory
/// dynamically.
///
/// # Parameters
/// - `[in]` - `Flags`: Specifies the attributes of the heap. This can include options such as
///   enabling heap serialization.
/// - `[in, opt]` - `HeapBase`: A pointer to a memory block that will serve as the base of the heap.
///   This parameter can be `NULL`, in which case the system determines the base address.
/// - `[in]` - `ReserveSize`: The initial size, in bytes, to reserve for the heap. This is the
///   amount of virtual memory reserved for the heap.
/// - `[in]` - `CommitSize`: The initial size, in bytes, of committed memory in the heap. This is
///   the amount of physical memory initially allocated for the heap.
/// - `[in, opt]` - `Lock`: A pointer to a lock for heap synchronization. This can be `NULL` if no
///   lock is required.
/// - `[in, opt]` - `Parameters`: A pointer to an optional structure that specifies advanced
///   parameters for heap creation. This can be `NULL`.
///
/// # Returns
/// - `HANDLE`: A handle to the newly created heap. If the heap creation fails, the handle will be
///   `NULL`.
pub type RtlCreateHeap = unsafe extern "system" fn(
    Flags: u32,
    HeapBase: *mut u8,
    ReserveSize: usize,
    CommitSize: usize,
    Lock: *mut u8,
    Parameters: *mut u8,
) -> HANDLE;

/// Type definition for the RtlAllocateHeap function.
///
/// Allocates a block of memory from the specified heap. The allocated memory is uninitialized.
///
/// # Parameters
/// - `[in]` - `hHeap`: A handle to the heap from which the memory will be allocated.
/// - `[in]` - `dwFlags`: Flags that control aspects of the allocation, such as whether to generate
///   exceptions on failure.
/// - `[in]` - `dwBytes`: The number of bytes to allocate from the heap.
///
/// # Returns
/// - `*mut u8`: A pointer to the allocated memory block. If the allocation fails, the pointer will
///   be `NULL`.
pub type RtlAllocateHeap = unsafe extern "system" fn(hHeap: HANDLE, dwFlags: u32, dwBytes: usize) -> *mut u8;

/// Type definition for the RtlFreeHeap function.
///
/// Frees a memory block allocated from the specified heap. The freed memory is returned to the heap
/// and can be reused.
///
/// # Parameters
/// - `[in]` - `hHeap`: A handle to the heap from which the memory was allocated.
/// - `[in]` - `dwFlags`: Flags that control aspects of the free operation, such as whether to
///   perform validation checks.
/// - `[in]` - `lpMem`: A pointer to the memory block to be freed.
///
/// # Returns
/// - `BOOL`: A boolean value indicating whether the operation was successful (`TRUE`) or not
///   (`FALSE`).
pub type RtlFreeHeap = unsafe extern "system" fn(hHeap: HANDLE, dwFlags: u32, lpMem: *mut u8) -> i32;

/// Type definition for the RtlReAllocateHeap function.
///
/// Reallocates a memory block from the specified heap, changing its size. The contents of the
/// memory block are preserved up to the smaller of the new or old sizes.
///
/// # Parameters
/// - `[in]` - `hHeap`: A handle to the heap from which the memory will be reallocated.
/// - `[in]` - `dwFlags`: Flags that control aspects of the reallocation, such as whether to
///   generate exceptions on failure.
/// - `[in]` - `lpMem`: A pointer to the memory block to be reallocated.
/// - `[in]` - `dwBytes`: The new size, in bytes, for the memory block.
///
/// # Returns
/// - `*mut u8`: A pointer to the reallocated memory block. If the reallocation fails, the pointer
///   will be `NULL`.
pub type RtlReAllocateHeap =
    unsafe extern "system" fn(hHeap: HANDLE, dwFlags: u32, lpMem: *mut u8, dwBytes: usize) -> *mut u8;

/// Type definition for the RtlDestroyHeap function.
///
/// Destroys the specified heap and releases all of its memory. Once a heap is destroyed, it cannot
/// be used.
///
/// # Parameters
/// - `[in]` - `hHeap`: A handle to the heap to be destroyed.
///
/// # Returns
/// - `HANDLE`: The function returns `NULL` if the heap was successfully destroyed. If the function
///   fails, it returns the handle to the heap.
pub type RtlDestroyHeap = unsafe extern "system" fn(hHeap: HANDLE) -> HANDLE;
