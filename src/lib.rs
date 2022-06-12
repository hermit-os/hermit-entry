//! # RustyHermit's entry API.

#![no_std]
#![cfg_attr(feature = "kernel", feature(const_ptr_offset_from))]
#![cfg_attr(feature = "kernel", feature(core_intrinsics))]

#[cfg(feature = "loader")]
mod loader;

#[cfg(feature = "loader")]
pub use loader::Entry;

#[cfg(feature = "kernel")]
mod kernel;

#[cfg(feature = "kernel")]
pub use kernel::ParseHeaderError;

#[cfg(target_arch = "x86_64")]
type SerialPortBase = u16;
#[cfg(target_arch = "aarch64")]
type SerialPortBase = u32;

#[derive(Debug)]
pub struct BootInfo {
    pub base: u64,
    pub limit: u64,
    pub image_size: u64,
    pub tls_info: TlsInfo,
    pub current_stack_address: u64,
    pub current_percore_address: u64,
    pub host_logical_addr: u64,
    pub boot_gtod: u64,
    pub cmdline: u64,
    pub cmdsize: u64,
    pub cpu_freq: u32,
    pub boot_processor: u32,
    pub cpu_online: u32,
    pub possible_cpus: u32,
    pub current_boot_id: u32,
    pub uartport: SerialPortBase,
    pub single_kernel: u8,
    pub uhyve: u8,
    pub net_info: NetInfo,
    #[cfg(target_arch = "x86_64")]
    pub mb_info: u64,
    #[cfg(target_arch = "aarch64")]
    pub ram_start: u64,
}

#[derive(Debug)]
pub struct NetInfo {
    pub ip: [u8; 4],
    pub gateway: [u8; 4],
    pub mask: [u8; 4],
}

#[derive(Debug)]
pub struct TlsInfo {
    pub start: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

#[repr(C)]
pub struct RawBootInfo {
    magic_number: u32,
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
    current_stack_address: u64,
    current_percore_address: u64,
    host_logical_addr: u64,
    boot_gtod: u64,
    #[cfg(target_arch = "x86_64")]
    mb_info: u64,
    cmdline: u64,
    cmdsize: u64,
    cpu_freq: u32,
    boot_processor: u32,
    cpu_online: u32,
    possible_cpus: u32,
    current_boot_id: u32,
    uartport: SerialPortBase,
    single_kernel: u8,
    uhyve: u8,
    hcip: [u8; 4],
    hcgateway: [u8; 4],
    hcmask: [u8; 4],
    #[cfg(target_arch = "x86_64")]
    tls_align: u64,
}

impl RawBootInfo {
    pub fn load_cpu_online(&self) -> u32 {
        unsafe { core::ptr::addr_of!(self.cpu_online).read_volatile() }
    }

    const MAGIC_NUMBER: u32 = 0xC0DE_CAFEu32;
    const VERSION: u32 = 1;
}
