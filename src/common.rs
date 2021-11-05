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

// ErrorCorruptedStream

#[derive(Debug)]
pub(crate) struct ErrorCorruptedStream {}
impl ErrorCorruptedStream {
    pub(crate) fn new() -> Self {
        Self {}
    }
    pub(crate) fn new_error<R, E: From<Self>>() -> StdResult<R, E> {
        Err(Self::new().into())
    }
}
impl fmt::Display for ErrorCorruptedStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "corrupted stream")
    }
}
impl std::error::Error for ErrorCorruptedStream {}
impl From<ErrorCorruptedStream> for IoError {
    fn from(error: ErrorCorruptedStream) -> Self {
        Self::new(IoErrorKind::InvalidData, error)
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
    CorruptedStream(ErrorCorruptedStream),
    WrongBlockSize(ErrorWrongBlockSize),
    Lz4(Lz4Error),
    Io(IoError),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Internal(e) => e.fmt(f),
            Self::CorruptedStream(e) => e.fmt(f),
            Self::WrongBlockSize(e) => e.fmt(f),
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
impl From<ErrorCorruptedStream> for Error {
    fn from(error: ErrorCorruptedStream) -> Self {
        Self::CorruptedStream(error)
    }
}
impl From<ErrorWrongBlockSize> for Error {
    fn from(error: ErrorWrongBlockSize) -> Self {
        Self::WrongBlockSize(error)
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
            Error::CorruptedStream(err) => err.into(),
            Error::WrongBlockSize(err) => err.into(),
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
