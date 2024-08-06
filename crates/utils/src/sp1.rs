pub extern crate sp1_zkvm;
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
