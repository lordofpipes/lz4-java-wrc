#[cfg(feature = "lz4_flex")]
use lz4_flex::block::{
    CompressError as Lz4FlexCompressError, DecompressError as Lz4FlexDecompressError,
};

use std::error::Error as StdError;
use std::fmt;
pub(crate) use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::result::Result as StdResult;

pub(crate) type Result<T> = StdResult<T, Error>;

// ErrorInternal

#[derive(Debug)]
pub(crate) struct ErrorInternal {
    description: &'static str,
}
impl ErrorInternal {
    pub(crate) fn new(description: &'static str) -> Self {
        Self { description }
    }
    pub(crate) fn new_error<R, E: From<Self>>(description: &'static str) -> StdResult<R, E> {
        Err(Self::new(description).into())
    }
}
impl fmt::Display for ErrorInternal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "internal error: {}", self.description)
    }
}
impl std::error::Error for ErrorInternal {}

// ErrorMagicNumber

#[derive(Debug)]
pub(crate) struct ErrorMagicNumber {
    expected: u64,
    actual: u64,
}
impl ErrorMagicNumber {
    pub(crate) fn new(expected: u64, actual: u64) -> Self {
        Self { expected, actual }
    }
    pub(crate) fn new_error<R, E: From<Self>>(expected: u64, actual: u64) -> StdResult<R, E> {
        Err(Self::new(expected, actual).into())
    }
}
impl fmt::Display for ErrorMagicNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "wrong magic number from header: {:016x?} instead of {:016x?}",
            self.actual, self.expected
        )
    }
}
impl std::error::Error for ErrorMagicNumber {}

// ErrorCompressionMethod

#[derive(Debug)]
pub(crate) struct ErrorCompressionMethod {
    token: u8,
}
impl ErrorCompressionMethod {
    pub(crate) fn new(token: u8) -> Self {
        Self { token }
    }
    pub(crate) fn new_error<R, E: From<Self>>(token: u8) -> StdResult<R, E> {
        Err(Self::new(token).into())
    }
}
impl fmt::Display for ErrorCompressionMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "wrong token {:02x?}: unknown compression method",
            self.token
        )
    }
}
impl std::error::Error for ErrorCompressionMethod {}

// ErrorDecompressedSizeTooBig

#[derive(Debug)]
pub(crate) struct ErrorDecompressedSizeTooBig {
    decompressed_size: u32,
    max_size: usize,
}
impl ErrorDecompressedSizeTooBig {
    pub(crate) fn new(decompressed_size: u32, max_size: usize) -> Self {
        Self {
            decompressed_size,
            max_size,
        }
    }
    pub(crate) fn new_error<R, E: From<Self>>(
        decompressed_size: u32,
        max_size: usize,
    ) -> StdResult<R, E> {
        Err(Self::new(decompressed_size, max_size).into())
    }
}
impl fmt::Display for ErrorDecompressedSizeTooBig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the decompressed size: {} is bigger than the maximum size: {}",
            self.decompressed_size, self.max_size,
        )
    }
}
impl std::error::Error for ErrorDecompressedSizeTooBig {}

// ErrorCompressedSizeTooBig

#[derive(Debug)]
pub(crate) struct ErrorCompressedSizeTooBig {
    compressed_size: u32,
    max_size: u32,
}
impl ErrorCompressedSizeTooBig {
    pub(crate) fn new(compressed_size: u32, max_size: u32) -> Self {
        Self {
            compressed_size,
            max_size,
        }
    }
    pub(crate) fn new_error<R, E: From<Self>>(
        compressed_size: u32,
        max_size: u32,
    ) -> StdResult<R, E> {
        Err(Self::new(compressed_size, max_size).into())
    }
}
impl fmt::Display for ErrorCompressedSizeTooBig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the compressed size: {} is bigger than the maximum size: {}",
            self.compressed_size, self.max_size,
        )
    }
}
impl std::error::Error for ErrorCompressedSizeTooBig {}

// ErrorIncoherentSize

#[derive(Debug)]
pub(crate) struct ErrorIncoherentSize {
    decompressed_size: u32,
    compressed_size: u32,
}
impl ErrorIncoherentSize {
    pub(crate) fn new(decompressed_size: u32, compressed_size: u32) -> Self {
        Self {
            decompressed_size,
            compressed_size,
        }
    }
    pub(crate) fn new_error<R, E: From<Self>>(
        decompressed_size: u32,
        compressed_size: u32,
    ) -> StdResult<R, E> {
        Err(Self::new(decompressed_size, compressed_size).into())
    }
}
impl fmt::Display for ErrorIncoherentSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the decompressed size: {} no coherent with the compressed size: {}",
            self.decompressed_size, self.compressed_size,
        )
    }
}
impl std::error::Error for ErrorIncoherentSize {}

// ErrorNoCompressionDifferentSize

