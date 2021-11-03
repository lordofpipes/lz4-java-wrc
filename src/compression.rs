use crate::common::Lz4Error;

/// Used to provide implementation for the LZ4 compression/decompression methods
pub trait Compression {
    /// Compress the data.
    ///
    /// # Arguments
    ///
    /// - input data to compress
    /// - output buffer to write to. It must be allocated with at least [`Self::get_maximum_compressed_buffer_len()`] bytes.
    ///
    /// # Result
    ///
    /// The number of bytes written into the output
    fn compress(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Lz4Error>;

    /// Decompress the data.
    ///
    /// # Arguments
    ///
    /// - input data to compress
    /// - output buffer to write to. It must be allocated with the number of bytes specified in the header.
    ///
    /// # Result
    ///
    /// The number of bytes written into the output
    fn decompress(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Lz4Error>;

    /// Find the maximum size of the output buffer when compressing.
    fn get_maximum_compressed_buffer_len(&self, decompressed_len: usize) -> usize;
}

// Context

/// Use a given context to switch between LZ4 libraries.
///
/// For most users, [`Context::default()`] is a good option.
#[derive(Debug, Copy, Clone)]
pub enum Context {
    #[cfg(feature = "lz4_flex")]
    /// Use the lz4_flex library to perform lz4 compression/decompression
    Lz4Flex,
    #[cfg(feature = "lz4-sys")]
    /// Use the lz4-sys library to perform lz4 compression/decompression
    Lz4Sys,
}

impl Default for Context {
    fn default() -> Self {
        match 0 {
            #[cfg(feature = "lz4_flex")]
            x if x == Self::Lz4Flex as usize => Self::Lz4Flex,
            #[cfg(feature = "lz4-sys")]
            x if x == Self::Lz4Sys as usize => Self::Lz4Sys,
            _ => panic!("No feature activated"),
        }
    }
}

impl Compression for Context {
    fn compress(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Lz4Error> {
        match self {
            #[cfg(feature = "lz4_flex")]
            Self::Lz4Flex => lz4_flex::compress(input, output),
            #[cfg(feature = "lz4-sys")]
            Self::Lz4Sys => lz4_sys::compress(input, output),
        }
    }
    fn decompress(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Lz4Error> {
        match self {
            #[cfg(feature = "lz4_flex")]
            Self::Lz4Flex => lz4_flex::decompress(input, output),
            #[cfg(feature = "lz4-sys")]
            Self::Lz4Sys => lz4_sys::decompress(input, output),
        }
    }
    fn get_maximum_compressed_buffer_len(&self, decompressed_len: usize) -> usize {
        match self {
            #[cfg(feature = "lz4_flex")]
            Self::Lz4Flex => lz4_flex::get_maximum_compressed_buffer_len(decompressed_len),
            #[cfg(feature = "lz4-sys")]
            Self::Lz4Sys => lz4_sys::get_maximum_compressed_buffer_len(decompressed_len),
        }
    }
}

#[cfg(feature = "lz4_flex")]
mod lz4_flex {
    use lz4_flex::block::{compress_into, decompress_into, get_maximum_output_size};

    use crate::common::Lz4Error;

    pub(crate) fn compress(input: &[u8], output: &mut [u8]) -> Result<usize, Lz4Error> {
        Ok(compress_into(input, output, 0)?)
    }
    pub(crate) fn decompress(input: &[u8], output: &mut [u8]) -> Result<usize, Lz4Error> {
        Ok(decompress_into(input, output, 0)?)
    }
    pub(crate) fn get_maximum_compressed_buffer_len(decompressed_len: usize) -> usize {
        get_maximum_output_size(decompressed_len)
    }
}

#[cfg(feature = "lz4-sys")]
mod lz4_sys {
    use libc::{c_char, c_int};
    use lz4_sys::{LZ4_compressBound, LZ4_compress_default, LZ4_decompress_safe};

    use crate::common::Lz4Error;

    pub(crate) fn compress(input: &[u8], output: &mut [u8]) -> Result<usize, Lz4Error> {
        let written_bytes = unsafe {
            LZ4_compress_default(
                input.as_ptr() as *const c_char,
                output.as_ptr() as *mut c_char,
                input.len() as c_int,
                output.len() as c_int,
            )
        };
        if written_bytes < 0 {
            Err(Lz4Error::Lz4SysCompressError)
        } else {
            Ok(written_bytes as usize)
        }
    }
    pub(crate) fn decompress(input: &[u8], output: &mut [u8]) -> Result<usize, Lz4Error> {
        let written_bytes = unsafe {
            LZ4_decompress_safe(
                input.as_ptr() as *const c_char,
                output.as_mut_ptr() as *mut c_char,
                input.len() as c_int,
                output.len() as c_int,
            )
        };
        if written_bytes < 0 {
            Err(Lz4Error::Lz4SysDecompressError)
        } else {
            Ok(written_bytes as usize)
        }
    }
    pub(crate) fn get_maximum_compressed_buffer_len(decompressed_len: usize) -> usize {
        unsafe { LZ4_compressBound(decompressed_len as c_int) as usize }
    }
}
