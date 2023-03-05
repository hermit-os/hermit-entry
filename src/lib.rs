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
const HERMIT_ENTRY_VERSION: u8 = 2;

/// Offsets and values used to interpret the boot params ("zeropage") setup by firecracker
/// For the full list of values see
/// https://github.com/torvalds/linux/blob/b6839ef26e549de68c10359d45163b0cfb031183/arch/x86/include/uapi/asm/bootparam.h#L151-L198
#[allow(missing_docs)]
pub mod fc {
    pub const LINUX_KERNEL_BOOT_FLAG_MAGIC: u16 = 0xaa55;
    pub const LINUX_KERNEL_HRD_MAGIC: u32 = 0x53726448;
    pub const LINUX_SETUP_HEADER_OFFSET: usize = 0x1f1;
    pub const BOOT_FLAG_OFFSET: usize = 13;
    pub const HDR_MAGIC_OFFSET: usize = 17;
    pub const E820_ENTRIES_OFFSET: usize = 0x1e8;
    pub const E820_TABLE_OFFSET: usize = 0x2d0;
    pub const RAMDISK_IMAGE_OFFSET: usize = 39;
    pub const RAMDISK_SIZE_OFFSET: usize = 43;
    pub const CMD_LINE_PTR_OFFSET: usize = 55;
    pub const CMD_LINE_SIZE_OFFSET: usize = 71;
}
