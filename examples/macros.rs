//! This file tests if the macros defined by the hermit-entry crate,
//! which affect linker .note entries, can actually be used.

#[cfg(feature = "kernel")]
hermit_entry::define_abi_tag!();

#[cfg(feature = "kernel")]
hermit_entry::define_entry_version!();

fn main() {}
