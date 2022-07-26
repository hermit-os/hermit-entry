use crate::{NetInfo, RawBootInfo, SerialPortBase, TlsInfo};

/// Defines the hermit entry version in the note section.
#[macro_export]
macro_rules! define_entry_version {
    () => {
        #[used]
        #[link_section = ".note.hermit.entry-version"]
        static ENTRY_VERSION: $crate::_Note = $crate::_Note::entry_version();
    };
}

#[repr(C)]
#[doc(hidden)]
pub struct _Note {
    header: Nhdr32,
    name: [u8; 8],
    data: [u8; 1],
}

impl _Note {
    pub const fn entry_version() -> Self {
        Self {
            header: Nhdr32 {
                n_namesz: 7,
                n_descsz: 1,
                n_type: crate::consts::NT_HERMIT_ENTRY_VERSION,
            },
            name: *b"HERMIT\0\0",
            data: [crate::consts::HERMIT_ENTRY_VERSION],
        }
    }
}

#[repr(C)]
struct Nhdr32 {
    n_namesz: u32,
    n_descsz: u32,
    n_type: u32,
}

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

impl BootInfo {
    pub fn copy_from(raw_boot_info: &'_ RawBootInfo) -> Self {
        Self {
            base: raw_boot_info.base,
            limit: raw_boot_info.limit,
            image_size: raw_boot_info.image_size,
            tls_info: TlsInfo {
                start: raw_boot_info.tls_start,
                filesz: raw_boot_info.tls_filesz,
                memsz: raw_boot_info.tls_memsz,
                align: raw_boot_info.tls_align,
            },
            current_stack_address: raw_boot_info.current_stack_address,
            current_percore_address: raw_boot_info.current_percore_address,
            host_logical_addr: raw_boot_info.host_logical_addr,
            boot_gtod: raw_boot_info.boot_gtod,
            cmdline: raw_boot_info.cmdline,
            cmdsize: raw_boot_info.cmdsize,
            cpu_freq: raw_boot_info.cpu_freq,
            boot_processor: raw_boot_info.boot_processor,
            cpu_online: raw_boot_info.cpu_online,
            possible_cpus: raw_boot_info.possible_cpus,
            current_boot_id: raw_boot_info.current_boot_id,
            uartport: raw_boot_info.uartport,
            single_kernel: raw_boot_info.single_kernel,
            uhyve: raw_boot_info.uhyve,
            net_info: NetInfo {
                ip: raw_boot_info.hcip,
                gateway: raw_boot_info.hcgateway,
                mask: raw_boot_info.hcmask,
            },
            #[cfg(target_arch = "x86_64")]
            mb_info: raw_boot_info.mb_info,
            #[cfg(target_arch = "aarch64")]
            ram_start: raw_boot_info.ram_start,
        }
    }
}

impl RawBootInfo {
    pub const fn current_stack_address_offset() -> usize {
        memoffset::offset_of!(Self, current_stack_address)
    }

    pub fn increment_cpu_online(&self) {
        unsafe {
            let _ = core::intrinsics::atomic_xadd(core::ptr::addr_of!(self.cpu_online) as _, 1);
        }
    }

    pub fn load_current_percore_address(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.current_percore_address).read_volatile() }
    }

    pub fn store_current_percore_address(&self, current_percore_address: u64) {
        unsafe {
            (core::ptr::addr_of!(self.current_percore_address) as *mut u64)
                .write_volatile(current_percore_address)
        }
    }

    pub fn load_current_stack_address(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.current_stack_address).read_volatile() }
    }

    pub fn store_current_stack_address(&self, current_stack_address: u64) {
        unsafe {
            (core::ptr::addr_of!(self.current_stack_address) as *mut u64)
                .write_volatile(current_stack_address)
        }
    }
}
