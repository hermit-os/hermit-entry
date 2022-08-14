//! Creating and reading [`RawBootInfo`] from [`Entry`](crate::Entry).
//!
//! Loaders assemble [`BootInfo`] and convert it into a [`RawBootInfo`] to pass it to the kernel.
//!
//! The kernel copies [`BootInfo`] from [`RawBootInfo`] to work with the values.

#[cfg(feature = "loader")]
mod loader;

#[cfg(feature = "kernel")]
mod kernel;

use core::{
    num::{NonZeroU32, NonZeroU64},
    ops::Range,
};

use time::OffsetDateTime;

/// Serial I/O port.
#[cfg(target_arch = "x86_64")]
pub type SerialPortBase = core::num::NonZeroU16;

/// Serial port base address
#[cfg(target_arch = "aarch64")]
pub type SerialPortBase = core::num::NonZeroU64;

/// Boot information.
///
/// This struct is built by the loader and consumed by the kernel.
/// It contains information on how the kernel image was loaded as well as
/// additional hardware and loader specific information.
#[derive(Debug)]
pub struct BootInfo {
    /// Hardware information.
    pub hardware_info: HardwareInfo,

    /// Load information.
    pub load_info: LoadInfo,

    /// Platform information.
    pub platform_info: PlatformInfo,
}

/// Hardware information.
#[derive(Debug)]
pub struct HardwareInfo {
    /// The range of all possible physical memory addresses.
    pub phys_addr_range: Range<u64>,

    /// Serial port base address.
    pub serial_port_base: Option<SerialPortBase>,
}

/// Load information.
#[derive(Debug)]
pub struct LoadInfo {
    /// The virtual address range of the loaded kernel image.
    pub kernel_image_addr_range: Range<u64>,

    /// Kernel image TLS information.
    pub tls_info: Option<TlsInfo>,
}

/// Platform information.
///
/// This struct holds platform and loader specific information.
#[derive(Debug)]
pub enum PlatformInfo {
    /// Multiboot.
    #[cfg(target_arch = "x86_64")]
    Multiboot {
        /// Command line passed to the kernel.
        command_line: Option<&'static str>,

        /// Multiboot boot information address.
        multiboot_info_addr: core::num::NonZeroU64,
    },
    /// Direct Linux Boot.
    #[cfg(target_arch = "aarch64")]
    LinuxBoot,
    /// Uhyve.
    Uhyve {
        /// PCI support.
        has_pci: bool,

        /// Total number of CPUs available.
        num_cpus: NonZeroU64,

        /// CPU frequency in kHz.
        cpu_freq: Option<NonZeroU32>,

        /// Boot time.
        boot_time: OffsetDateTime,
    },
}

/// Thread local storage (TLS) image information.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TlsInfo {
    /// The start address of the TLS image.
    pub start: u64,

    /// `filesz` of the TLS program header.
    pub filesz: u64,

    /// `memsz` of the TLS program header.
    pub memsz: u64,

    /// `align` of the TLS program header.
    pub align: u64,
}

/// The raw boot information struct.
///
/// This is kept separate from [`BootInfo`] to make non-breaking API evolution possible.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct RawBootInfo {
    hardware_info: RawHardwareInfo,
    load_info: RawLoadInfo,
    platform_info: RawPlatformInfo,
}

#[cfg_attr(not(feature = "kernel"), allow(dead_code))]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct RawHardwareInfo {
    phys_addr_start: u64,
    phys_addr_end: u64,
    serial_port_base: Option<SerialPortBase>,
}

#[cfg_attr(not(feature = "kernel"), allow(dead_code))]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct RawLoadInfo {
    kernel_image_addr_start: u64,
    kernel_image_addr_end: u64,
    tls_info: TlsInfo,
}

#[cfg_attr(not(all(feature = "loader", feature = "kernel")), allow(dead_code))]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
enum RawPlatformInfo {
    #[cfg(target_arch = "x86_64")]
    Multiboot {
        command_line_data: *const u8,
        command_line_len: u64,
        multiboot_info_addr: core::num::NonZeroU64,
    },
    #[cfg(target_arch = "aarch64")]
    LinuxBoot,
    Uhyve {
        has_pci: bool,
        num_cpus: NonZeroU64,
        cpu_freq: Option<NonZeroU32>,
        boot_time: i128,
    },
}
