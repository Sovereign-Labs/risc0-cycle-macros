use std::ffi::CStr;

use risc0_zkvm_platform::syscall::SyscallName;

pub const SYSCALL_NAME_METRICS: SyscallName = compute_const_syscall_name();

const fn compute_const_syscall_name() -> SyscallName {
    let c_str = if let Ok(name) = CStr::from_bytes_with_nul(b"cycle_metrics\0") {
        name
    } else {
        panic!("Failed to create syscall name")
    };

    if let Ok(syscall_name) = SyscallName::from_c_str(&c_str) {
        syscall_name
    } else {
        panic!("Failed to create syscall name")
    }
}

pub fn get_syscall_name() -> SyscallName {
    SYSCALL_NAME_METRICS
}

#[cfg(feature = "native")]
pub fn cycle_count_callback(input: risc0_zkvm::Bytes) -> risc0_zkvm::Result<risc0_zkvm::Bytes> {
    if input.len() == std::mem::size_of::<usize>() {
        let mut array = [0u8; std::mem::size_of::<usize>()];
        array.copy_from_slice(&input);
        println!("== syscall ==> {}", usize::from_le_bytes(array));
    } else {
        println!("NONE");
    }
    Ok(risc0_zkvm::Bytes::new())
}

pub fn get_syscall_name_cycles() -> SyscallName {
    risc0_zkvm_platform::syscall::nr::SYS_CYCLE_COUNT
}

// pub fn print_cycle_count() {
//     let metrics_syscall_name = get_syscall_name_cycles();
//     let serialized = risc0_zkvm_platform::syscall::sys_cycle_count().to_le_bytes();
//     risc0_zkvm::guest::env::send_recv_slice::<u8, u8>(metrics_syscall_name, &serialized);
// }
