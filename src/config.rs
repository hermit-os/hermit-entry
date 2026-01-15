//! The image `hermit.toml` config file format.
//!
//! All file paths are relative to the image root.

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::fmt;

/// The default configuration file name, relative to the image root.
const DEFAULT_CONFIG_NAME: &str = "hermit.toml";

/// The possible errors which the parser might emit.
type ParserError = toml::de::Error;

/// The configuration toplevel structure.
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(tag = "version")]
pub enum Config<'a> {
    /// The first (and current) version of the config format.
    #[serde(rename = "1")]
    V1 {
        /// Input parameter for the kernel and application
        #[serde(borrow)]
        input: Input<'a>,

        /// Minimal requirements for an image to be able to run as expected
        #[serde(default)]
        requirements: Requirements,

        /// Kernel ELF file path
        #[serde(borrow)]
        kernel: Cow<'a, str>,
    },
}

/// Input parameter for the kernel and application
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct Input<'a> {
    /// Arguments to be passed to the kernel
    #[serde(borrow)]
    pub kernel_args: Vec<Cow<'a, str>>,

    /// Arguments to be passed to the application
    #[serde(borrow)]
    pub app_args: Vec<Cow<'a, str>>,

    /// Environment variables
    #[serde(borrow, default)]
    pub env_vars: Vec<Cow<'a, str>>,
}

/// Minimal requirements for an image to be able to run as expected
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
pub struct Requirements {
    /// Minimum RAM
    pub memory: Option<byte_unit::Byte>,

    /// Minimum amount of CPUs
    #[serde(default)]
    pub cpus: u32,
}

/// An error from [`parse_tar`].
#[derive(Clone, Debug)]
pub struct ParseTarError(ParseTarErrorInner);

impl From<ParseTarErrorInner> for ParseTarError {
    #[inline]
    fn from(x: ParseTarErrorInner) -> Self {
        Self(x)
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
enum ParseTarErrorInner {
    /// The Hermit image tar is corrupt.
    TarCorrupt,

    /// [`DEFAULT_CONFIG_NAME`]
    /// either couldn't be found in the image or isn't a regular file.
    ConfigResolve,

    /// The Hermit image configuration file failed to parse due to being non-UTF8.
    ConfigUtf8Error(core::str::Utf8Error),

    /// The Hermit image configuration file failed to parse due to being invalid TOML.
    ConfigTomlParseError(ParserError),

    /// The Kernel specified in the image configuration file
    /// either couldn't be found in the image or isn't a regular file.
    KernelResolve,
}

impl fmt::Display for ParseTarErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TarCorrupt => f.write_str("tar file in Hermit image is corrupt"),
            Self::ConfigResolve => {
                write!(f, "couldn't find Hermit image configuration file")
            }
            Self::ConfigUtf8Error(e) => write!(f, "Hermit image configuration is invalid: {e}"),
            Self::ConfigTomlParseError(e) => {
                write!(f, "Hermit image configuration is invalid: {e}")
            }
            Self::KernelResolve => write!(f, "couldn't find Hermit kernel in image"),
        }
    }
}

impl fmt::Display for ParseTarError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl core::error::Error for ParseTarErrorInner {}
impl core::error::Error for ParseTarError {}

/// Parsed data from an image
pub struct ConfigHandle<'a> {
    /// The image configuration
    pub config: Config<'a>,

    /// The raw kernel ELF slice
    pub raw_kernel: &'a [u8],
}

/// A convenience function to handle looking up the config
/// in a tar file (decompressed) and retrieve the kernel slice.
pub fn parse_tar(image: &[u8]) -> Result<ConfigHandle<'_>, ParseTarError> {
    use ParseTarErrorInner as Error;

    let taref = tar_no_std::TarArchiveRef::new(image).map_err(|_| Error::TarCorrupt)?;

    fn lookup_in_image<'a>(
        taref: &tar_no_std::TarArchiveRef<'a>,
        f: &str,
    ) -> Result<Option<&'a [u8]>, Error> {
        let f = {
            let f = f.as_bytes();
            let mut tmp = [0u8; 256];
            if f.len() >= 256 {
                return Ok(None);
            }
            tmp.copy_from_slice(f);
            tar_no_std::TarFormatString::new(tmp)
        };

        let mut ret = None;
        for i in taref.entries() {
            // multiple entries with the same name might exist,
            // latest entry wins / overwrites existing ones
            if i.filename() == f {
                ret = Some(i.data());
            }
        }

        Ok(ret)
    }

    let config_slice = lookup_in_image(&taref, DEFAULT_CONFIG_NAME)?.ok_or(Error::ConfigResolve)?;
    let config_slice = core::str::from_utf8(config_slice).map_err(Error::ConfigUtf8Error)?;
    let config: Config<'_> = toml::from_str(config_slice).map_err(Error::ConfigTomlParseError)?;

    let kernel_name: &str = match &config {
        Config::V1 { kernel, .. } => kernel,
    };

    let raw_kernel = lookup_in_image(&taref, kernel_name)?.ok_or(Error::KernelResolve)?;

    Ok(ConfigHandle { config, raw_kernel })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parsing() {
        let dat = r#"
version = "1"
kernel = "/kernel.elf"

[input]
kernel_args = []
app_args = []

[requirements]
"#;
        let parsed: super::Config = toml::from_str(dat).unwrap();
        use alloc::vec;
        assert_eq!(
            parsed,
            super::Config::V1 {
                kernel: "/kernel.elf".into(),
                input: super::Input {
                    kernel_args: vec![],
                    app_args: vec![],
                    env_vars: vec![],
                },
                requirements: super::Requirements {
                    memory: None,
                    cpus: 0,
                },
            }
        );
    }
}
