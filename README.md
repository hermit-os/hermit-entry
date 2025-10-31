# hermit-entry

[![Crates.io](https://img.shields.io/crates/v/hermit-entry)](https://crates.io/crates/hermit-entry)
[![docs.rs](https://img.shields.io/badge/docs.rs-documentation-green.svg)](https://docs.rs/hermit-entry/latest/hermit_entry/)
[![CI](https://github.com/hermitcore/hermit-entry/actions/workflows/ci.yml/badge.svg)](https://github.com/hermitcore/hermit-entry/actions/workflows/ci.yml)

Hermit's loading and entry API.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Hermit images

This Rust crate also implements a basic reader for Hermit images.
Overall, these are just `.tar.gz` (i.e. gzipped tar) files.

For performance reasons, it should be preferred to put the config and kernel
as the first two entries of the image (tar files don't have any sorting or index,
except that normally, the latest entry of the file takes precedence).
If this condition is not adhered to, some hermit / uhyve tools might refuse
to work with the image, or emit a warning.

They contain at least 2 special entries:
* The config file (in TOML format), at `hermit_config.toml` in the image root.
  The expected entries are described in the crate documentation in `hermit_entry::config::Config` (requires enabling the `config` feature).
* A Hermit Kernel ELF file, whose path is specified in the config.

The image itself is mapped (from the Hermit kernel perspective) into a path
(`mount_point`) specified in the config file.
