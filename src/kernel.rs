use core::sync::atomic::Ordering;

use crate::{BootInfo, RawBootInfo, TlsInfo};

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
            boot_gtod: raw_boot_info.boot_gtod,
            cmdline: raw_boot_info.cmdline,
            cmdsize: raw_boot_info.cmdsize,
            cpu_freq: raw_boot_info.cpu_freq.try_into().unwrap(),
            possible_cpus: raw_boot_info.possible_cpus,
            uartport: raw_boot_info.uartport,
            uhyve: raw_boot_info.uhyve,
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
        self.cpu_online.fetch_add(1, Ordering::Release);
    }

    pub fn load_current_stack_address(&self) -> u64 {
        self.current_stack_address.load(Ordering::Relaxed)
    }
}
