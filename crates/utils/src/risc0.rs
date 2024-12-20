#[cfg(feature = "risc0")]
pub use actual_impl::*;

#[cfg(feature = "risc0")]
mod actual_impl {
    use super::*;
    #[cfg(feature = "native")]
    use anyhow::Context;
    use risc0_zkvm;
    use risc0_zkvm_platform::syscall::sys_cycle_count;
    use risc0_zkvm_platform::syscall::SyscallName;
    use std::hint::black_box;

    #[cfg(feature = "native")]
    pub fn deserialize_metrics_call(serialized: &[u8]) -> anyhow::Result<(String, u64, u64)> {
        let null_pos = serialized
            .iter()
            .position(|&b| b == 0)
            .context("Could not find separator in provided bytes")?;

        let (name_bytes, metric_bytes_with_null) = serialized.split_at(null_pos);
        let name = std::str::from_utf8(name_bytes)
            .context("Invalid UTF-8 in name")?
            .to_owned();

        let cycles_bytes = &metric_bytes_with_null[1..9]; // Skip the null terminator
        let cycles = u64::from_le_bytes(cycles_bytes.try_into()?); // Convert bytes back into usize
                                                                   // Upper bound so we don't panice if more things
        let free_heap_bytes = &metric_bytes_with_null[9..17];
        let free_heap = u64::from_le_bytes(free_heap_bytes.try_into()?);
        Ok((name, cycles, free_heap))
    }

    fn serialize_metric(name: &str, cycle_count: u64, free_heap_bytes: u64) -> Vec<u8> {
        let name_bytes = name.as_bytes();
        // We know the exact capacity:
        // name_bytes plus one null terminator plus two u64s (16 bytes total)
        let mut serialized = Vec::with_capacity(name_bytes.len() + 1 + 16);
        serialized.extend_from_slice(name_bytes);
        serialized.push(0);
        serialized.extend_from_slice(&cycle_count.to_le_bytes());
        serialized.extend_from_slice(&free_heap_bytes.to_le_bytes());
        serialized
    }

    // Safety: string is null terminated
    pub const SYSCALL_NAME_METRICS: SyscallName =
        unsafe { SyscallName::from_bytes_with_nul("cycle_metrics\0".as_bytes().as_ptr()) };

    pub fn get_syscall_name() -> SyscallName {
        SYSCALL_NAME_METRICS
    }

    pub fn get_cycle_count() -> u64 {
        sys_cycle_count()
    }

    pub fn report_cycle_count(name: &str, cycle_count: u64, free_heap_bytes: u64) {
        let serialized = serialize_metric(name, cycle_count, free_heap_bytes);
        risc0_zkvm::guest::env::send_recv_slice::<u8, u8>(SYSCALL_NAME_METRICS, &serialized);
    }

    /// Returns how many bytes of heap are still available
    pub fn get_available_heap() -> u64 {
        // TODO hack, this is allocating just to get a pointer to the top of the heap.
        // Assumes bump alloc
        // When embed alloc is fixed https://github.com/risc0/risc0/pull/2677 can use that.
        let new_alloc = black_box(Box::new(()));
        let available = 0x0C00_0000 - &new_alloc as *const _ as usize;
        available as u64
    }

    #[cfg(all(test, feature = "native"))]
    mod tests {
        use super::*;

        fn check_in_out(name: &str, cycle_count: u64, free_heap_bytes: u64) {
            let serialized = serialize_metric(name, cycle_count, free_heap_bytes);

            let (de_name, de_cycles, de_heap) = deserialize_metrics_call(&serialized[..]).unwrap();

            assert_eq!(de_name, name, "wrong metric name");
            assert_eq!(de_cycles, cycle_count, "wrong cycle count");
            assert_eq!(de_heap, free_heap_bytes, "wrong free heap");
        }

        #[test]
        fn callback_serialize_and_deserialize() {
            let cases = vec![
                ("zeros", 0, 0),
                ("something", 1024, 4095),
                ("different", 9056, 3870),
                ("one_max", u64::MAX, 514),
                ("two_max", 512, u64::MAX),
            ];
            for (name, cycles, heap_bytes) in cases {
                check_in_out(name, cycles, heap_bytes);
            }
        }
    }
}

#[cfg(not(feature = "risc0"))]
pub use facade::*;
#[cfg(not(feature = "risc0"))]
mod facade {
    pub fn get_cycle_count() -> u64 {
        0
    }

    pub fn report_cycle_count(_name: &str, _count: u64, _free_heap_bytes: u64) {
        panic!("Reporting risc0 cycle count without risc0 feature enabled");
    }

    pub fn get_available_heap() -> u64 {
        0x0C00_0000
    }
}
