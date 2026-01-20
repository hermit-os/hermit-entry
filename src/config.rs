//! The image `hermit.toml` config file format.
//!
//! All file paths are relative to the image root.

use alloc::borrow::Cow;
use alloc::vec::Vec;
#[cfg(feature = "loader")]
use core::fmt;

/// The default configuration file name, relative to the image root.
pub const DEFAULT_CONFIG_NAME: &str = "hermit.toml";

/// The possible errors which the parser might emit.
pub type ParserError = toml::de::Error;

pub use tar_no_std;

/// The configuration toplevel structure.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Input<'a> {
    /// Arguments to be passed to the kernel
    #[serde(borrow)]
    pub kernel_args: Vec<Cow<'a, str>>,

    /// Arguments to be passed to the application
    #[serde(borrow)]
    pub app_args: Vec<Cow<'a, str>>,

    /// Environment variables
    #[serde(borrow)]
    pub env_vars: Vec<Cow<'a, str>>,
}

/// Minimal requirements for an image to be able to run as expected
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Requirements {
    /// Minimum RAM
    pub memory: Option<byte_unit::Byte>,

    /// Minimum amount of CPUs
    #[serde(default)]
    pub cpus: u32,
}

/// Parse a config file from a byte slice.
#[inline]
pub fn parse(data: &[u8]) -> Result<Config<'_>, ParserError> {
    toml::from_slice(data)
}

/// An error from [`handle_config`].
#[cfg(feature = "loader")]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum HandleConfigError {
    /// The Hermit image tar is corrupt.
    TarCorrupt,

    /// [`DEFAULT_CONFIG_NAME`]
    /// either couldn't be found in the image or isn't a regular file.
    ConfigResolve,

    /// The Hermit image configuration file failed to parse.
    ConfigParserError(ParserError),

    /// The Kernel specified in the image configuration file
    /// either couldn't be found in the image or isn't a regular file.
    KernelResolve,
}

#[cfg(feature = "loader")]
impl fmt::Display for HandleConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TarCorrupt => f.write_str("tar file in Hermit image is corrupt"),
            Self::ConfigResolve => {
                write!(f, "couldn't find Hermit image configuration file")
            }
            Self::ConfigParserError(e) => write!(f, "Hermit image configuration is invalid: {e}"),
            Self::KernelResolve => write!(f, "couldn't find Hermit kernel in image"),
        }
    }
}

#[cfg(feature = "loader")]
impl core::error::Error for HandleConfigError {}

/// A convenience function to handle looking up the config
/// in a tar file (decompressed) and retrieve the kernel slice.
#[cfg(feature = "loader")]
pub fn handle_config(image: &[u8]) -> Result<(Config<'_>, &[u8]), HandleConfigError> {
    let taref = tar_no_std::TarArchiveRef::new(image).map_err(|_| HandleConfigError::TarCorrupt)?;

    fn lookup_in_image<'a>(
        taref: &tar_no_std::TarArchiveRef<'a>,
        f: &str,
    ) -> Result<Option<&'a [u8]>, HandleConfigError> {
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

    let config_slice =
        lookup_in_image(&taref, DEFAULT_CONFIG_NAME)?.ok_or(HandleConfigError::ConfigResolve)?;
    let config = parse(config_slice).map_err(HandleConfigError::ConfigParserError)?;

    let kernel_slice = match &config {
        Config::V1 { kernel, .. } => {
            lookup_in_image(&taref, kernel)?.ok_or(HandleConfigError::KernelResolve)?
        }
    };

    Ok((config, kernel_slice))
}
