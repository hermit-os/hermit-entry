/// Defines the hermit entry version in the note section.
///
/// This macro must be used in a module that is guaranteed to be linked.
/// See <https://github.com/rust-lang/rust/issues/99721>.
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
