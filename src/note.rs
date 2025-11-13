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
                n_type: crate::NT_HERMIT_ENTRY_VERSION,
            },
            name: *b"HERMIT\0\0",
            data: [crate::HERMIT_ENTRY_VERSION],
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
        #[unsafe(link_section = ".note.ABI-tag")]
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
