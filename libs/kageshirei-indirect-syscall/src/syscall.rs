/// A crate for performing indirect syscalls on x86 and x86_64 architectures.
/// It provides macros and functions to execute syscalls with dynamic syscall numbers and addresses.

use core::arch::global_asm;

/// Macro to run a syscall with a given syscall number (`ssn`) and address (`addr`).
///
/// This macro is designed for the x86_64 architecture and ensures the correct
/// number of arguments are passed to the syscall function.
///
/// # Parameters
///
/// - `$ssn`: The syscall number (u16).
/// - `$addr`: The address of the syscall (usize).
/// - `$y`: The arguments for the syscall. This can be one or more expressions.
///
/// # Safety
///
/// This macro calls an unsafe function, so the caller must ensure that the arguments
/// and syscall address are valid and that the syscall is safe to execute.
#[cfg(target_arch = "x86_64")]
#[macro_export]
macro_rules! run {
    ($ssn:expr, $addr:expr, $($y:expr), +) => {
        {
            // Initialize the argument count to zero
            let mut cnt: u32 = 0;

            // Iterate over each argument to count the number of arguments
            $(
                let _ = $y;
                cnt += 1;
            )+

            // Call the unsafe syscall function with the syscall number, address (offset by 0x12),
            // the argument count, and the actual arguments
            unsafe { $crate::syscall::do_syscall($ssn, $addr + 0x12, cnt, $($y), +) }
        }
    }
}

#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! run_syscall {
    ($ssn:expr, $addr:expr, $($y:expr), +) => {
        {
        let mut cnt:u32 = 0;
        $(
            let _ = $y;
            cnt += 1;
        )+
        unsafe { $crate::syscall::do_syscall($ssn, $addr + 0x0F, cnt, $($y), +) }
    }}
}

#[cfg(target_arch = "x86_64")]
global_asm!(
    "
.global do_syscall

.section .text

do_syscall:
    mov [rsp - 0x8],  rsi
    mov [rsp - 0x10], rdi
    mov [rsp - 0x18], r12

    xor r10, r10			
    mov rax, rcx			
    mov r10, rax

    mov eax, ecx

    mov r12, rdx
    mov rcx, r8

    mov r10, r9
    mov rdx,  [rsp + 0x28]
    mov r8,   [rsp + 0x30]
    mov r9,   [rsp + 0x38]

    sub rcx, 0x4
    jle skip

    lea rsi,  [rsp + 0x40]
    lea rdi,  [rsp + 0x28]

    rep movsq
skip:
    mov rcx, r12

    mov rsi, [rsp - 0x8]
    mov rdi, [rsp - 0x10]
    mov r12, [rsp - 0x18]

    jmp rcx
"
);

#[cfg(target_arch = "x86")]
global_asm!(
    "
.global _do_syscall

.section .text

_do_syscall:
    mov ecx, [esp + 0x0C]
    not ecx
    add ecx, 1
    lea edx, [esp + ecx * 4]

    mov ecx, [esp]
    mov [edx], ecx

    mov [edx - 0x04], esi
    mov [edx - 0x08], edi

    mov eax, [esp + 0x04]
    mov ecx, [esp + 0x0C]

    lea esi, [esp + 0x10]
    lea edi, [edx + 0x04]

    rep movsd

    mov esi, [edx - 0x04]
    mov edi, [edx - 0x08]
    mov ecx, [esp + 0x08]
    
    mov esp, edx

    mov edx, fs:[0xC0]
    test edx, edx
    je native

    mov edx, fs:[0xC0]
    jmp ecx

native:
    mov edx, ecx
    sub edx, 0x05
    push edx
    mov edx, esp
    jmp ecx
    ret

is_wow64:
"
);

extern "C" {
    /// Executes a syscall with the given syscall number, address, and arguments.
    ///
    /// ### Parameters
    ///
    /// - `ssn`: The syscall number.
    /// - `addr`: The address of the syscall.
    /// - `n_args`: The number of arguments for the syscall.
    /// - `...`: The arguments for the syscall.
    ///
    /// ### Returns
    ///
    /// The result of the syscall as an `i32`.
    pub fn do_syscall(ssn: u16, addr: usize, n_args: u32, ...) -> i32;
}
