//! # RustyHermit's entry API.
//!
//! This crate unifies RustyHermit's entry API ([`Entry`]) for all loaders and the kernel.
//!
//! This crate also parses and loads RustyHermit applications ([`elf`]).

#![no_std]
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
///
/// `cpu_id` is the number of the CPU core with the boot processor being number 0.
///
/// The stack pointer has to be valid for the boot processor only.
pub type Entry =
    unsafe extern "C" fn(raw_boot_info: &'static boot_info::RawBootInfo, cpu_id: u32) -> !;

/// Note type for specifying the hermit entry version.
///
/// The note name for this is `HERMIT`.
///
/// The `desc` field will be 1 word, which specifies the hermit entry version.
#[cfg_attr(not(all(feature = "loader", feature = "kernel")), allow(dead_code))]
const NT_HERMIT_ENTRY_VERSION: u32 = 0x5a00;

/// The current hermit entry version.
#[cfg_attr(not(all(feature = "loader", feature = "kernel")), allow(dead_code))]
const HERMIT_ENTRY_VERSION: u8 = 1;
