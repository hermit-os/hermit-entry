//! Parsing and loading kernel objects from ELF files.

use core::mem::{self, MaybeUninit};
use core::{fmt, str};

use align_address::Align;
use goblin::elf::note::Nhdr32;
use goblin::elf::reloc::r_to_str;
use goblin::elf::section_header::{self, SHN_UNDEF};
use goblin::elf::sym::{self, STB_WEAK};
use goblin::elf64::dynamic::{self, Dyn, DynamicInfo};
use goblin::elf64::header::{self, Header};
use goblin::elf64::program_header::{self, ProgramHeader};
use goblin::elf64::reloc::{self, Rela};
use goblin::elf64::section_header::SectionHeader;
use goblin::elf64::sym::Sym;
use log::{info, warn};
use plain::Plain;

use crate::boot_info::{LoadInfo, TlsInfo};

// See https://refspecs.linuxbase.org/elf/x86_64-abi-0.98.pdf
#[cfg(target_arch = "x86_64")]
const ELF_ARCH: u16 = goblin::elf::header::EM_X86_64;
#[cfg(target_arch = "x86_64")]
const R_ABS64: u32 = goblin::elf::reloc::R_X86_64_64;
#[cfg(target_arch = "x86_64")]
const R_RELATIVE: u32 = goblin::elf::reloc::R_X86_64_RELATIVE;
#[cfg(target_arch = "x86_64")]
const R_GLOB_DAT: u32 = goblin::elf::reloc::R_X86_64_GLOB_DAT;

// See https://github.com/ARM-software/abi-aa/blob/2023Q3/aaelf64/aaelf64.rst#relocation
#[cfg(target_arch = "aarch64")]
const ELF_ARCH: u16 = goblin::elf::header::EM_AARCH64;
#[cfg(target_arch = "aarch64")]
const R_ABS64: u32 = goblin::elf::reloc::R_AARCH64_ABS64;
#[cfg(target_arch = "aarch64")]
const R_RELATIVE: u32 = goblin::elf::reloc::R_AARCH64_RELATIVE;
#[cfg(target_arch = "aarch64")]
const R_GLOB_DAT: u32 = goblin::elf::reloc::R_AARCH64_GLOB_DAT;

/// https://github.com/riscv-non-isa/riscv-elf-psabi-doc/blob/v1.0/riscv-elf.adoc#relocations
#[cfg(target_arch = "riscv64")]
const ELF_ARCH: u16 = goblin::elf::header::EM_RISCV;
#[cfg(target_arch = "riscv64")]
const R_ABS64: u32 = goblin::elf::reloc::R_RISCV_64;
#[cfg(target_arch = "riscv64")]
const R_RELATIVE: u32 = goblin::elf::reloc::R_RISCV_RELATIVE;

/// A parsed kernel object ready for loading.
pub struct KernelObject<'a> {
    /// The raw bytes of the parsed ELF file.
    elf: &'a [u8],

    /// The ELF file header at the beginning of [`Self::elf`].
    header: &'a Header,

    /// The kernel's program headers.
    ///
    /// Loadable program segments will be copied for execution.
    ///
    /// The thread-local storage segment will be used for creating [`TlsInfo`] for the kernel.
    phs: &'a [ProgramHeader],

    /// Relocations with an explicit addend.
    relas: &'a [Rela],

    /// Symbol table for relocations
    dynsyms: &'a [Sym],
}

struct NoteIterator<'a> {
    bytes: &'a [u8],
    align: usize,
}

#[derive(Debug)]
struct Note<'a> {
    ty: u32,
    name: &'a str,
    desc: &'a [u8],
}

