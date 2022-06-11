use crate::{BootInfo, NetInfo, RawBootInfo, TlsInfo};

pub type Entry = extern "C" fn(raw_boot_info: &'static RawBootInfo) -> !;

impl Default for BootInfo {
    fn default() -> Self {
        Self {
            base: Default::default(),
            limit: Default::default(),
            image_size: Default::default(),
            tls_info: Default::default(),
            current_stack_address: Default::default(),
            current_percore_address: Default::default(),
            host_logical_addr: Default::default(),
            boot_gtod: Default::default(),
            cmdline: Default::default(),
            cmdsize: Default::default(),
            cpu_freq: Default::default(),
            boot_processor: !0,
            cpu_online: Default::default(),
            possible_cpus: Default::default(),
            current_boot_id: Default::default(),
            uartport: Default::default(),
            single_kernel: 1,
            uhyve: Default::default(),
            net_info: Default::default(),
            #[cfg(target_arch = "x86_64")]
            mb_info: Default::default(),
            #[cfg(target_arch = "aarch64")]
            ram_start: Default::default(),
        }
    }
}

impl Default for NetInfo {
    fn default() -> Self {
        Self {
            ip: [255, 255, 255, 255],
            gateway: [255, 255, 255, 255],
            mask: [255, 255, 255, 0],
        }
    }
}

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
    pub const INVALID: Self = Self {
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
        current_stack_address: 0,
        current_percore_address: 0,
        host_logical_addr: 0,
        boot_gtod: 0,
        #[cfg(target_arch = "x86_64")]
        mb_info: 0,
        cmdline: 0,
        cmdsize: 0,
        cpu_freq: 0,
        boot_processor: 0,
        cpu_online: 0,
        possible_cpus: 0,
        current_boot_id: 0,
        uartport: 0,
        single_kernel: 0,
        uhyve: 0,
        hcip: [0; 4],
        hcgateway: [0; 4],
        hcmask: [0; 4],
    };
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
            current_stack_address: boot_info.current_stack_address,
            current_percore_address: boot_info.current_percore_address,
            host_logical_addr: boot_info.host_logical_addr,
            boot_gtod: boot_info.boot_gtod,
            #[cfg(target_arch = "x86_64")]
            mb_info: boot_info.mb_info,
            cmdline: boot_info.cmdline,
            cmdsize: boot_info.cmdsize,
            cpu_freq: boot_info.cpu_freq,
            boot_processor: boot_info.boot_processor,
            cpu_online: boot_info.cpu_online,
            possible_cpus: boot_info.possible_cpus,
            current_boot_id: boot_info.current_boot_id,
            uartport: boot_info.uartport,
            single_kernel: boot_info.single_kernel,
            uhyve: boot_info.uhyve,
            hcip: boot_info.net_info.ip,
            hcgateway: boot_info.net_info.gateway,
            hcmask: boot_info.net_info.mask,
        }
    }
}
