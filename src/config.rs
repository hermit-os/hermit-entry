//! The image `hermit.toml` config file format.
//!
//! All file paths are relative to the image root.

use alloc::string::String;
use alloc::vec::Vec;

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
