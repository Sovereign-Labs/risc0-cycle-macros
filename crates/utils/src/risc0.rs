pub use risc0_zkvm;
pub use risc0_zkvm_platform::syscall::sys_cycle_count;
pub use risc0_zkvm_platform::syscall::SyscallName;

// Safety: string is null terminated
pub const SYSCALL_NAME_METRICS: SyscallName =
    unsafe { SyscallName::from_bytes_with_nul("cycle_metrics\0".as_bytes().as_ptr()) };

pub fn get_syscall_name() -> SyscallName {
    SYSCALL_NAME_METRICS
}
