#[cfg(feature = "sp1")]
pub use actual_impl::*;
#[cfg(feature = "sp1")]
mod actual_impl {
    /// File descriptor for the cycle count hook, which is used to get the cycle count.
    /// Can be any number, as long as it doesn't conflict with default/other hooks.
    pub const FD_CYCLE_COUNT_HOOK: u32 = 1000;
    /// File descriptor for the metrics hook, which is used to collect cycle duration data for functions.
    /// Can be any number, as long as it doesn't conflict with default/other hooks.
    pub const FD_METRICS_HOOK: u32 = 1001;

    pub fn cycle_count_hook(env: sp1_sdk::HookEnv, _buf: &[u8]) -> Vec<Vec<u8>> {
        vec![Vec::from(
            env.runtime.report.total_instruction_count().to_le_bytes(),
        )]
    }

    /// Report the cycle count to the host, if available. Otherwise, this is a no-op.
    pub fn report_cycle_count(name: &str, count: u64) {
        // Cheap serialization: concat the u64 (fixed size) with the string (unknown size).
        let mut buf = Vec::from(count.to_le_bytes());
        buf.extend_from_slice(name.as_bytes());
        sp1_lib::io::write(FD_METRICS_HOOK, &buf);
    }

    /// Get the current cycle count of the sp1 zkvm, if available. Otherwise, return 0.
    pub fn get_cycle_count() -> u64 {
        // Writing zero bytes is a no-op, so we write &[0].
        sp1_lib::io::write(FD_CYCLE_COUNT_HOOK, &[0]);
        u64::from_le_bytes(
            sp1_lib::io::read_vec()
                .try_into()
                .expect("Failed to read cycle count before hook."),
        )
    }
}

#[cfg(not(feature = "sp1"))]
pub use facade::*;

#[cfg(not(feature = "sp1"))]
mod facade {
    /// Get the current cycle count of the sp1 zkvm, if available. Otherwise, return 0.
    pub fn get_cycle_count() -> u64 {
        0
    }

    /// Report the cycle count to the host.
    pub fn report_cycle_count(_name: &str, _count: u64) {
        panic!("Reporting sp1 cycle count without sp1 feature enabled");
    }
}
