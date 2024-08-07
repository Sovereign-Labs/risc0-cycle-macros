## Introduction

- This repository contains two crates
  - `sov-cycle-macros`
  - `sov-cycle-utils`

## sov-cycle-macros

- Contains the `cycle-tracker` macro which can be used to annotate functions that run inside the risc0 vm
- To use the macro, the following changes need to be made:
- Cargo.toml

```toml
[dependencies]
sov-cycle-macros = { git = "https://github.com/Sovereign-Labs/sov-cycle-macros.git", rev="362a5a7", optional = true }
sov-cycle-utils = { git = "https://github.com/Sovereign-Labs/sov-cycle-macros.git", rev="362a5a7", optional = true }
risc0-zkvm = { version = "0.16", default-features = false, features = ["std"], optional=true}
risc0-zkvm-platform = { version = "0.16", optional=true}

[features]
bench = ["sov-cycle-macros", "sov-cycle-utils", "risc0-zkvm", "risc0-zkvm-platform"]
```

- The feature gating is needed because we don't want the cycle tracker scaffolding to be used unless the `bench` feature is enabled
- If the `bench` feature is not enabled, the risc0 host will not be built with the necessary syscalls to support tracking cycles
- The additional imports are necessary because the macro wraps the user function with the necessary code for tracking the number of cycles before and after the function execution
- The rust code that needs to use the `cycle_tracker` macro needs to import it and then annotate the function with it

```rust,ignore
//
#[cfg(all(target_os = "zkvm", feature = "bench"))]
use sov_cycle_macros::cycle_tracker;
//
//
#[cfg_attr(all(target_os = "zkvm", feature = "bench"), cycle_tracker)]
fn begin_slot(
    &mut self,
    slot_data: &impl SlotData<Cond = Cond>,
    witness: <Self as StateTransitionFunction<Vm, B>>::Witness,
) {
    let state_checkpoint = StateCheckpoint::with_witness(self.current_storage.clone(), witness);

    let mut working_set = state_checkpoint.to_revertable();

    self.runtime.begin_slot_hook(slot_data, &mut working_set);

    self.checkpoint = Some(working_set.checkpoint());
}
```

## sov-cycle-utils

This crate contains utilities that provide the syscall name and helper functions for printing the cycle count
