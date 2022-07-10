use core::fmt;

#[used]
#[link_section = ".note.hermit.entry-version"]
static ENTRY_VERSION: Note = Note {
    header: Nhdr32 {
        n_namesz: 7,
        n_descsz: 1,
        n_type: crate::NT_HERMIT_ENTRY_VERSION,
    },
    name: *b"HERMIT\0\0",
    data: [1],
};

#[repr(C)]
struct Note {
    header: Nhdr32,
    name: [u8; 8],
    data: [u8; 1],
}

#[repr(C)]
struct Nhdr32 {
    n_namesz: u32,
    n_descsz: u32,
    n_type: u32,
}

use crate::{BootInfo, NetInfo, RawBootInfo, TlsInfo};

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

    /// Returns `None` if the boot info's magic number and version match, or else returns a shared
    /// reference to the boot info wrapped in `Some`.
    ///
    /// # Safety
    ///
    /// When calling this method, you have to ensure that all of the following is true:
    ///
    /// * The pointer must be properly aligned.
    ///
    /// * The magic number and the version's fields must be "dereferenceable" in the sense defined
    ///   in [`core::ptr#safety`].
    ///
    /// * If the magic number and the version match, the pointer must be "dereferenceable" as a whole.
    ///
    /// * You must enforce Rust's aliasing rules, since the returned lifetime `'a` is
    ///   arbitrarily chosen and does not necessarily reflect the actual lifetime of the data.
    ///   In particular, for the duration of this lifetime, the memory the pointer points to must
    ///   not get mutated (except inside `UnsafeCell`).
    ///
    /// This applies even if the result of this method is unused!
    pub unsafe fn try_from_ptr<'a>(this: *const Self) -> Result<&'a Self, ParseHeaderError> {
        #[derive(Clone, Copy)]
        #[repr(C)]
        struct RawBootInfoHeader {
            magic_number: u32,
            version: u32,
        }

        let RawBootInfoHeader {
            magic_number,
            version,
        } = unsafe {
            // SAFETY: The caller must guarantee that `this` meets all the requirements to be
            // dereferenced as `RawBootInfoHeader`.
            *this.cast()
        };

        if magic_number != Self::MAGIC_NUMBER {
            return Err(ParseHeaderError::InvalidMagicNumber { magic_number });
        }

        if version != Self::VERSION {
            return Err(ParseHeaderError::InvalidVersion { version });
        }

        // SAFETY: The caller must guarantee that `this` meets all the requirements to be
        // dereferenced, since magic number and version match.
        unsafe { Ok(&*this) }
    }

    pub fn increment_cpu_online(&self) {
        unsafe {
            let _ = core::intrinsics::atomic_xadd(core::ptr::addr_of!(self.cpu_online) as _, 1);
        }
    }

    pub fn load_boot_time(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.boot_gtod).read_volatile() }
    }

    pub fn store_boot_time(&self, boot_time: u64) {
        unsafe { (core::ptr::addr_of!(self.boot_gtod) as *mut u64).write_volatile(boot_time) }
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

#[derive(Debug)]
pub enum ParseHeaderError {
    InvalidMagicNumber { magic_number: u32 },
    InvalidVersion { version: u32 },
}

impl fmt::Display for ParseHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseHeaderError::InvalidMagicNumber { magic_number } => {
                let expected = RawBootInfo::MAGIC_NUMBER;
                write!(
                    f,
                    "invalid magic number (expected {expected:?}, found {magic_number:?})"
                )
            }
            ParseHeaderError::InvalidVersion { version } => {
                let expected = RawBootInfo::VERSION;
                write!(
                    f,
                    "invalid version (expected {expected:?}, found {version:?})"
                )
            }
        }
    }
}
