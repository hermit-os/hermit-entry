use core::fmt;

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

    pub unsafe fn try_from_ptr<'a>(this: *const Self) -> Result<&'a Self, ParseHeaderError> {
        let expected = Self::MAGIC_NUMBER;
        let found = (*this).magic_number;
        if found != expected {
            return Err(ParseHeaderError::InvalidMagicNumber { expected, found });
        }

        let expected = Self::VERSION;
        let found = (*this).version;
        if found != expected {
            return Err(ParseHeaderError::InvalidVersion { expected, found });
        }

        Ok(&*this)
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
    InvalidMagicNumber { expected: u32, found: u32 },
    InvalidVersion { expected: u32, found: u32 },
}

impl fmt::Display for ParseHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseHeaderError::InvalidMagicNumber { expected, found } => {
                write!(
                    f,
                    "invalid magic number (expected {expected:?}, found {found:?})"
                )
            }
            ParseHeaderError::InvalidVersion { expected, found } => {
                write!(
                    f,
                    "invalid version (expected {expected:?}, found {found:?})"
                )
            }
        }
    }
}
