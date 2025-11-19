//! A thin (directory) tree (i.e. it doesn't own the data it points to) implementation.

use alloc::collections::btree_map::BTreeMap;
use core::fmt;

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

/// An error from [`ThinTree::resolve_to_leaf`] or [`ThinTree::resolve_mut_to_leaf`],
/// to distinguish between "entry couldn't be found" and "entry isn't a leaf".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveToLeafError {
    /// The entry couldn't be found.
    NotFound,
    /// The entry was found, but is a directory instead of a regular file / leaf.
    IsDirectory,
}

impl fmt::Display for ResolveToLeafError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "file node not found"),
            Self::IsDirectory => write!(f, "expected file, got directory"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ResolveToLeafError {}

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

    /// Resolve a file name in a thin tree and making sure it is a leaf / file node
    pub fn resolve_to_leaf(
        &self,
        entry: Filename<'_>,
    ) -> Result<(&'a [u8], &FileMetadata), ResolveToLeafError> {
        if let Self::File { content, metadata } =
            self.resolve(entry).ok_or(ResolveToLeafError::NotFound)?
        {
            Ok((*content, metadata))
        } else {
            Err(ResolveToLeafError::IsDirectory)
        }
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

    /// Resolve a file name in a thin tree and making sure it is a leaf / file node (mutable version)
    pub fn resolve_mut_to_leaf(
        &mut self,
        entry: Filename<'_>,
    ) -> Result<(&mut &'a [u8], &mut FileMetadata), ResolveToLeafError> {
        if let Self::File { content, metadata } = self
            .resolve_mut(entry)
            .ok_or(ResolveToLeafError::NotFound)?
        {
            Ok((content, metadata))
        } else {
            Err(ResolveToLeafError::IsDirectory)
        }
    }
}
