//! # RustyHermit's entry API.

#![no_std]
#![cfg_attr(feature = "kernel", feature(const_ptr_offset_from))]
#![cfg_attr(feature = "kernel", feature(const_refs_to_cell))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(missing_docs)]

#[cfg(feature = "loader")]
mod loader;

#[cfg(feature = "loader")]
pub use loader::elf::{KernelObject, LoadedKernel, ParseKernelError};

#[cfg(feature = "kernel")]
mod kernel;

#[cfg(feature = "kernel")]
pub use kernel::_Note;

use core::{
    num::{NonZeroU32, NonZeroU64},
    ops::Range,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

use time::OffsetDateTime;

/// Kernel entry point.
///
/// This is the signature of the entry point of the kernel.
pub type Entry = unsafe extern "C" fn(raw_boot_info: &'static RawBootInfo) -> !;

mod consts {
    /// Note type for specifying the hermit entry version.
    ///
    /// The note name for this is `HERMIT`.
    ///
    /// The `desc` field will be 1 word, which specifies the hermit entry version.
    pub const NT_HERMIT_ENTRY_VERSION: u32 = 0x5a00;

    /// The current hermit entry version.
    pub const HERMIT_ENTRY_VERSION: u8 = 1;
}

#[cfg(feature = "loader")]
pub use consts::NT_HERMIT_ENTRY_VERSION;

#[cfg(feature = "loader")]
pub use consts::HERMIT_ENTRY_VERSION;

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
#[derive(Debug)]
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
#[derive(Debug)]
#[repr(C)]
pub struct RawBootInfo {
    /// Magic number (legacy)
    ///
    /// Used for identifying a `RawBootInfo` struct.
    magic_number: u32,

    /// Boot info version (legacy)
    ///
    /// Used to agree on the layout of `RawBootInfo`.
    /// Not necessary since the introduction of the entry version note.
    version: u32,

    base: u64,
    #[cfg(target_arch = "aarch64")]
    ram_start: u64,
    limit: u64,
    image_size: u64,
    tls_start: u64,
    tls_filesz: u64,
    tls_memsz: u64,
    #[cfg(target_arch = "aarch64")]
    tls_align: u64,
    current_stack_address: AtomicU64,

    /// The current percore address (legacy).
    ///
    /// libhermit-rs now uses an internal statically allocated variable.
    current_percore_address: u64,

    /// Virtual host address (legacy)
    ///
    /// Used by HermitCore for sharing a memory pool with uhyve at the same host and guest virtual address.
    ///
    /// <https://github.com/hermitcore/libhermit/commit/9a28225424519cd6ab2b42fb5a2997455ba03242>
    host_logical_addr: u64,

    boot_gtod: u64,
    #[cfg(target_arch = "x86_64")]
    mb_info: u64,
    cmdline: u64,
    cmdsize: u64,
    cpu_freq: u32,

    /// CPU ID of the boot processor (legacy)
    ///
    /// Used by HermitCore to identify the processor core that is the boot processor.
    /// libhermit-rs defaults to 0.
    boot_processor: u32,

    cpu_online: AtomicU32,

    possible_cpus: u32,

    /// CPU ID of the currently booting processor (legacy)
    ///
    /// Used by HermitCore to identify the processor core that is currently booting.
    /// libhermit-rs deduces this from `cpu_online`.
    current_boot_id: u32,

    #[cfg(target_arch = "x86_64")]
    uartport: u16,
    #[cfg(target_arch = "aarch64")]
    uartport: u32,

    /// Single Kernel (legacy)
    ///
    /// This bool was used to determine whether HermitCore is the only kernel on the machine
    /// or if it is running in multikernel mode side by side with Linux.
    single_kernel: u8,

    uhyve: u8,

    /// Uhyve IP Address (legacy)
    ///
    /// Was used by lwIP once.
    hcip: [u8; 4],

    /// Uhyve Gateway Address (legacy)
    ///
    /// Was used by lwIP once.
    hcgateway: [u8; 4],

    /// Uhyve Network Mask (legacy)
    ///
    /// Was used by lwIP once.
    hcmask: [u8; 4],

    #[cfg(target_arch = "x86_64")]
    tls_align: u64,
}

impl RawBootInfo {
    /// Stores the current stack address.
    pub fn store_current_stack_address(&self, current_stack_address: u64) {
        self.current_stack_address
            .store(current_stack_address, Ordering::Relaxed);
    }

    /// Returns the number of initialized CPUs.
    // Used by uhyve to synchronize vCPU startup.
    pub fn load_cpu_online(&self) -> u32 {
        self.cpu_online.load(Ordering::Acquire)
    }
}
