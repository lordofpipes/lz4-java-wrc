#![warn(missing_docs)]

//! A Rust implementation of the [LZ4BlockOutputStream] format from [lz4-java].
//!
//! **Beware**: this format is not compatible with the standard [LZ4 Block format].
//! You should not use it if you unless you have to work with some historical data compressed using the Java code.
//!
//! [LZ4BlockOutputStream]: https://github.com/lz4/lz4-java/blob/1.8.0/src/java/net/jpountz/lz4/LZ4BlockOutputStream.java
//! [lz4-java]: https://github.com/lz4/lz4-java
//! [LZ4 Block format]: https://github.com/lz4/lz4/blob/dev/doc/lz4_Block_format.md
//!
//! # Example
//!
//! ```rust
//! use lz4jb::{Lz4BlockInput, Lz4BlockOutput};
//! use std::io::{Read, Result, Write};
//!
//! fn compress(d: &str) -> Result<Vec<u8>> {
//!     let mut compressed = Vec::new();
//!     Lz4BlockOutput::new(&mut compressed).write_all(d.as_bytes())?;
//!     Ok(compressed)
//! }
//! fn decompress(r: &[u8]) -> Result<String> {
//!     let mut decompressed = String::new();
//!     Lz4BlockInput::new(r).read_to_string(&mut decompressed)?;
//!     Ok(decompressed)
//! }
//!
//! fn main() -> Result<()> {
//!     // compress the string
//!     let compressed = compress("Hello World!")?;
//!
//!     // decompress back into the original value
//!     let decompressed = decompress(compressed.as_slice())?;
//!     println!("{}", decompressed);
//!     Ok(())
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `use_lz4_flex`: use `lz4_flex` as lz4 compression library (enabled by default)
//! - `use_lz4-sys`: use `lz4-sys` as lz4 compression library (disabled by default)
//!
//! When compiling with one of the lz4 compression library, it is used by default.
//! When compiling with both of them, one can choose with the [`Context`] enum.

mod common;
mod compression;
mod lz4_block_header;
mod lz4_block_input;
mod lz4_block_output;

pub use compression::{Compression, Context};
pub use lz4_block_input::{Lz4BlockInput, Lz4BlockInputBase};
pub use lz4_block_output::{Lz4BlockOutput, Lz4BlockOutputBase};
