//! # RustyHermit's entry API.
//!
//! This crate unifies RustyHermit's entry API ([`Entry`]) for all loaders and the kernel.
//!
//! This crate also parses and loads RustyHermit applications ([`elf`]).

#![no_std]
#![cfg_attr(feature = "kernel", feature(const_ptr_offset_from))]
#![cfg_attr(feature = "kernel", feature(const_refs_to_cell))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(missing_docs)]

pub mod boot_info;

#[cfg(feature = "loader")]
pub mod elf;

#[cfg(feature = "kernel")]
mod note;

#[cfg(feature = "kernel")]
pub use note::_Note;

/// Kernel entry point.
///
/// This is the signature of the entry point of the kernel.
pub type Entry = unsafe extern "C" fn(raw_boot_info: &'static boot_info::RawBootInfo) -> !;

/// Note type for specifying the hermit entry version.
///
/// The note name for this is `HERMIT`.
///
/// The `desc` field will be 1 word, which specifies the hermit entry version.
const NT_HERMIT_ENTRY_VERSION: u32 = 0x5a00;

/// The current hermit entry version.
const HERMIT_ENTRY_VERSION: u8 = 1;
