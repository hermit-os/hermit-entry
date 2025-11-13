//! The image `hermit.toml` config file format.
//!
//! All file paths are relative to the image root.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::ThinTree;

/// The default configuration file name, relative to the image root.
pub const DEFAULT_CONFIG_NAME: &str = "hermit.toml";

/// The possible errors which the parser might emit.
pub type ParserError = toml::de::Error;

/// The configuration toplevl structure.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "version")]
pub enum Config {
    /// The first (and current) version of the config format.
    #[serde(rename = "1")]
    V1 {
        /// Input parameter for the kernel and application
        input: Input,

        /// Minimal requirements for an image to be able to run as expected
        #[serde(default)]
        requirements: Requirements,

        /// Kernel ELF file path
        kernel: String,
    },
}

/// Input parameter for the kernel and application
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Input {
    /// Arguments to be passed to the kernel
    pub kernel_args: Vec<String>,

    /// Arguments to be passed to the application
    pub app_args: Vec<String>,

    /// Environment variables
    pub env_vars: Vec<String>,
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
pub fn parse(data: &[u8]) -> Result<Config, ParserError> {
    toml::from_slice(data)
}

/// An error from [`ThinTree::handle_config`].
#[derive(Clone, Debug)]
pub enum HandleConfigError {
    /// [`DEFAULT_CONFIG_NAME`]
    /// either couldn't be found in the image or isn't a regular file.
    ConfigResolve(crate::ResolveToLeafError),
    /// The Hermit image configuration file failed to parse.
    ConfigParserError(ParserError),
    /// The Kernel specified in the image configuration file
    /// either couldn't be found in the image or isn't a regular file.
    KernelResolve(crate::ResolveToLeafError),
}

impl fmt::Display for HandleConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigResolve(e) => {
                write!(f, "couldn't find Hermit image configuration file: {e}")
            }
            Self::ConfigParserError(e) => write!(f, "Hermit image configuration is invalid: {e}"),
            Self::KernelResolve(e) => write!(f, "couldn't find Hermit kernel in image: {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HandleConfigError {}

impl<'a> ThinTree<'a> {
    /// A convenience function to handle looking up the config in a thin tree and
    /// retrieve the kernel slice.
    pub fn handle_config(&self) -> Result<(Config, &'a [u8]), HandleConfigError> {
        let config = parse(
            self.resolve_to_leaf(DEFAULT_CONFIG_NAME.into())
                .map_err(HandleConfigError::ConfigResolve)?
                .0,
        )
        .map_err(HandleConfigError::ConfigParserError)?;

        let kernel_slice = match &config {
            Config::V1 { kernel, .. } => {
                self.resolve_to_leaf((**kernel).into())
                    .map_err(HandleConfigError::KernelResolve)?
                    .0
            }
        };

        Ok((config, kernel_slice))
    }
}
