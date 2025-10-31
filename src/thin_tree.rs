use alloc::collections::btree_map::BTreeMap;

use crate::tar_parser::{Parser, ParserError};
use crate::StrFilename;

#[derive(Clone, Debug, PartialEq, Eq, yoke::Yokeable)]
pub enum ThinTreeRef<'a> {
    File(&'a [u8]),
    Directory(BTreeMap<&'a str, ThinTreeRef<'a>>),
}

impl<'a> ThinTreeRef<'a> {
    /// Populate a thin directory tree, with `entry` pointing to `r`
    pub fn update(&mut self, entry: StrFilename<'a>, r: &'a [u8]) -> Result<(), ParserError<'a>> {
        let mut this = self;
        for (n, i) in entry.enumerate() {
            let dir = match this {
                Self::File([]) => {
                    *this = Self::Directory(BTreeMap::new());
                    if let Self::Directory(dir) = this {
                        dir
                    } else {
                        unreachable!()
                    }
                }
                Self::File(_) => {
                    return Err(ParserError::FileOverridenWithDirectory(entry.truncate(n)));
                }
                Self::Directory(dir) => dir,
            };
            this = dir.entry(i).or_insert(Self::File(b""));
        }
        *this = Self::File(r);
        Ok(())
    }

    pub fn try_from_image(image: &'a [u8]) -> Result<Self, ParserError<'a>> {
        let mut content = Self::File(b"");
        for i in Parser::new(image) {
            let i = i?;
            let name = i.name.try_as_str().ok_or(ParserError::Utf8Opaque)?;
            // multiple entries with the same name might exist,
            // latest entry wins / overwrites existing ones
            content.update(name, i.value)?;
        }
        Ok(content)
    }

    pub fn resolve(&self, mut entry: StrFilename<'_>) -> Option<&Self> {
        entry.try_fold(self, move |this, i| {
            if let Self::Directory(dir) = this {
                dir.get(i)
            } else {
                None
            }
        })
    }

    pub fn resolve_mut(&mut self, mut entry: StrFilename<'_>) -> Option<&mut Self> {
        entry.try_fold(self, move |this, i| {
            if let Self::Directory(dir) = this {
                dir.get_mut(i)
            } else {
                None
            }
        })
    }
}
