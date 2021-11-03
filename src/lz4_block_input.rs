use crate::common::{Checksum, ErrorCorruptedStream, Result};
use crate::compression::{Compression, Context};
use crate::lz4_block_header::{CompressionMethod, Lz4BlockHeader};

use std::cmp::min;
use std::io::Read;

/// Wrapper around a [`Read`] object to decompress data.
///
/// The data read from [`Lz4BlockInput`] is first read from the wrapped [`Read`], decompressed and then returned.
///
/// # Example
///
/// ```rust
/// use lz4jb::Lz4BlockInput;
/// use std::io::Read;
///
/// // &[u8] implements the Read trait
/// const D: [u8; 24] = [
///     76, 90, 52, 66, 108, 111, 99, 107, 16, 3, 0, 0, 0, 3, 0, 0, 0, 82, 228, 119, 6, 46, 46, 46,
/// ];
///
/// fn main() -> std::io::Result<()> {
///     let mut output = String::new();
///     Lz4BlockInput::new(&D[..]).read_to_string(&mut output)?;
///     println!("{}", output);
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Lz4BlockInput<R: Read + Sized, C: Compression> {
    reader: R,
    compression: C,
    compressed_buf: Vec<u8>,
    decompressed_buf: Vec<u8>,
    read_ptr: usize,
    checksum: Checksum,
    stop_on_empty_block: bool,
}

impl<R: Read> Lz4BlockInput<R, Context> {
    /// Create a new [`Lz4BlockInput`] with the default [`Compression`] implementation.
    ///
    /// See [`Self::with_context()`]
    pub fn new(r: R) -> Self {
        Self::with_context(r, Context::default())
    }
}

impl<R: Read, C: Compression> Lz4BlockInput<R, C> {
    /// Create a new [`Lz4BlockInput`] with the default checksum implementation which matches the Java's default implementation.
    ///
    ///
    ///
    /// See [`Self::with_checksum()`]
    pub fn with_context(r: R, c: C) -> Self {
        Self::with_checksum(r, c, Lz4BlockHeader::default_checksum, true)
    }

    /// Create a new [`Lz4BlockInput`].
    ///
    /// The checksum must return a [`u32`].
    pub fn with_checksum(
        r: R,
        c: C,
        checksum: fn(&[u8]) -> u32,
        stop_on_empty_block: bool,
    ) -> Self {
        Self {
            reader: r,
            compression: c,
            compressed_buf: Vec::new(),
            decompressed_buf: Vec::new(),
            read_ptr: 0,
            checksum: Checksum::new(checksum),
            stop_on_empty_block,
        }
    }

    fn read_header(&mut self) -> std::io::Result<Option<Lz4BlockHeader>> {
        Ok(loop {
            match Lz4BlockHeader::read(&mut self.reader)? {
                None => break None,
                Some(h) => {
                    if h.decompressed_len > 0 {
                        break Some(h);
                    } else if self.stop_on_empty_block {
                        break None;
                    }
                }
            };
        })
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.read_ptr == self.decompressed_buf.len() {
            let header = match self.read_header()? {
                None => return Ok(0),
                Some(h) => h,
            };

            ensure_vec(
                &mut self.decompressed_buf,
                header.compression_level.get_max_decompressed_buffer_len(),
                header.decompressed_len,
            );

            match header.compression_method {
                CompressionMethod::Raw => self.reader.read_exact(self.decompressed_buf.as_mut())?,
                CompressionMethod::LZ4 => {
                    ensure_vec(
                        &mut self.compressed_buf,
                        self.compression.get_maximum_compressed_buffer_len(
                            header.compression_level.get_max_decompressed_buffer_len(),
                        ),
                        header.compressed_len,
                    );
                    self.reader.read_exact(self.compressed_buf.as_mut())?;
                    match self
                        .compression
                        .decompress(self.compressed_buf.as_ref(), self.decompressed_buf.as_mut())
                    {
                        Ok(s) => {
                            if s != self.decompressed_buf.len() {
                                return ErrorCorruptedStream::new_error();
                            }
                        }
                        Err(err) => {
                            return Err(err.into());
                        }
                    };
                }
            }
            if self.checksum.run(self.decompressed_buf.as_ref()) != header.checksum {
                return ErrorCorruptedStream::new_error();
            }
            self.read_ptr = 0;
        }

        let size_to_copy = min(buf.len(), self.decompressed_buf.len() - self.read_ptr);
        buf[..size_to_copy]
            .copy_from_slice(&self.decompressed_buf[self.read_ptr..self.read_ptr + size_to_copy]);
        self.read_ptr += size_to_copy;
        Ok(size_to_copy)
    }
}

fn ensure_vec(v: &mut Vec<u8>, max_block_size: usize, desired_len: u32) {
    let max_block_size = max_block_size;
    if v.capacity() < max_block_size {
        v.reserve(max_block_size - v.len())
    }
    v.resize_with(desired_len as usize, u8::default);
}

impl<R: Read, C: Compression> Read for Lz4BlockInput<R, C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(Lz4BlockInput::read(self, buf)?)
    }
}

#[cfg(test)]
mod test_lz4_block_input {
    use super::Lz4BlockInput;
    use crate::compression::Context;
    use crate::lz4_block_header::data::{VALID_DATA, VALID_EMPTY};

    use std::io::Read;

    #[test]
    fn read_empty() {
        let mut out = Vec::<u8>::new();
        Lz4BlockInput::new(&VALID_EMPTY[..])
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, []);
    }

    #[test]
    fn read_basic() {
        let mut out = Vec::<u8>::new();
        Lz4BlockInput::new(&VALID_DATA[..])
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, "...".as_bytes());
    }

    #[test]
    fn read_with_checksum_invalid() {
        let mut out = Vec::<u8>::new();
        assert!(Lz4BlockInput::with_checksum(
            &VALID_DATA[..],
            Context::default(),
            |_| 0x12345678,
            true
        )
        .read_to_end(&mut out)
        .is_err());
    }

    #[test]
    fn read_with_checksum_valid() {
        let mut out = Vec::<u8>::new();
        Lz4BlockInput::with_checksum(&VALID_DATA[..], Context::default(), |_| 0x0677e452, true)
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, "...".as_bytes());
    }

    #[test]
    fn read_with_empty_block_stop() {
        let mut input = VALID_EMPTY.to_vec();
        input.extend_from_slice(&[0; 21]);

        let mut out = Vec::<u8>::new();
        Lz4BlockInput::with_checksum(&input[..], Context::default(), |_| 0x0677e452, true)
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, "".as_bytes());
    }

    #[test]
    fn read_with_empty_block_no_stop() {
        let mut input = VALID_EMPTY.to_vec();
        input.extend_from_slice(&VALID_EMPTY);
        input.extend_from_slice(&VALID_EMPTY);
        input.extend_from_slice(&VALID_EMPTY);

        let mut out = Vec::<u8>::new();
        Lz4BlockInput::with_checksum(&input[..], Context::default(), |_| 0x0677e452, false)
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, "".as_bytes());
    }

    #[test]
    fn read_with_empty_block_no_stop_with_error() {
        let mut input = VALID_EMPTY.to_vec();
        input.extend_from_slice(&VALID_EMPTY);
        input.extend_from_slice(&VALID_EMPTY);
        input.extend_from_slice(&VALID_EMPTY);
        input.extend_from_slice(&[0; 21]);

        let mut out = Vec::<u8>::new();
        assert!(Lz4BlockInput::with_checksum(
            &input[..],
            Context::default(),
            |_| 0x0677e452,
            false
        )
        .read_to_end(&mut out)
        .is_err());
    }
}
