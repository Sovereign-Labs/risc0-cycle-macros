#[cfg(feature = "risc0")]
pub use actual_impl::*;

#[cfg(feature = "risc0")]
mod actual_impl {
    use risc0_zkvm;
    use risc0_zkvm_platform::syscall::sys_cycle_count;
    use risc0_zkvm_platform::syscall::SyscallName;

    // Safety: string is null terminated
    pub const SYSCALL_NAME_METRICS: SyscallName =
        unsafe { SyscallName::from_bytes_with_nul("cycle_metrics\0".as_bytes().as_ptr()) };

    pub fn get_syscall_name() -> SyscallName {
        SYSCALL_NAME_METRICS
    }

    pub fn get_cycle_count() -> u64 {
        sys_cycle_count()
    }

    pub fn report_cycle_count(name: &str, count: u64) {
        // simple serialization to avoid pulling in bincode or other libs
        let mut serialized = Vec::new();
        serialized.extend(name.as_bytes());
        serialized.push(0);
        let size_bytes = count.to_le_bytes();
        serialized.extend(&size_bytes);

        // calculate the syscall name.
        let metrics_syscall_name = SYSCALL_NAME_METRICS;

        risc0_zkvm::guest::env::send_recv_slice::<u8, u8>(metrics_syscall_name, &serialized);
    }
}

#[cfg(not(feature = "risc0"))]
pub use facade::*;
#[cfg(not(feature = "risc0"))]
mod facade {
    pub fn get_cycle_count() -> u64 {
        0
    }

    pub fn report_cycle_count(_name: &str, _count: u64) {
        panic!("Reporting cycle count without risc0 feature enabled");
    }
}
