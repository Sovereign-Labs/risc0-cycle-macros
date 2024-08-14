#[cfg(feature = "macros")]
pub use sov_cycle_macros as macros;
pub mod risc0;
pub mod sp1;

#[cfg(feature = "native")]
pub use native::{increment_metric, METRICS_HASHMAP};
#[cfg(feature = "native")]
mod native {
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// A global hashmap mapping metric names to their values.
    pub static METRICS_HASHMAP: Lazy<Mutex<HashMap<String, (u64, u64)>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    /// Increments the requested metric by the given value, creating a
    /// new entry in the global map if necessary.
    pub fn increment_metric(metric: String, value: u64) {
        let mut hashmap = METRICS_HASHMAP.lock().unwrap();
        hashmap
            .entry(metric)
            .and_modify(|(sum, count)| {
                *sum += value;
                *count += 1;
            })
            .or_insert((value, 1));
    }
}
