use super::{
    BootInfo, HardwareInfo, LoadInfo, PlatformInfo, RawBootInfo, RawHardwareInfo, RawLoadInfo,
    RawPlatformInfo, TlsInfo,
};

impl From<HardwareInfo> for RawHardwareInfo {
    fn from(hardware_info: HardwareInfo) -> Self {
        Self {
            phys_addr_start: hardware_info.phys_addr_range.start,
            phys_addr_end: hardware_info.phys_addr_range.end,
            serial_port_base: hardware_info.serial_port_base,
            device_tree: hardware_info.device_tree,
        }
    }
}

impl From<LoadInfo> for RawLoadInfo {
    fn from(load_info: LoadInfo) -> Self {
        Self {
            kernel_image_addr_start: load_info.kernel_image_addr_range.start,
            kernel_image_addr_end: load_info.kernel_image_addr_range.end,
            tls_info: load_info.tls_info.unwrap_or(TlsInfo {
                start: 0,
                filesz: 0,
                memsz: 0,
                align: 0,
            }),
        }
    }
}

impl From<PlatformInfo> for RawPlatformInfo {
    fn from(platform_info: PlatformInfo) -> Self {
        match platform_info {
            #[cfg(target_arch = "x86_64")]
            PlatformInfo::Multiboot {
                command_line,
                multiboot_info_addr,
            } => Self::Multiboot {
                command_line_data: command_line
                    .map(|s| s.as_ptr())
                    .unwrap_or(core::ptr::null()),
                command_line_len: command_line.map(|s| s.len() as u64).unwrap_or(0),
                multiboot_info_addr,
            },
            #[cfg(target_arch = "aarch64")]
            PlatformInfo::LinuxBoot => Self::LinuxBoot,
            PlatformInfo::Uhyve {
                has_pci,
                num_cpus,
                cpu_freq,
                boot_time,
            } => Self::Uhyve {
                has_pci,
                num_cpus,
                cpu_freq,
                boot_time: boot_time.unix_timestamp_nanos(),
            },
        }
    }
}

impl From<BootInfo> for RawBootInfo {
    fn from(boot_info: BootInfo) -> Self {
        RawBootInfo {
            hardware_info: boot_info.hardware_info.into(),
            load_info: boot_info.load_info.into(),
            platform_info: boot_info.platform_info.into(),
        }
    }
}
