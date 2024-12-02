#[cfg(feature = "nt-virtualalloc")]
use mod_nt_virtualalloc::NtVirtualAlloc;

/// Set the global allocator to the custom NT VirtualAlloc allocator
#[cfg(feature = "nt-virtualalloc")]
#[global_allocator]
static GLOBAL: NtVirtualAlloc = NtVirtualAlloc;

// #[cfg(feature = "nt-heapalloc")]
// use mod_nt_heapalloc::NT_HEAPALLOCATOR;

// Initialize global heap allocator
// #[cfg(feature = "nt-heapalloc")]
// NT_HEAPALLOCATOR.initialize();