#[derive(Debug)]
pub(crate) struct ErrorNoCompressionDifferentSize {
    compressed_size: u32,
    decompressed_size: u32,
}
impl ErrorNoCompressionDifferentSize {
    pub(crate) fn new(compressed_size: u32, decompressed_size: u32) -> Self {
        Self {
            compressed_size,
            decompressed_size,
        }
    }
    pub(crate) fn new_error<R, E: From<Self>>(
        compressed_size: u32,
        decompressed_size: u32,
    ) -> StdResult<R, E> {
        Err(Self::new(compressed_size, decompressed_size).into())
    }
}
impl fmt::Display for ErrorNoCompressionDifferentSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "without compression, the compressed size: {} different than the decompressed size: {}",
            self.compressed_size, self.decompressed_size,
        )
    }
}
impl std::error::Error for ErrorNoCompressionDifferentSize {}

// ErrorChecksum

#[derive(Debug)]
pub(crate) struct ErrorChecksum {
    header_value: u32,
    computed_value: u32,
}
impl ErrorChecksum {
    pub(crate) fn new(header_value: u32, computed_value: u32) -> Self {
        Self {
            header_value,
            computed_value,
        }
    }
    pub(crate) fn new_error<R, E: From<Self>>(
        header_value: u32,
        computed_value: u32,
    ) -> StdResult<R, E> {
        Err(Self::new(header_value, computed_value).into())
    }
}
impl fmt::Display for ErrorChecksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "wrong checksum: header value={:08X} computed value={:08X}",
            self.header_value, self.computed_value
        )
    }
}
impl std::error::Error for ErrorChecksum {}

// ErrorLz4WrongDecompressedSize

#[derive(Debug)]
pub(crate) struct ErrorLz4WrongDecompressedSize {
    expected_uncompressed_size: usize,
    uncompressed_size: usize,
}
impl ErrorLz4WrongDecompressedSize {
    pub(crate) fn new(expected_uncompressed_size: usize, uncompressed_size: usize) -> Self {
        Self {
            expected_uncompressed_size,
            uncompressed_size,
        }
    }
    pub(crate) fn new_error<R, E: From<Self>>(
        expected_uncompressed_size: usize,
        uncompressed_size: usize,
    ) -> StdResult<R, E> {
        Err(Self::new(expected_uncompressed_size, uncompressed_size).into())
    }
}
impl fmt::Display for ErrorLz4WrongDecompressedSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the uncompressed data size using LZ4={} is different than the value from the header={}",
            self.uncompressed_size, self.expected_uncompressed_size
        )
    }
}
impl std::error::Error for ErrorLz4WrongDecompressedSize {}

// Lz4Flex

#[derive(Debug)]
pub enum Lz4Error {
    #[cfg(feature = "lz4_flex")]
    Lz4FlexCompressError(Lz4FlexCompressError),
    #[cfg(feature = "lz4_flex")]
    Lz4FlexDecompressError(Lz4FlexDecompressError),
    #[cfg(feature = "lz4-sys")]
    Lz4SysCompressError,
    #[cfg(feature = "lz4-sys")]
    Lz4SysDecompressError,
}
impl fmt::Display for Lz4Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "lz4_flex")]
            Self::Lz4FlexCompressError(e) => write!(f, "lz4_flex compression failed: {}", e),
            #[cfg(feature = "lz4_flex")]
            Self::Lz4FlexDecompressError(e) => write!(f, "lz4_flex decompression failed: {}", e),
            #[cfg(feature = "lz4-sys")]
            Self::Lz4SysCompressError => write!(f, "lz4-sys compression failed"),
            #[cfg(feature = "lz4-sys")]
            Self::Lz4SysDecompressError => write!(f, "lz4-sys decompression failed"),
        }
    }
}
impl StdError for Lz4Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            #[cfg(feature = "lz4_flex")]
            Self::Lz4FlexCompressError(e) => Some(e),
            #[cfg(feature = "lz4_flex")]
            Self::Lz4FlexDecompressError(e) => Some(e),
            #[cfg(feature = "lz4-sys")]
            Self::Lz4SysCompressError => None,
            #[cfg(feature = "lz4-sys")]
            Self::Lz4SysDecompressError => None,
        }
    }
}
#[cfg(feature = "lz4_flex")]
impl From<Lz4FlexCompressError> for Lz4Error {
    fn from(error: Lz4FlexCompressError) -> Self {
        Self::Lz4FlexCompressError(error)
    }
}
#[cfg(feature = "lz4_flex")]
impl From<Lz4FlexDecompressError> for Lz4Error {
    fn from(error: Lz4FlexDecompressError) -> Self {
        Self::Lz4FlexDecompressError(error)
    }
}

// Error

