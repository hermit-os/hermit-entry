use time::OffsetDateTime;

use super::{
    BootInfo, HardwareInfo, LoadInfo, PlatformInfo, RawBootInfo, RawHardwareInfo, RawLoadInfo,
    RawPlatformInfo, TlsInfo,
};

impl From<RawHardwareInfo> for HardwareInfo {
    fn from(raw_hardware_info: RawHardwareInfo) -> Self {
        Self {
            phys_addr_range: raw_hardware_info.phys_addr_start..raw_hardware_info.phys_addr_end,
            serial_port_base: raw_hardware_info.serial_port_base,
        }
    }
}

impl From<RawLoadInfo> for LoadInfo {
    fn from(raw_load_info: RawLoadInfo) -> Self {
        let TlsInfo {
            start,
            filesz,
            memsz,
            align,
        } = raw_load_info.tls_info;

        Self {
            kernel_image_addr_range: raw_load_info.kernel_image_addr_start
                ..raw_load_info.kernel_image_addr_end,
            tls_info: (start != 0 || filesz != 0 || memsz != 0 || align != 0)
                .then_some(raw_load_info.tls_info),
        }
    }
}

impl From<RawPlatformInfo> for PlatformInfo {
    fn from(raw_platform_info: RawPlatformInfo) -> Self {
        match raw_platform_info {
            #[cfg(target_arch = "x86_64")]
            RawPlatformInfo::Multiboot {
                command_line_data,
                command_line_len,
                multiboot_info_addr,
            } => {
                let command_line = (!command_line_data.is_null()).then(|| {
                    // SAFETY: cmdline and cmdsize are valid forever.
                    let slice = unsafe {
                        core::slice::from_raw_parts(command_line_data, command_line_len as usize)
                    };
                    core::str::from_utf8(slice).unwrap()
                });

                Self::Multiboot {
                    command_line,
                    multiboot_info_addr,
                }
            }
            #[cfg(target_arch = "aarch64")]
            RawPlatformInfo::LinuxBoot => Self::LinuxBoot,
            RawPlatformInfo::Uhyve {
                has_pci,
                num_cpus,
                cpu_freq,
                boot_time,
            } => Self::Uhyve {
                has_pci,
                num_cpus,
                cpu_freq,
                boot_time: OffsetDateTime::from_unix_timestamp_nanos(boot_time).unwrap(),
            },
        }
    }
}

impl From<RawBootInfo> for BootInfo {
    fn from(raw_boot_info: RawBootInfo) -> Self {
        Self {
            hardware_info: raw_boot_info.hardware_info.into(),
            load_info: raw_boot_info.load_info.into(),
            platform_info: raw_boot_info.platform_info.into(),
        }
    }
}
