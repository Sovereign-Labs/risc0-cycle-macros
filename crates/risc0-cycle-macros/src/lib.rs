#![deny(missing_docs)]
#![doc = include_str!("../../../README.md")]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

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
    let input = parse_macro_input!(item as ItemFn);

    wrap_function(input).unwrap_or_else(|err| err.to_compile_error().into())
}

fn wrap_function(input: ItemFn) -> Result<TokenStream, syn::Error> {
    let visibility = &input.vis;
    let name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let attributes = &input.attrs;
    let output = &input.sig.output;
    let block = &input.block;
    let generics = &input.sig.generics;
    let where_clause = &input.sig.generics.where_clause;
    let risc0_zkvm = syn::Ident::new("risc0_zkvm", proc_macro2::Span::call_site());
    let risc0_zkvm_platform =
        syn::Ident::new("risc0_zkvm_platform", proc_macro2::Span::call_site());

    let result = quote! {
        #( #attributes )*
        #visibility fn #name #generics (#inputs) #output #where_clause {
            let before = #risc0_zkvm_platform::syscall::sys_cycle_count();
            let result = (|| #block)();
            let after = #risc0_zkvm_platform::syscall::sys_cycle_count();

            // simple serialization to avoid pulling in bincode or other libs
            let tuple = (stringify!(#name).to_string(), (after - before) as u64);
            let mut serialized = Vec::new();
            serialized.extend(tuple.0.as_bytes());
            serialized.push(0);
            let size_bytes = tuple.1.to_le_bytes();
            serialized.extend(&size_bytes);

            // calculate the syscall name.

            const fn compute_const_syscall_name() -> risc0_zkvm_platform::syscall::SyscallName {
                let c_str = if let Ok(name) = std::ffi::CStr::from_bytes_with_nul(b"cycle_metrics\0") {
                    name
                } else {
                    panic!("Failed to create syscall name")
                };

                if let Ok(syscall_name) = risc0_zkvm_platform::syscall::SyscallName::from_c_str(&c_str) {
                    syscall_name
                } else {
                    panic!("Failed to create syscall name")
                }
            }
            println!("Making syscall!");

            let metrics_syscall_name = compute_const_syscall_name();
            #risc0_zkvm::guest::env::send_recv_slice::<u8,u8>(metrics_syscall_name, &serialized);
            println!("Done with syscall");
            result
        }
    };
    Ok(result.into())
}
