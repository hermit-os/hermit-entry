use core::{cmp, fmt};

const SEP: u8 = b'/';

/// Truncate a byte slice to the first NULL byte. if any.
// taken from `tar-rs`
pub fn truncate(slice: &[u8]) -> &[u8] {
    match slice.iter().position(|i| *i == 0) {
        Some(i) => &slice[..i],
        None => slice,
    }
}

// FIXME: once `slice`.split_once is stable, we can get rid of this.
fn split_once(this: &[u8], on: u8) -> Option<(&[u8], &[u8])> {
    let index = this.iter().position(|i| *i == on)?;
    Some((&this[..index], &this[index + 1..]))
}

/// Zero-copy file name (usually from an in-memory tar file)
///
/// Code might rely on the invariant that it doesn't contain null bytes.
#[derive(Clone, Copy, Debug, Eq)]
pub enum Filename<'a> {
    /// A simple file name
    One(&'a [u8]),
    /// A prefix and file name (meaning `{0}/{1}`)
    Two(&'a [u8], &'a [u8]),
}

impl Filename<'_> {
    /// Truncate the filename to the first `n` components
    #[must_use = "input is not modified"]
    pub fn as_truncated(self, mut n: usize) -> Self {
        let handle_parts = |n: &mut usize, x: &mut &[u8]| {
            if *n > 0 {
                let mut offset = 0;
                for i in (*x).split(|i| *i == SEP) {
                    *n -= 1;
                    offset += i.len();
                    if *n == 0 {
                        // truncate
                        *x = &x[..offset];
                        break;
                    }
                    // separator
                    offset += 1;
                }
            }
        };

        match self {
            Self::One(_) if n == 0 => Self::One(&[]),
            Self::One(mut x) => {
                handle_parts(&mut n, &mut x);
                Self::One(x)
            }
            Self::Two(mut x, mut y) => {
                handle_parts(&mut n, &mut x);
                if n >= 1 {
                    handle_parts(&mut n, &mut y);
                    Self::Two(x, y)
                } else {
                    Self::One(x)
                }
            }
        }
    }
}

impl fmt::Display for Filename<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One(x) => write!(f, "{:?}", x),
            Self::Two(x, y) => write!(f, "{:?}/{:?}", x, y),
        }
    }
}

impl cmp::PartialEq for Filename<'_> {
    fn eq(&self, oth: &Self) -> bool {
        let (mut this, mut oth) = (*self, *oth);
        loop {
            match (this.next(), oth.next()) {
                (None, None) => break true,
                (None, _) | (_, None) => break false,
                (Some(x), Some(y)) if x != y => break false,
                _ => {}
            }
        }
    }
}

impl<'a> From<&'a [u8]> for Filename<'a> {
    #[inline]
    fn from(x: &'a [u8]) -> Self {
        // null bytes are the only illegal bytes in a filename
        assert!(!x.contains(&0x00));
        Self::One(x)
    }
}

impl<'a> From<&'a str> for Filename<'a> {
    #[inline]
    fn from(x: &'a str) -> Self {
        // null bytes are the only illegal bytes in a str filename
        assert!(!x.contains('\0'));
        Self::One(x.as_bytes())
    }
}

impl<'a> Iterator for Filename<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        match self {
            Self::One([]) => None,
            Self::One(x) => Some(match split_once(x, SEP) {
                None => {
                    let ret = *x;
                    *x = &[];
                    ret
                }
                Some((fi, rest)) => {
                    *x = rest;
                    fi
                }
            }),
            Self::Two(x, y) => Some(match split_once(x, SEP) {
                None => {
                    let ret = *x;
                    *self = Self::One(y);
                    ret
                }
                Some((fi, rest)) => {
                    *x = rest;
                    fi
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_iteq<'a, I: Iterator<Item = &'a [u8]>>(mut it: I, eqto: &[&str]) {
        for i in eqto {
            assert_eq!(it.next(), Some(i.as_bytes()), "divergence @ {}", i);
        }
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_filename_iter_one() {
        let it = Filename::One(b"aleph/beta/omicron");
        assert_iteq(it, &["aleph", "beta", "omicron"]);
    }

    #[test]
    fn test_filename_as_truncated_one() {
        let it = Filename::One(b"aleph/beta/omicron").as_truncated(2);
        assert_iteq(it, &["aleph", "beta"]);
    }

    #[test]
    fn test_filename_iter_two() {
        let it = Filename::Two(b"aleph/beta", b"omicron/depth");
        assert_iteq(it, &["aleph", "beta", "omicron", "depth"]);
    }

    #[test]
    fn test_filename_as_truncated_two() {
        let mfn = Filename::Two(b"aleph/beta", b"omicron/depth");
        assert_iteq(mfn.as_truncated(2), &["aleph", "beta"]);
        assert_iteq(mfn.as_truncated(3), &["aleph", "beta", "omicron"]);
        assert_iteq(mfn.as_truncated(4), &["aleph", "beta", "omicron", "depth"]);
        assert_iteq(mfn.as_truncated(5), &["aleph", "beta", "omicron", "depth"]);
    }

    #[test]
    fn test_filename_non_ascii() {
        let mfn = Filename::Two("αleph/βetα".as_bytes(), "ο/δth".as_bytes());
        assert_iteq(mfn, &["αleph", "βetα", "ο", "δth"]);
        assert_iteq(mfn.as_truncated(2), &["αleph", "βetα"]);
        assert_iteq(mfn.as_truncated(3), &["αleph", "βetα", "ο"]);
        assert_iteq(mfn.as_truncated(4), &["αleph", "βetα", "ο", "δth"]);
        assert_iteq(mfn.as_truncated(5), &["αleph", "βetα", "ο", "δth"]);
    }
}
