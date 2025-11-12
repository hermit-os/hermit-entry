use core::mem;

use crate::HermitVersion;

/// Defines the hermit entry version in the note section.
///
/// This macro must be used in a module that is guaranteed to be linked.
/// See <https://github.com/rust-lang/rust/issues/99721>.
#[macro_export]
macro_rules! define_entry_version {
    () => {
        #[used]
        #[unsafe(link_section = ".note.hermit.entry-version")]
        static ENTRY_VERSION: $crate::_Note<1> = $crate::_Note::entry_version();
    };
}

/// Defines the Uhyve interface version in the note section.
///
/// This macro must be used in a module that is guaranteed to be linked.
/// See <https://github.com/rust-lang/rust/issues/99721>.
#[macro_export]
macro_rules! define_uhyve_interface_version {
    () => {
        #[used]
        #[unsafe(link_section = ".note.hermit.uhyve-interface-version")]
        static INTERFACE_VERSION: $crate::_Note<4> =
            $crate::_Note::uhyveif_version(uhyve_interface::UHYVE_INTERFACE_VERSION);
    };
}

#[repr(C)]
#[doc(hidden)]
pub struct _Note<const N: usize> {
    header: Nhdr32,
    name: [u8; 8],
    data: [u8; N],
}

impl _Note<1> {
    pub const fn entry_version() -> Self {
        Self {
            header: Nhdr32 {
                n_namesz: 7,
                n_descsz: 1,
                n_type: crate::NT_HERMIT_ENTRY_VERSION,
            },
            name: *b"HERMIT\0\0",
            data: [crate::HERMIT_ENTRY_VERSION],
        }
    }
}

impl _Note<4> {
    pub const fn uhyveif_version(ver: u32) -> Self {
        Self {
            header: Nhdr32 {
                n_namesz: 8,
                n_descsz: 4,
                n_type: crate::NT_UHYVE_INTERFACE_VERSION,
            },
            name: *b"UHYVEIF\0",
            data: ver.to_be_bytes(),
        }
    }
}

#[repr(C)]
struct Nhdr32 {
    n_namesz: u32,
    n_descsz: u32,
    n_type: u32,
}

/// Defines the current Hermit kernel version in the note section.
///
/// The version is saved in `.note.ABI-tag` in accordance with [LSB].
///
/// [LSB]: https://refspecs.linuxfoundation.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/noteabitag.html
#[macro_export]
macro_rules! define_abi_tag {
    () => {
        #[used]
        #[link_section = ".note.ABI-tag"]
        static ABI_TAG: $crate::_AbiTag = $crate::_AbiTag::new($crate::HermitVersion {
            major: $crate::_parse_u128(::core::env!("CARGO_PKG_VERSION_MAJOR")) as u32,
            minor: $crate::_parse_u128(::core::env!("CARGO_PKG_VERSION_MINOR")) as u32,
            patch: $crate::_parse_u128(::core::env!("CARGO_PKG_VERSION_PATCH")) as u32,
        });
    };
}

#[repr(C)]
#[doc(hidden)]
pub struct _AbiTag {
    header: Nhdr32,
    name: [u8; 4],
    data: [u32; 4],
}

impl _AbiTag {
    pub const fn new(version: HermitVersion) -> Self {
        Self {
            header: Nhdr32 {
                n_namesz: mem::size_of::<[u8; 4]>() as u32,
                n_descsz: mem::size_of::<[u32; 4]>() as u32,
                n_type: crate::NT_GNU_ABI_TAG,
            },
            name: *b"GNU\0",
            data: [
                crate::ELF_NOTE_OS_HERMIT,
                version.major,
                version.minor,
                version.patch,
            ],
        }
    }
}
