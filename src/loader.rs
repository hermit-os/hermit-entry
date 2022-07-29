use core::sync::atomic::{AtomicU32, AtomicU64};

use crate::{BootInfo, PlatformInfo, RawBootInfo};

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
        let image_size =
            boot_info.kernel_image_addr_range.end - boot_info.kernel_image_addr_range.start;

        #[cfg(target_arch = "x86_64")]
        let mb_info;

        let (cmdline, cmdsize, boot_gtod, cpu_freq, possible_cpus, uhyve) = match boot_info
            .platform_info
        {
            #[cfg(target_arch = "x86_64")]
            PlatformInfo::Multiboot {
                command_line,
                multiboot_info_addr,
            } => {
                mb_info = multiboot_info_addr.get();
                let (cmdline, cmdsize) = command_line
                    .map(|command_line| (command_line.as_ptr() as u64, command_line.len() as u64))
                    .unwrap_or_default();
                (cmdline, cmdsize, 0, 0, 0, 0)
            }
            #[cfg(target_arch = "aarch64")]
            PlatformInfo::LinuxBoot => Default::default(),
            PlatformInfo::Uhyve {
                has_pci,
                num_cpus,
                cpu_freq,
                boot_time,
            } => {
                #[cfg(target_arch = "x86_64")]
                {
                    mb_info = 0;
                }
                let uhyve = if has_pci { 0b11 } else { 0b1 };
                let boot_time = u64::try_from(boot_time.unix_timestamp_nanos() / 1000).unwrap();
                (
                    0,
                    0,
                    boot_time,
                    cpu_freq / 1000,
                    u32::try_from(num_cpus.get()).unwrap(),
                    uhyve,
                )
            }
        };

        #[cfg(target_arch = "x86_64")]
        assert_eq!(0, boot_info.phys_addr_range.start);

        let (tls_start, tls_filesz, tls_memsz, tls_align) = boot_info
            .tls_info
            .map(|tls_info| {
                (
                    tls_info.start,
                    tls_info.filesz,
                    tls_info.memsz,
                    tls_info.align,
                )
            })
            .unwrap_or_default();

        let uartport = boot_info
            .serial_port_base
            .map(|serial_port_base| serial_port_base.get())
            .unwrap_or_default();

        #[cfg(target_arch = "aarch64")]
        let uartport = u32::try_from(uartport).unwrap();

        Self {
            magic_number: Self::MAGIC_NUMBER,
            version: Self::VERSION,
            base: boot_info.kernel_image_addr_range.start,
            #[cfg(target_arch = "aarch64")]
            ram_start: boot_info.phys_addr_range.start,
            limit: boot_info.phys_addr_range.end,
            image_size,
            tls_start,
            tls_filesz,
            tls_memsz,
            tls_align,
            current_stack_address: Default::default(),
            current_percore_address: 0,
            host_logical_addr: Default::default(),
            boot_gtod,
            #[cfg(target_arch = "x86_64")]
            mb_info,
            cmdline,
            cmdsize,
            cpu_freq,
            boot_processor: !0,
            cpu_online: 0.into(),
            possible_cpus,
            current_boot_id: Default::default(),
            uartport,
            single_kernel: 1,
            uhyve,
            hcip: Default::default(),
            hcgateway: Default::default(),
            hcmask: Default::default(),
        }
    }
}
