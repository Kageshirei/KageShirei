/// Macro to define a syscall structure and its associated implementations.
///
/// This macro generates a struct with the given name and a specified hash value.
/// It also implements the `NtSyscall` trait, `Send`, `Sync`, and `Default` traits for the generated
/// struct.
///
/// # Arguments
///
/// * `$name` - The identifier for the syscall struct.
/// * `$hash` - The hash value associated with the syscall.
///
/// # Generated Struct
///
/// The generated struct will have the following fields:
/// * `number` - A `u16` representing the syscall number.
/// * `address` - A mutable pointer to `u8` representing the address of the syscall.
/// * `hash` - A `usize` representing the hash value of the syscall.
///
/// # Example
///
/// ```rust ignore
/// define_syscall!(MySyscall, 0x12345678);
///
/// let syscall = MySyscall::new();
/// assert_eq!(syscall.hash(), 0x12345678);
/// ```
#[macro_export]
macro_rules! define_indirect_syscall {
    ($name:ident, $hash:expr) => {
        pub struct $name {
            pub number:  u16,
            pub address: *mut u8,
            pub hash:    usize,
        }

        impl NtSyscall for $name {
            fn new() -> Self {
                Self {
                    number:  0,
                    address: core::ptr::null_mut(),
                    hash:    $hash,
                }
            }

            fn number(&self) -> u16 { self.number }

            fn address(&self) -> *mut u8 { self.address }

            fn hash(&self) -> usize { self.hash }
        }

        // Safety: This is safe because the struct $name does not contain any non-thread-safe data.
        unsafe impl Send for $name {}
        // Safety: This is safe because the struct $name does not contain any non-thread-safe data.
        unsafe impl Sync for $name {}

        impl Default for $name {
            fn default() -> Self { Self::new() }
        }
    };
}

#[macro_export]
macro_rules! define_direct_syscall {
    ($name:ident, $hash:expr, $f:ty) => {
        /// Struct representing a direct syscall with a specific hash and function pointer.
        ///
        /// This struct holds the hash of the syscall and a function pointer
        /// to the corresponding implementation.
        ///
        /// # Fields
        /// - `f`: A pointer to the function implementing the syscall.
        /// - `hash`: The hash value of the syscall.
        pub struct $name {
            pub f:    Option<$f>,
            pub hash: usize,
        }

        impl $name {
            /// Creates a new instance of the direct syscall struct.
            ///
            /// The function pointer is initialized with the provided function or `None`.
            fn new() -> Self {
                Self {
                    f:    None,
                    hash: $hash,
                }
            }
        }

        // Safety: This is safe because the struct $name does not contain any non-thread-safe data.
        unsafe impl Send for $name {}
        // Safety: This is safe because the struct $name does not contain any non-thread-safe data.
        unsafe impl Sync for $name {}

        impl Default for $name {
            fn default() -> Self { Self::new() }
        }
    };
}
