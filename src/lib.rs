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
//!     Lz4BlockOutput::new(&mut compressed, 64)?.write_all(d.as_bytes())?;
//!     Ok(compressed)
//! }
//! fn decompress(r: &mut dyn Read) -> Result<String> {
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
//!     let decompressed = decompress(&mut compressed.as_slice())?;
//!     println!("{}", decompressed);
//!     Ok(())
//! }
//! ```

mod common;
mod lz4_block_header;
mod lz4_block_input;
mod lz4_block_output;

pub use lz4_block_input::Lz4BlockInput;
pub use lz4_block_output::Lz4BlockOutput;