impl<'a> Iterator for NoteIterator<'a> {
    type Item = Note<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let header = Nhdr32::from_bytes(self.bytes).ok()?;
        let mut offset = mem::size_of_val(header);
        let name = str::from_utf8(&self.bytes[offset..][..header.n_namesz as usize - 1]).unwrap();
        offset = (offset + header.n_namesz as usize).align_up(self.align);
        let desc = &self.bytes[offset..][..header.n_descsz as usize];
        offset = (offset + header.n_descsz as usize).align_up(self.align);
        self.bytes = &self.bytes[offset..];
        Some(Note {
            ty: header.n_type,
            name,
            desc,
        })
    }
}

fn iter_notes(bytes: &[u8], align: usize) -> NoteIterator<'_> {
    NoteIterator { bytes, align }
}

/// An error returned when parsing a kernel ELF fails.
#[derive(Debug)]
pub struct ParseKernelError(&'static str);

impl fmt::Display for ParseKernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info = self.0;
        write!(f, "invalid ELF: {info}")
    }
}

impl KernelObject<'_> {
    /// Parses raw bytes of an ELF file into a loadable kernel object.
    pub fn parse(elf: &[u8]) -> Result<KernelObject<'_>, ParseKernelError> {
        {
            let range = elf.as_ptr_range();
            let len = elf.len();
            info!("Parsing kernel from ELF at {range:?} (len = {len:#x} B / {len} B)");
        }

        let header = plain::from_bytes::<Header>(elf).unwrap();

        let phs = {
            let start = header.e_phoff as usize;
            let len = header.e_phnum as usize;
            ProgramHeader::slice_from_bytes_len(&elf[start..], len).unwrap()
        };

        let shs = {
            let start = header.e_shoff as usize;
            let len = header.e_shnum as usize;
            SectionHeader::slice_from_bytes_len(&elf[start..], len).unwrap()
        };

        // General compatibility checks
        {
            let class = header.e_ident[header::EI_CLASS];
            if class != header::ELFCLASS64 {
                return Err(ParseKernelError("kernel ist not a 64-bit object"));
            }
            let data_encoding = header.e_ident[header::EI_DATA];
            if data_encoding != header::ELFDATA2LSB {
                return Err(ParseKernelError("kernel object is not little endian"));
            }
            let os_abi = header.e_ident[header::EI_OSABI];
            if os_abi != header::ELFOSABI_STANDALONE {
                warn!("Kernel is not a hermit application");
            }

            let note_section = phs
                .iter()
                .find(|ph| ph.p_type == program_header::PT_NOTE)
                .ok_or(ParseKernelError("Kernel does not have note section"))?;
            let mut note_iter = iter_notes(
                &elf[note_section.p_offset as usize..][..note_section.p_filesz as usize],
                note_section.p_align as usize,
            );
            let note = note_iter
                .find(|note| note.name == "HERMIT" && note.ty == crate::NT_HERMIT_ENTRY_VERSION)
                .ok_or(ParseKernelError(
                    "Kernel does not specify hermit entry version",
                ))?;
            if note.desc[0] != crate::HERMIT_ENTRY_VERSION {
                return Err(ParseKernelError("hermit entry version does not match"));
            }

            if !matches!(header.e_type, header::ET_DYN | header::ET_EXEC) {
                return Err(ParseKernelError("kernel has unsupported ELF type"));
            }

            if header.e_machine != ELF_ARCH {
                return Err(ParseKernelError(
                    "kernel is not compiled for the correct architecture",
                ));
            }
        }

        let dyns = phs
            .iter()
            .find(|program_header| program_header.p_type == program_header::PT_DYNAMIC)
            .map(|ph| {
                let start = ph.p_offset as usize;
                let len = ph.p_filesz as usize;
                Dyn::slice_from_bytes(&elf[start..][..len]).unwrap()
            })
            .unwrap_or_default();

        if dyns.iter().any(|d| d.d_tag == dynamic::DT_NEEDED) {
            return Err(ParseKernelError(
                "kernel was linked against dynamic libraries",
            ));
        }

        let dynamic_info = DynamicInfo::new(dyns, phs);
        assert_eq!(0, dynamic_info.relcount);

        let relas = {
            let start = dynamic_info.rela;
            let len = dynamic_info.relasz;
            Rela::slice_from_bytes(&elf[start..][..len]).unwrap()
        };

        let dynsyms = shs
            .iter()
            .find(|section_header| section_header.sh_type == section_header::SHT_DYNSYM)
            .map(|sh| {
                let start = sh.sh_offset as usize;
                let len = sh.sh_size as usize;
                Sym::slice_from_bytes(&elf[start..][..len]).unwrap()
            })
            .unwrap_or_default();

        Ok(KernelObject {
            elf,
            header,
            phs,
            relas,
            dynsyms,
        })
    }

    /// Required memory size for loading.
    pub fn mem_size(&self) -> usize {
        let first_ph = self
            .phs
            .iter()
            .find(|ph| ph.p_type == program_header::PT_LOAD)
            .unwrap();
        let start_addr = first_ph.p_vaddr;

        let last_ph = self
            .phs
            .iter()
            .rev()
            .find(|ph| ph.p_type == program_header::PT_LOAD)
            .unwrap();
        let end_addr = last_ph.p_vaddr + last_ph.p_memsz;

        let mem_size = end_addr - start_addr;
        mem_size.try_into().unwrap()
    }

    fn is_relocatable(&self) -> bool {
        match self.header.e_type {
            header::ET_DYN => true,
            header::ET_EXEC => false,
            _ => unreachable!(),
        }
    }

    /// Returns the required start address.
    ///
    /// If this returns [`None`], the kernel is relocatable and does not require a certain start address.
    pub fn start_addr(&self) -> Option<u64> {
        (!self.is_relocatable()).then(|| {
            self.phs
                .iter()
                .find(|ph| ph.p_type == program_header::PT_LOAD)
                .unwrap()
                .p_vaddr
        })
    }

    fn tls_info(&self, start_addr: u64) -> Option<TlsInfo> {
        self.phs
            .iter()
            .find(|ph| ph.p_type == program_header::PT_TLS)
            .map(|ph| {
                let mut tls_start = ph.p_vaddr;
                if self.is_relocatable() {
                    tls_start += start_addr;
                }
                let tls_info = TlsInfo {
                    start: tls_start,
                    filesz: ph.p_filesz,
                    memsz: ph.p_memsz,
                    align: ph.p_align,
                };
                let range =
                    tls_info.start as *const ()..(tls_info.start + tls_info.memsz) as *const ();
                let len = tls_info.memsz;
                info!("TLS is at {range:?} (len =  {len:#x} B / {len} B)",);
                tls_info
            })
    }

    fn entry_point(&self, start_addr: u64) -> u64 {
        let mut entry_point = self.header.e_entry;
        if self.is_relocatable() {
            entry_point += start_addr;
        }
        entry_point
    }

    /// Loads the kernel into the provided memory.
    pub fn load_kernel(&self, memory: &mut [MaybeUninit<u8>], start_addr: u64) -> LoadedKernel {
        info!(
            "Loading kernel to {:?} (len = {len:#x} B / {len} B)",
            memory.as_ptr_range(),
            len = memory.len()
        );

        if !self.is_relocatable() {
            assert_eq!(self.start_addr().unwrap(), start_addr);
        }
        assert_eq!(self.mem_size(), memory.len());

        // Load program segments
        // Contains TLS initialization image
        let load_start_addr = self.start_addr().unwrap_or_default();
        self.phs
            .iter()
            .filter(|ph| ph.p_type == program_header::PT_LOAD)
            .for_each(|ph| {
                let ph_memory = {
                    let mem_start = (ph.p_vaddr - load_start_addr) as usize;
                    let mem_len = ph.p_memsz as usize;
                    &mut memory[mem_start..][..mem_len]
                };
                let file_len = ph.p_filesz as usize;
                let ph_file = &self.elf[ph.p_offset as usize..][..file_len];
                // FIXME: Replace with `maybe_uninit_write_slice` once stable
                let ph_file = unsafe { mem::transmute::<&[u8], &[MaybeUninit<u8>]>(ph_file) };
                ph_memory[..file_len].copy_from_slice(ph_file);
                for byte in &mut ph_memory[file_len..] {
                    byte.write(0);
                }
            });

        if self.is_relocatable() {
            // Perform relocations
            self.relas.iter().for_each(|rela| {
                match reloc::r_type(rela.r_info) {
                    R_ABS64 => {
                        let sym = reloc::r_sym(rela.r_info) as usize;
                        let sym = &self.dynsyms[sym];

                        if sym::st_bind(sym.st_info) == STB_WEAK
                            && u32::from(sym.st_shndx) == SHN_UNDEF
                        {
                            let memory = &memory[rela.r_offset as usize..][..8];
                            let memory =
                                unsafe { mem::transmute::<&[MaybeUninit<u8>], &[u8]>(memory) };
                            assert_eq!(memory, &[0; 8]);
                            return;
                        }

                        let relocated =
                            (start_addr as i64 + sym.st_value as i64 + rela.r_addend).to_ne_bytes();
                        let buf = &relocated[..];
                        // FIXME: Replace with `maybe_uninit_write_slice` once stable
                        let buf = unsafe { mem::transmute::<&[u8], &[MaybeUninit<u8>]>(buf) };
                        memory[rela.r_offset as usize..][..mem::size_of_val(&relocated)]
                            .copy_from_slice(buf);
                    }
                    R_RELATIVE => {
                        let relocated = (start_addr as i64 + rela.r_addend).to_ne_bytes();
                        let buf = &relocated[..];
                        // FIXME: Replace with `maybe_uninit_write_slice` once stable
                        let buf = unsafe { mem::transmute::<&[u8], &[MaybeUninit<u8>]>(buf) };
                        memory[rela.r_offset as usize..][..mem::size_of_val(&relocated)]
                            .copy_from_slice(buf);
                    }
                    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
                    R_GLOB_DAT => {
                        let sym = reloc::r_sym(rela.r_info) as usize;
                        let sym = &self.dynsyms[sym];

                        if sym::st_bind(sym.st_info) == STB_WEAK
                            && u32::from(sym.st_shndx) == SHN_UNDEF
                        {
                            let memory = &memory[rela.r_offset as usize..][..8];
                            let memory =
                                unsafe { mem::transmute::<&[MaybeUninit<u8>], &[u8]>(memory) };
                            assert_eq!(memory, &[0; 8]);
                            return;
                        }

                        let relocated =
                            (start_addr as i64 + sym.st_value as i64 + rela.r_addend).to_ne_bytes();
                        #[cfg(target_arch = "x86_64")]
                        assert_eq!(rela.r_addend, 0);
                        let buf = &relocated[..];
                        // FIXME: Replace with `maybe_uninit_write_slice` once stable
                        let buf = unsafe { mem::transmute::<&[u8], &[MaybeUninit<u8>]>(buf) };
                        memory[rela.r_offset as usize..][..mem::size_of_val(&relocated)]
                            .copy_from_slice(buf);
                    }
                    typ => panic!("unkown relocation type {}", r_to_str(typ, ELF_ARCH)),
                }
            });
        }

        LoadedKernel {
            load_info: LoadInfo {
                kernel_image_addr_range: start_addr..start_addr + self.mem_size() as u64,
                tls_info: self.tls_info(start_addr),
            },
            entry_point: self.entry_point(start_addr),
        }
    }
}

/// Load information required by the loader.
#[derive(Debug)]
pub struct LoadedKernel {
    /// Load information required by the kernel.
    pub load_info: LoadInfo,

    /// The kernel's entry point.
    pub entry_point: u64,
}
