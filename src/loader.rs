use core::sync::atomic::{AtomicU32, AtomicU64};

use crate::{BootInfo, RawBootInfo, TlsInfo};

#[allow(clippy::derivable_impls)] // This is feature-gated
impl Default for BootInfo {
    fn default() -> Self {
        Self {
            base: Default::default(),
            limit: Default::default(),
            image_size: Default::default(),
            tls_info: Default::default(),
            boot_gtod: Default::default(),
            cmdline: Default::default(),
            cmdsize: Default::default(),
            cpu_freq: Default::default(),
            possible_cpus: Default::default(),
            uartport: Default::default(),
            uhyve: Default::default(),
            #[cfg(target_arch = "x86_64")]
            mb_info: Default::default(),
            #[cfg(target_arch = "aarch64")]
            ram_start: Default::default(),
        }
    }
}

#[allow(clippy::derivable_impls)] // This is feature-gated
impl Default for TlsInfo {
    fn default() -> Self {
        Self {
            start: Default::default(),
            filesz: Default::default(),
            memsz: Default::default(),
            align: Default::default(),
        }
    }
}

impl RawBootInfo {
    const MAGIC_NUMBER: u32 = 0xC0DE_CAFEu32;

    const VERSION: u32 = 1;

    pub const fn invalid() -> Self {
        Self {
            magic_number: 0,
            version: 0,
            base: 0,
            #[cfg(target_arch = "aarch64")]
            ram_start: 0,
            limit: 0,
            image_size: 0,
            tls_start: 0,
            tls_filesz: 0,
            tls_memsz: 0,
            tls_align: 0,
            current_stack_address: AtomicU64::new(0),
            current_percore_address: 0,
            host_logical_addr: 0,
            boot_gtod: 0,
            #[cfg(target_arch = "x86_64")]
            mb_info: 0,
            cmdline: 0,
            cmdsize: 0,
            cpu_freq: 0,
            boot_processor: 0,
            cpu_online: AtomicU32::new(0),
            possible_cpus: 0,
            current_boot_id: 0,
            uartport: 0,
            single_kernel: 0,
            uhyve: 0,
            hcip: [0; 4],
            hcgateway: [0; 4],
            hcmask: [0; 4],
        }
    }
}

impl From<BootInfo> for RawBootInfo {
    fn from(boot_info: BootInfo) -> Self {
        Self {
            magic_number: Self::MAGIC_NUMBER,
            version: Self::VERSION,
            base: boot_info.base,
            #[cfg(target_arch = "aarch64")]
            ram_start: boot_info.ram_start,
            limit: boot_info.limit,
            image_size: boot_info.image_size,
            tls_start: boot_info.tls_info.start,
            tls_filesz: boot_info.tls_info.filesz,
            tls_memsz: boot_info.tls_info.memsz,
            tls_align: boot_info.tls_info.align,
            current_stack_address: Default::default(),
            current_percore_address: 0,
            host_logical_addr: Default::default(),
            boot_gtod: boot_info.boot_gtod,
            #[cfg(target_arch = "x86_64")]
            mb_info: boot_info.mb_info,
            cmdline: boot_info.cmdline,
            cmdsize: boot_info.cmdsize,
            cpu_freq: boot_info.cpu_freq.into(),
            boot_processor: !0,
            cpu_online: 0.into(),
            possible_cpus: boot_info.possible_cpus,
            current_boot_id: Default::default(),
            uartport: boot_info.uartport,
            single_kernel: 1,
            uhyve: boot_info.uhyve,
            hcip: Default::default(),
            hcgateway: Default::default(),
            hcmask: Default::default(),
        }
    }
}
