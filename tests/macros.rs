//! This file tests if the macros defined by the hermit-entry crate,
//! which affect linker .note entries, can actually be used.

hermit_entry::define_abi_tag!();
hermit_entry::define_entry_version!();
