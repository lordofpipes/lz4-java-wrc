use lz4_flex::block::{CompressError as Lz4CompressError, DecompressError as Lz4DecompressError};
pub(crate) use std::io::{Error as IoError, ErrorKind as IoErrorKind};
pub(crate) type Result<T> = std::result::Result<T, Error>;

use std::fmt;

// ErrorInternal

#[derive(Debug)]
pub(crate) struct ErrorInternal {
    description: &'static str,
}
impl ErrorInternal {
    pub(crate) fn new<E: From<Self>>(description: &'static str) -> E {
        Self {
            description: description,
        }
        .into()
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
    pub(crate) fn new<E: From<Self>>() -> E {
        Self {}.into()
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
    pub(crate) fn new<E: From<Self>>(size: usize, min_size: usize, max_size: usize) -> E {
        Self {
            size: size,
            min_size: min_size,
            max_size: max_size,
        }
        .into()
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

// Error

#[derive(Debug)]
pub(crate) enum Error {
    Internal(ErrorInternal),
    CorruptedStream(ErrorCorruptedStream),
    WrongBlockSize(ErrorWrongBlockSize),
    Lz4Compress(Lz4CompressError),
    Lz4Decompress(Lz4DecompressError),
    Io(IoError),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Internal(e) => e.fmt(f),
            Self::CorruptedStream(e) => e.fmt(f),
            Self::WrongBlockSize(e) => e.fmt(f),
            Self::Lz4Compress(e) => e.fmt(f),
            Self::Lz4Decompress(e) => e.fmt(f),
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
impl From<Lz4CompressError> for Error {
    fn from(error: Lz4CompressError) -> Self {
        Self::Lz4Compress(error)
    }
}
impl From<Lz4DecompressError> for Error {
    fn from(error: Lz4DecompressError) -> Self {
        Self::Lz4Decompress(error)
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
            Error::Lz4Compress(err) => Self::new(IoErrorKind::Other, err),
            Error::Lz4Decompress(err) => Self::new(IoErrorKind::Other, err),
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
        Self { f: f }
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