#[derive(Debug)]
pub(crate) enum Error {
    Internal(ErrorInternal),
    MagicNumber(ErrorMagicNumber),
    CompressionMethod(ErrorCompressionMethod),
    DecompressedSizeTooBig(ErrorDecompressedSizeTooBig),
    CompressedSizeTooBig(ErrorCompressedSizeTooBig),
    IncoherentSize(ErrorIncoherentSize),
    NoCompressionDifferentSize(ErrorNoCompressionDifferentSize),
    Checksum(ErrorChecksum),
    Lz4WrongDecompressedSize(ErrorLz4WrongDecompressedSize),
    Lz4(Lz4Error),
    Io(IoError),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Internal(e) => e.fmt(f),
            Self::MagicNumber(e) => e.fmt(f),
            Self::CompressionMethod(e) => e.fmt(f),
            Self::DecompressedSizeTooBig(e) => e.fmt(f),
            Self::CompressedSizeTooBig(e) => e.fmt(f),
            Self::IncoherentSize(e) => e.fmt(f),
            Self::NoCompressionDifferentSize(e) => e.fmt(f),
            Self::Checksum(e) => e.fmt(f),
            Self::Lz4WrongDecompressedSize(e) => e.fmt(f),
            Self::Lz4(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}
impl From<&'static str> for Error {
    fn from(error: &'static str) -> Self {
        Self::Internal(ErrorInternal { description: error })
    }
}
impl From<ErrorInternal> for Error {
    fn from(error: ErrorInternal) -> Self {
        Self::Internal(error)
    }
}
impl From<ErrorMagicNumber> for Error {
    fn from(error: ErrorMagicNumber) -> Self {
        Self::MagicNumber(error)
    }
}
impl From<ErrorCompressionMethod> for Error {
    fn from(error: ErrorCompressionMethod) -> Self {
        Self::CompressionMethod(error)
    }
}
impl From<ErrorDecompressedSizeTooBig> for Error {
    fn from(error: ErrorDecompressedSizeTooBig) -> Self {
        Self::DecompressedSizeTooBig(error)
    }
}
impl From<ErrorCompressedSizeTooBig> for Error {
    fn from(error: ErrorCompressedSizeTooBig) -> Self {
        Self::CompressedSizeTooBig(error)
    }
}
impl From<ErrorIncoherentSize> for Error {
    fn from(error: ErrorIncoherentSize) -> Self {
        Self::IncoherentSize(error)
    }
}
impl From<ErrorNoCompressionDifferentSize> for Error {
    fn from(error: ErrorNoCompressionDifferentSize) -> Self {
        Self::NoCompressionDifferentSize(error)
    }
}
impl From<ErrorChecksum> for Error {
    fn from(error: ErrorChecksum) -> Self {
        Self::Checksum(error)
    }
}
impl From<ErrorLz4WrongDecompressedSize> for Error {
    fn from(error: ErrorLz4WrongDecompressedSize) -> Self {
        Self::Lz4WrongDecompressedSize(error)
    }
}
impl From<Lz4Error> for Error {
    fn from(error: Lz4Error) -> Self {
        Self::Lz4(error)
    }
}
impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}
impl From<Error> for IoError {
    fn from(error: Error) -> Self {
        match error {
            Error::Internal(err) => Self::new(IoErrorKind::Other, err),
            Error::MagicNumber(err) => Self::new(IoErrorKind::InvalidData, err),
            Error::CompressionMethod(err) => Self::new(IoErrorKind::InvalidData, err),
            Error::DecompressedSizeTooBig(err) => Self::new(IoErrorKind::InvalidData, err),
            Error::CompressedSizeTooBig(err) => Self::new(IoErrorKind::InvalidData, err),
            Error::IncoherentSize(err) => Self::new(IoErrorKind::InvalidData, err),
            Error::NoCompressionDifferentSize(err) => Self::new(IoErrorKind::InvalidData, err),
            Error::Checksum(err) => Self::new(IoErrorKind::InvalidData, err),
            Error::Lz4WrongDecompressedSize(err) => Self::new(IoErrorKind::InvalidData, err),
            Error::Lz4(err) => Self::new(IoErrorKind::Other, err),
            Error::Io(err) => err,
        }
    }
}

// Checksum

pub(crate) struct Checksum {
    f: fn(&[u8]) -> u32,
}

impl Checksum {
    pub(crate) fn new(f: fn(&[u8]) -> u32) -> Self {
        Self { f }
    }

    pub(crate) fn run(&self, buf: &[u8]) -> u32 {
        let f = self.f;
        f(buf)
    }
}

impl fmt::Debug for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(self.f as *const ()), f)
    }
}

// ErrorWrongBlockSize

#[derive(Debug)]
pub(crate) struct ErrorWrongBlockSize {
    size: usize,
    min_size: usize,
    max_size: usize,
}
impl ErrorWrongBlockSize {
    pub(crate) fn new(size: usize, min_size: usize, max_size: usize) -> Self {
        Self {
            size,
            min_size,
            max_size,
        }
    }
    pub(crate) fn new_error<R, E: From<Self>>(
        size: usize,
        min_size: usize,
        max_size: usize,
    ) -> StdResult<R, E> {
        Err(Self::new(size, min_size, max_size).into())
    }
}
impl fmt::Display for ErrorWrongBlockSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "wrong block size {}. It should be between {} and {}",
            self.size, self.min_size, self.max_size
        )
    }
}
impl std::error::Error for ErrorWrongBlockSize {}
impl From<ErrorWrongBlockSize> for IoError {
    fn from(error: ErrorWrongBlockSize) -> Self {
        Self::new(IoErrorKind::InvalidData, error)
    }
}
