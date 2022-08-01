use core::{num::NonZeroU32, sync::atomic::Ordering};

use time::OffsetDateTime;

use crate::{BootInfo, PlatformInfo, RawBootInfo, SerialPortBase, TlsInfo};

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
    /// Copies boot information from [`RawBootInfo`].
    pub fn copy_from(raw_boot_info: &'_ RawBootInfo) -> Self {
        #[cfg(target_arch = "x86_64")]
        let phys_start = 0;
        #[cfg(target_arch = "aarch64")]
        let phys_start = raw_boot_info.ram_start;
        let phys_addr_range = phys_start..raw_boot_info.limit;

        let kernel_start = raw_boot_info.base;
        let kernel_image_addr_range = kernel_start..kernel_start + raw_boot_info.image_size;

        let uhyve = (raw_boot_info.uhyve & 0b1) == 0b1;
        let platform_info = if uhyve {
            let has_pci = (raw_boot_info.uhyve & 0b10) == 0b10;
            let boot_time =
                OffsetDateTime::from_unix_timestamp_nanos(raw_boot_info.boot_gtod as i128 * 1000)
                    .unwrap();

            PlatformInfo::Uhyve {
                has_pci,
                num_cpus: u64::from(raw_boot_info.possible_cpus).try_into().unwrap(),
                cpu_freq: NonZeroU32::new(raw_boot_info.cpu_freq * 1000),
                boot_time,
            }
        } else {
            #[cfg(target_arch = "x86_64")]
            {
                use core::{slice, str};

                let cmdline = raw_boot_info.cmdline as *const u8;
                let command_line = (!cmdline.is_null()).then(|| {
                    // SAFETY: cmdline and cmdsize are valid forever.
                    let slice =
                        unsafe { slice::from_raw_parts(cmdline, raw_boot_info.cmdsize as usize) };
                    str::from_utf8(slice).unwrap()
                });

                PlatformInfo::Multiboot {
                    command_line,
                    multiboot_info_addr: raw_boot_info.mb_info.try_into().unwrap(),
                }
            }

            #[cfg(target_arch = "aarch64")]
            {
                PlatformInfo::LinuxBoot
            }
        };

        let tls_info = {
            let (start, filesz, memsz, align) = (
                raw_boot_info.tls_start,
                raw_boot_info.tls_filesz,
                raw_boot_info.tls_memsz,
                raw_boot_info.tls_align,
            );

            (start != 0 || filesz != 0 || memsz != 0 || align != 0).then_some(TlsInfo {
                start,
                filesz,
                memsz,
                align,
            })
        };

        #[cfg(target_arch = "x86_64")]
        let uartport = raw_boot_info.uartport;
        #[cfg(target_arch = "aarch64")]
        let uartport = raw_boot_info.uartport.into();

        let serial_port_base =
            (raw_boot_info.uartport != 0).then_some(SerialPortBase::new(uartport).unwrap());

        Self {
            phys_addr_range,
            kernel_image_addr_range,
            tls_info,
            serial_port_base,
            platform_info,
        }
    }
}

impl RawBootInfo {
    /// `current_stack_address` offset.
    ///
    /// Returns the offset of the field `current_stack_address` from the beginning of the `RawBootInfo` struct.
    pub const fn current_stack_address_offset() -> usize {
        memoffset::offset_of!(Self, current_stack_address)
    }

    /// Returns the current stack address.
    pub fn load_current_stack_address(&self) -> u64 {
        self.current_stack_address.load(Ordering::Relaxed)
    }

    /// Increments the number of initialized CPUs.
    // Used to synchronize vCPU startup with uhyve.
    pub fn increment_cpu_online(&self) {
        self.cpu_online.fetch_add(1, Ordering::Release);
    }
}
