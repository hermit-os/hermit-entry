//! # Hermit's loading and entry API.
//!
//! This crate parses and loads Hermit applications ([`elf`]).
//!
//! Additionally, this crate unifies Hermit's entry API ([`Entry`]) for all loaders and the kernel.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

pub mod boot_info;

#[cfg(feature = "loader")]
pub mod elf;

#[cfg(feature = "kernel")]
mod note;

use core::error::Error;
use core::fmt;
use core::str::FromStr;

#[doc(hidden)]
pub use const_parse::parse_u128 as _parse_u128;
#[cfg(feature = "kernel")]
#[doc(hidden)]
pub use note::{_AbiTag, _Note};

/// GZIP magic number.
///
/// For details, see [10.17487/RFC1952](https://doi.org/10.17487/RFC1952).
#[cfg(feature = "loader")]
const GZIPMAG: &[u8; 3] = &[0x1f, 0x8b, 0x08];

/// Possible input formats for a Hermit loader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg(feature = "loader")]
pub enum Format {
    /// An ELF file, probably a Hermit kernel.
    Elf,
    /// A gzipped tar file, probably containing a config + ELF kernel image, and associated files.
    Gzip,
}

/// Attempts to detect the format of an input file (using magic bytes), whether it is an ELF kernel or an image.
#[cfg(feature = "loader")]
pub fn detect_format(data: &[u8]) -> Option<Format> {
    if data.len() < 4 {
        None
    } else if data.starts_with(goblin::elf64::header::ELFMAG) {
        Some(Format::Elf)
    } else if data.starts_with(GZIPMAG) {
        Some(Format::Gzip)
    } else {
        None
    }
}

/// Kernel entry point.
///
/// This is the signature of the entry point of the kernel.
///
/// `cpu_id` is the number of the CPU core with the boot processor being number 0.
///
/// The stack pointer has to be valid for the boot processor only.
#[cfg(not(target_arch = "riscv64"))]
pub type Entry =
    unsafe extern "C" fn(raw_boot_info: &'static boot_info::RawBootInfo, cpu_id: u32) -> !;

/// Kernel entry point.
///
/// This is the signature of the entry point of the kernel.
///
/// `hart_id` is the number of the hardware thread.
///
/// The stack pointer has to be valid for the boot processor only.
#[cfg(target_arch = "riscv64")]
pub type Entry =
    unsafe extern "C" fn(hart_id: usize, raw_boot_info: &'static boot_info::RawBootInfo) -> !;

/// Note type for specifying the hermit entry version.
///
/// The note name for this is `HERMIT`.
///
/// The `desc` field will be 1 word, which specifies the hermit entry version.
#[cfg_attr(not(any(feature = "loader", feature = "kernel")), expect(dead_code))]
const NT_HERMIT_ENTRY_VERSION: u32 = 0x5a00;

/// The current hermit entry version.
#[cfg_attr(not(any(feature = "loader", feature = "kernel")), expect(dead_code))]
const HERMIT_ENTRY_VERSION: u8 = 4;

/// Note type for specifying the Uhyve interface version in an elf header.
#[cfg_attr(not(any(feature = "loader", feature = "kernel")), expect(dead_code))]
const NT_UHYVE_INTERFACE_VERSION: u32 = 0x5b00;

/// Offsets and values used to interpret the boot params ("zeropage") setup by firecracker
/// For the full list of values see
/// <https://github.com/torvalds/linux/blob/b6839ef26e549de68c10359d45163b0cfb031183/arch/x86/include/uapi/asm/bootparam.h#L151-L198>
#[expect(missing_docs)]
#[deprecated]
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

#[cfg_attr(not(any(feature = "loader", feature = "kernel")), expect(dead_code))]
const NT_GNU_ABI_TAG: u32 = 1;
#[cfg_attr(not(any(feature = "loader", feature = "kernel")), expect(dead_code))]
const ELF_NOTE_OS_HERMIT: u32 = 6;

/// A Hermit version.
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
pub struct HermitVersion {
    /// The major version of Hermit.
    pub major: u32,

    /// The minor version of Hermit.
    pub minor: u32,

    /// The patch version of Hermit.
    pub patch: u32,
}

impl fmt::Display for HermitVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            major,
            minor,
            patch,
        } = self;
        write!(f, "{major}.{minor}.{patch}")
    }
}

impl FromStr for HermitVersion {
    type Err = ParseHermitVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major, rest) = s.split_once('.').ok_or(ParseHermitVersionError)?;
        let (minor, patch) = rest.split_once('.').ok_or(ParseHermitVersionError)?;

        let major = major.parse().map_err(|_| ParseHermitVersionError)?;
        let minor = minor.parse().map_err(|_| ParseHermitVersionError)?;
        let patch = patch.parse().map_err(|_| ParseHermitVersionError)?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

/// An error which can be returned when parsing a [`HermitVersion`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParseHermitVersionError;

impl fmt::Display for ParseHermitVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string could not be parsed as Hermit version")
    }
}

impl Error for ParseHermitVersionError {}

/// A Uhyve interface version.
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
pub struct UhyveIfVersion(pub u32);

impl fmt::Display for UhyveIfVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_hermit_version() {
        let small = HermitVersion {
            major: 0,
            minor: 1,
            patch: 2,
        };
        let big = HermitVersion {
            major: 2,
            minor: 1,
            patch: 0,
        };

        assert!(small < big);
        assert!(small == small);
        assert!(big == big);
        assert!(big > small);
    }

    #[test]
    fn parse_hermit_version() {
        let version = HermitVersion::from_str("0.1.2").unwrap();
        assert_eq!(
            version,
            HermitVersion {
                major: 0,
                minor: 1,
                patch: 2,
            }
        );

        let version = HermitVersion::from_str("2.1.0").unwrap();
        assert_eq!(
            version,
            HermitVersion {
                major: 2,
                minor: 1,
                patch: 0,
            }
        );
    }
}
