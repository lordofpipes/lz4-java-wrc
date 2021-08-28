use crate::common::{Checksum, ErrorInternal, Result};
use crate::lz4_block_header::{CompressionLevel, CompressionMethod, Lz4BlockHeader};

use lz4_flex::block::compress_into as lz4_compress;

use std::cmp::min;
use std::io::Write;

/// Wrapper around a [`Write`] object to compress data.
///
/// The data written to [`Lz4BlockOutput`] is be compressed and then written to the wrapped [`Write`].
///
/// # Example
///
/// ```rust
/// use lz4jb::Lz4BlockOutput;
/// use std::io::Write;
///
/// fn main() -> std::io::Result<()> {
///     let mut output = Vec::new(); // Vec<u8> implements the Write trait
///     Lz4BlockOutput::new(&mut output, 64)?.write_all("...".as_bytes())?;
///     println!("{:?}", output);
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Lz4BlockOutput<W: Write + Sized> {
    writer: W,
    compression_level: CompressionLevel,
    write_ptr: usize,
    decompressed_buf: Vec<u8>,
    compressed_buf: Vec<u8>,
    checksum: Checksum,
}
impl<W: Write> Lz4BlockOutput<W> {
    /// Create a new [`Lz4BlockOutput`] with the default checksum implementation which matches the Java's default implementation.
    ///
    /// See [`Self::with_checksum()`]
    pub fn new(w: W, block_size: usize) -> std::io::Result<Self> {
        Self::with_checksum(w, block_size, Lz4BlockHeader::default_checksum)
    }

    /// Create a new [`Lz4BlockOutput`].
    ///
    /// The `block_size` must be between `64` and `33554432` bytes.
    /// The checksum must return a [`u32`].
    ///
    /// # Errors
    ///
    /// It will return an error if the `block_size` is out of range
    pub fn with_checksum(
        w: W,
        block_size: usize,
        checksum: fn(&[u8]) -> u32,
    ) -> std::io::Result<Self> {
        let compression_level = CompressionLevel::from_block_size(block_size)?;
        let compressed_buf_len = compression_level.get_max_compressed_buffer_len();
        Ok(Self {
            writer: w,
            compression_level: compression_level,
            write_ptr: 0,
            compressed_buf: vec![0u8; compressed_buf_len],
            decompressed_buf: vec![0u8; block_size],
            checksum: Checksum::new(checksum),
        })
    }

    fn copy_to_buf(&mut self, buf: &[u8]) -> Result<usize> {
        let buf_into = &mut self.decompressed_buf[self.write_ptr..];
        if buf.len() > buf_into.len() {
            return Err(ErrorInternal::new(
                "Attempt to write a bigger buffer than the available one",
            ));
        }

        buf_into[..buf.len()].copy_from_slice(buf);
        self.write_ptr += buf.len();

        Ok(buf.len())
    }

    fn remaining_buf_len(&self) -> Result<usize> {
        if self.write_ptr <= self.decompressed_buf.len() {
            Ok(self.decompressed_buf.len() - self.write_ptr)
        } else {
            Err(ErrorInternal::new("Could not determine the buffer size"))
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.write_ptr == self.decompressed_buf.len() {
            self.flush()?;
        }
        let size_to_copy = min(buf.len(), self.remaining_buf_len()?);
        self.copy_to_buf(&buf[..size_to_copy])
    }

    fn flush(&mut self) -> Result<()> {
        if self.write_ptr > 0 {
            let decompressed_buf = &self.decompressed_buf[..self.write_ptr];
            let compressed_buf =
                match lz4_compress(decompressed_buf, self.compressed_buf.as_mut(), 0) {
                    Ok(s) => &self.compressed_buf[..s],
                    Err(err) => return Err(err.into()),
                };
            let (compression_method, buf_to_write) =
                if compressed_buf.len() < decompressed_buf.len() {
                    (CompressionMethod::LZ4, compressed_buf)
                } else {
                    (CompressionMethod::RAW, decompressed_buf)
                };
            Lz4BlockHeader {
                compression_method: compression_method,
                compression_level: self.compression_level.clone(),
                compressed_len: buf_to_write.len() as u32,
                decompressed_len: decompressed_buf.len() as u32,
                checksum: self.checksum.run(decompressed_buf),
            }
            .write(&mut self.writer)?;
            self.writer.write_all(buf_to_write)?;
        }
        self.write_ptr = 0;
        self.writer.flush()?;
        Ok(())
    }
}

impl<W: Write> Write for Lz4BlockOutput<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(Lz4BlockOutput::write(self, buf)?)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(Lz4BlockOutput::flush(self)?)
    }
}

impl<W: Write> Drop for Lz4BlockOutput<W> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
mod test_lz4_block_output {
    use super::Lz4BlockOutput;
    use crate::lz4_block_header::data::VALID_DATA;

    use std::io::Write;

    #[test]
    fn write_empty() {
        let mut out = Vec::<u8>::new();
        Lz4BlockOutput::new(&mut out, 128).unwrap();
        assert_eq!(out, []);
    }

    #[test]
    fn write_basic() {
        let mut out = Vec::<u8>::new();
        Lz4BlockOutput::new(&mut out, 128)
            .unwrap()
            .write_all("...".as_bytes())
            .unwrap();
        assert_eq!(out, VALID_DATA);
    }

    #[test]
    fn write_several_small_blocks() {
        let mut out = Vec::<u8>::new();
        let buf = ['.' as u8; 1024];
        let loops = 1024;
        {
            let mut writer = Lz4BlockOutput::new(&mut out, buf.len() * loops).unwrap();
            for _ in 0..loops {
                writer.write_all(&buf).unwrap();
            }
        }
        let needle = &VALID_DATA[..8];
        // count number of blocks
        assert_eq!(
            out.windows(needle.len())
                .filter(|window| *window == needle)
                .count(),
            1
        );
    }

    #[test]
    fn write_several_big_blocks() {
        let mut out = Vec::<u8>::new();
        let buf = ['.' as u8; 128];
        let loops = 1234;
        {
            let mut writer = Lz4BlockOutput::new(&mut out, buf.len()).unwrap();
            for _ in 0..loops {
                writer.write_all(&buf).unwrap();
            }
        }
        let needle = &VALID_DATA[..8];
        // count number of blocks
        assert_eq!(
            out.windows(needle.len())
                .filter(|window| *window == needle)
                .count(),
            loops
        );
    }

    #[test]
    fn flush_basic() {
        let mut out = Vec::<u8>::new();
        {
            let mut writer = Lz4BlockOutput::new(&mut out, 128).unwrap();
            writer.write_all("...".as_bytes()).unwrap();
            writer.flush().unwrap();
            writer.write_all("...".as_bytes()).unwrap();
        }
        let mut expected = VALID_DATA.to_vec();
        expected.extend_from_slice(&VALID_DATA[..]);
        assert_eq!(out, expected);
    }
}
