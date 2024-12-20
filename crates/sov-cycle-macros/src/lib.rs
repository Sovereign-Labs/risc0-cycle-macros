#![deny(missing_docs)]
#![doc = include_str!("../../../README.md")]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{parse2, parse_quote, Block, Ident, ItemFn};

/// This macro is used to annotate functions that we want to track the number of riscV cycles being
/// generated inside the VM. The purpose of the this macro is to measure how many cycles a rust
/// function takes because prover time is directly proportional to the number of riscv cycles
/// generated. It does this by making use of a risc0 provided function
/// ```rust,ignore
/// risc0_zkvm_platform::syscall::sys_cycle_count
/// ```
/// The macro essentially generates new function with the same name by wrapping the body with a sys_cycle_count
/// at the beginning and end of the function, subtracting it and then emitting it out using the
/// a custom syscall that is generated when the prover is run with the `bench` feature.
/// `send_recv_slice` is used to communicate and pass a slice to the syscall that we defined.
/// The handler for the syscall can be seen in adapters/risc0/src/host.rs and adapters/risc0/src/metrics.rs
#[proc_macro_attribute]
pub fn cycle_tracker(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = TokenStream2::from(item);
    wrap_function_with(cycles, input)
        .unwrap_or_else(|err| err.to_compile_error().into())
        .into()
}

/// Wrap a block with benchmarking. Fills the correct cycle counter based on the target and vendor.
fn cycles(ident: &Ident, block: &Block) -> Box<Block> {
    let risc0_block = cycles_risc0(ident, block);
    let sp1_block = cycles_sp1(ident, block);

    // Too much to require cfg-if at the call site.
    parse_quote!({
        #[cfg(all(target_os = "zkvm", target_vendor = "risc0"))] return #risc0_block;
        #[cfg(all(target_os = "zkvm", target_vendor = "succinct"))] return #sp1_block;
        #[cfg(not(all(target_os = "zkvm", any(target_vendor = "risc0", target_vendor = "succinct"))))]
        #block
    })
}

/// Wrap a function, where `f` wraps a block with (benchmarking) code.
fn wrap_function_with<F>(f: F, input: TokenStream2) -> Result<TokenStream2, syn::Error>
where
    F: Fn(&Ident, &Block) -> Box<Block>,
{
    let mut input = parse2::<ItemFn>(input)?;
    let ItemFn {
        sig: syn::Signature { ident, .. },
        block,
        ..
    } = &input;
    input.block = f(ident, block);

    Ok(input.into_token_stream().into())
}

fn cycles_risc0(ident: &Ident, block: &Block) -> Box<Block> {
    parse_quote! {
        {
            let cycles_before = ::sov_cycle_utils::risc0::get_cycle_count();
            let result = (|| #block)();
            let cycles_after = ::sov_cycle_utils::risc0::get_cycle_count();
            let heap_bytes_free_after = ::sov_cycle_utils::risc0::get_available_heap();

            let cycles = cycles_after.saturating_sub(cycles_before);
            ::sov_cycle_utils::risc0::report_cycle_count(stringify!(#ident), cycles, heap_bytes_free_after);
            result
        }
    }
}

fn cycles_sp1(ident: &Ident, block: &Block) -> Box<Block> {
    parse_quote!({
       {
            let before = ::sov_cycle_utils::sp1::get_cycle_count();
            let result = (move || #block)();
            let after = ::sov_cycle_utils::sp1::get_cycle_count();
            let heap_bytes_free_after = ::sov_cycle_utils::sp1::get_available_heap();

            ::sov_cycle_utils::sp1::report_cycle_count(stringify!(#ident), after - before, heap_bytes_free_after);
            result
        }
    })
}
