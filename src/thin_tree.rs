//! A thin (directory) tree (i.e. it doesn't own the data it points to) implementation.

use alloc::collections::btree_map::BTreeMap;

use crate::Filename;
use crate::tar_parser::{Bytes, FileMetadata, Parser, ParserError};

/// A thin (directory) tree (i.e. it doesn't own the data it points to).
#[derive(Clone, Debug, PartialEq, Eq, yoke::Yokeable)]
pub enum ThinTree<'a> {
    /// A regular file
    #[allow(missing_docs)]
    File {
        content: &'a [u8],
        metadata: FileMetadata,
    },
    /// A (sub-)directory
    Directory(BTreeMap<&'a [u8], ThinTree<'a>>),
}

impl<'a> ThinTree<'a> {
    /// Populate a thin directory tree, with `entry` pointing to `content`
    fn update(
        &mut self,
        entry: Filename<'a>,
        content: &'a [u8],
        metadata: FileMetadata,
    ) -> Result<(), ParserError<'a>> {
        let mut this = self;
        for (n, i) in entry.enumerate() {
            let dir = match this {
                Self::File { content: [], .. } => {
                    *this = Self::Directory(BTreeMap::new());
                    if let Self::Directory(dir) = this {
                        dir
                    } else {
                        unreachable!()
                    }
                }
                Self::File { .. } => {
                    return Err(ParserError::FileOverridenWithDirectory(
                        entry.as_truncated(n),
                    ));
                }
                Self::Directory(dir) => dir,
            };
            this = dir.entry(i).or_insert(Self::File {
                content: &[],
                metadata: Default::default(),
            });
        }
        *this = Self::File { content, metadata };
        Ok(())
    }

    /// Populate a thin directory tree from a tar archive (uncompressed)
    pub fn try_from_image(image: &'a Bytes) -> Result<Self, ParserError<'a>> {
        let mut content = Self::File {
            content: &[],
            metadata: Default::default(),
        };
        for i in Parser::new(image) {
            let i = i?;
            // multiple entries with the same name might exist,
            // latest entry wins / overwrites existing ones
            content.update(i.name, i.value, i.metadata)?;
        }
        Ok(content)
    }

    /// Resolve a file name in a thin tree
    pub fn resolve(&self, mut entry: Filename<'_>) -> Option<&Self> {
        entry.try_fold(self, move |this, i| {
            if let Self::Directory(dir) = this {
                dir.get(i)
            } else {
                None
            }
        })
    }

    /// Resolve a file name in a thin tree (mutable version)
    pub fn resolve_mut(&mut self, mut entry: Filename<'_>) -> Option<&mut Self> {
        entry.try_fold(self, move |this, i| {
            if let Self::Directory(dir) = this {
                dir.get_mut(i)
            } else {
                None
            }
        })
    }
}
