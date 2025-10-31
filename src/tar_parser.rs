//! A This module implements a parser for uncompressed tar files,
//! including the ustar extension (but not yet supporting the PAX extension).

use core::ops::Range;

use crate::filename::{Filename, truncate};

/// A parser for an already decompressed image / tar ball
#[derive(Clone, Copy)]
pub struct Parser<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> Parser<'a> {
    /// Constructs a new parser from a given tar byte slice
    pub fn new(input: &'a Bytes) -> Self {
        Self {
            input: &input.0,
            offset: 0,
        }
    }
}

/// Metadata for a tar file entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileMetadata {
    /// Is the file entry marked as executable?
    pub is_exec: bool,
}

/// A file entry in a tar ball
#[derive(Clone, Debug)]
pub struct File<'a> {
    /// The full file name of the entry
    pub name: Filename<'a>,

    /// The associated file metadata
    pub metadata: FileMetadata,

    /// The indices range of the value associated to this entry
    pub value_range: Range<usize>,

    /// A slice of the value (file content) associated to this entry
    pub value: &'a [u8],
}

/// Possible parser errors:
#[derive(Clone, Debug, PartialEq)]
pub enum ParserError<'a> {
    /// The parser encountered the end of the file during parsing of an entry.
    UnexpectedEof,

    /// The parser encountered something which was supposed to be an integer, but wasn't.
    ParseInt(core::num::ParseIntError),

    /// The parser encountered an integer which was out-of-range.
    FromInt(core::num::TryFromIntError),

    /// The parser encountered something which
    /// was supposed to be an UTF-8 string (usually an ASCII integer), but wasn't.
    Utf8(core::str::Utf8Error),

    /// The thin tree encountered a file which got overridden by a directory.
    FileOverridenWithDirectory(Filename<'a>),
}

impl<'a> From<core::num::ParseIntError> for ParserError<'a> {
    #[inline(always)]
    fn from(x: core::num::ParseIntError) -> Self {
        Self::ParseInt(x)
    }
}

impl<'a> From<core::num::TryFromIntError> for ParserError<'a> {
    #[inline(always)]
    fn from(x: core::num::TryFromIntError) -> Self {
        Self::FromInt(x)
    }
}

impl<'a> From<core::str::Utf8Error> for ParserError<'a> {
    #[inline(always)]
    fn from(x: core::str::Utf8Error) -> Self {
        Self::Utf8(x)
    }
}

/// Bytes belonging to a decompressed tar file
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct Bytes([u8]);

impl Bytes {
    /// Interpret an existing byte slice as a tar byte slice.
    #[inline(always)]
    pub const fn new(data: &[u8]) -> &Self {
        unsafe { &*(data as *const [u8] as *const Self) }
    }
}

fn try_parse_octal<'a, T: num_traits::Num>(s: &[u8]) -> Result<T, ParserError<'a>>
where
    T::FromStrRadixErr: Into<ParserError<'a>>,
{
    // FIXME: use from_ascii_radix once that is stabilized:
    // https://github.com/rust-lang/rust/issues/134821
    T::from_str_radix(str::from_utf8(truncate(s))?, 8).map_err(Into::into)
}

const BLOCK_SIZE: usize = 512;
const BLOCK_SIZE_2POW: u32 = 9;

impl<'a> Parser<'a> {
    fn next_intern(&mut self) -> Result<Option<File<'a>>, ParserError<'a>> {
        while self.input.len() >= BLOCK_SIZE {
            // `input` starts with a tar header, padded to 512 bytes (block size)
            let offset = self.offset;
            let (header, rest) = self.input.split_at(BLOCK_SIZE);

            // note that integers are usually encoded as octal numbers
            let name = truncate(&header[0..100]);
            if header.iter().take_while(|i| **i == 0).count() == BLOCK_SIZE {
                // EOF marker
                return Ok(None);
            }
            let mut metadata = FileMetadata::default();
            if let Ok(mode) = try_parse_octal::<u16>(&header[100..108]) {
                metadata.is_exec = mode & 0o111 != 0;
            }
            let size: usize = try_parse_octal::<u64>(&header[124..136])?.try_into()?;
            let _linkname = &header[157..257];
            let magic = &header[257..263];
            let _version = &header[263..265];
            let prefix = &header[345..500];

            // check if this is a supported file type
            let ret = match header[156] {
                0 | b'0' => {
                    // regular file
                    let value_offset = offset + BLOCK_SIZE;
                    Some(File {
                        name: Filename::One(name),
                        metadata,
                        value_range: value_offset..(value_offset + size),
                        value: rest.get(..size).ok_or(ParserError::UnexpectedEof)?,
                    })
                }
                _ => None,
            };

            // finish handling this record
            // header
            self.offset += BLOCK_SIZE;
            // rest (size rounded to next multiple of BLOCK_SIZE)
            let actual_rest_size = {
                let mut tmp = size >> BLOCK_SIZE_2POW;
                if !size.is_multiple_of(BLOCK_SIZE) {
                    tmp += 1;
                }
                tmp << BLOCK_SIZE_2POW
            };
            self.offset += actual_rest_size;
            self.input = rest
                .get(actual_rest_size..)
                .ok_or(ParserError::UnexpectedEof)?;

            if let Some(mut x) = ret {
                // gather full file name (we might have to honor the ustar prefix)
                if magic == b"ustar\0" && (prefix[0] != 0 || name.contains(&b'\\')) {
                    let prefix = truncate(prefix);
                    if !prefix.is_empty() {
                        x.name = Filename::Two(prefix, name);
                    }
                }
                return Ok(Some(x));
            }
        }

        if self.input.is_empty() {
            return Ok(None);
        }
        Err(ParserError::UnexpectedEof)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<File<'a>, ParserError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_intern() {
            Ok(None) => None,
            Ok(Some(x)) => Some(Ok(x)),
            Err(e) => {
                // make sure we don't get stuck
                self.input = &[];
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn doesnt_crash(data: Vec<u8>) {
            Parser::new(Bytes::new(&*data)).count();
        }
    }
}
