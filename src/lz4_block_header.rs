use crate::common::{ErrorCorruptedStream, ErrorWrongBlockSize, IoErrorKind};

use lz4_flex::block::get_maximum_output_size as lz4_get_maximum_output_size;
use twox_hash::XxHash32;

use std::convert::TryInto;
use std::hash::Hasher;
use std::io::{Read, Result, Write};
use std::ops::Range;
use std::result::Result as StdResult;

const MAGIC_HEADER: [u8; 8] = [
    'L' as u8, 'Z' as u8, '4' as u8, 'B' as u8, 'l' as u8, 'o' as u8, 'c' as u8, 'k' as u8,
];
const MAGIC_HEADER_RANGE: Range<usize> = 0..MAGIC_HEADER.len();
const TOKEN_INDEX: usize = MAGIC_HEADER_RANGE.end;
const COMPRESSED_LEN_RANGE: Range<usize> = (TOKEN_INDEX + 1)..(TOKEN_INDEX + 5);
const DECOMPRESSED_LEN_RANGE: Range<usize> =
    COMPRESSED_LEN_RANGE.end..(COMPRESSED_LEN_RANGE.end + 4);
const CHECKSUM_RANGE: Range<usize> = DECOMPRESSED_LEN_RANGE.end..(DECOMPRESSED_LEN_RANGE.end + 4);
const HEADER_LENGTH: usize = CHECKSUM_RANGE.end;

const COMPRESSION_LEVEL_BASE: usize = 10;
const MIN_BLOCK_SIZE: usize = 64;
const MAX_BLOCK_SIZE: usize = 1 << (COMPRESSION_LEVEL_BASE + 0x0f);
const DEFAULT_SEED: u32 = 0x9747b28c;
const COMPRESSION_BLOCKS: [usize; 0x10] = [
    1 << (COMPRESSION_LEVEL_BASE + 14),
    1 << (COMPRESSION_LEVEL_BASE + 13),
    1 << (COMPRESSION_LEVEL_BASE + 12),
    1 << (COMPRESSION_LEVEL_BASE + 11),
    1 << (COMPRESSION_LEVEL_BASE + 10),
    1 << (COMPRESSION_LEVEL_BASE + 9),
    1 << (COMPRESSION_LEVEL_BASE + 8),
    1 << (COMPRESSION_LEVEL_BASE + 7),
    1 << (COMPRESSION_LEVEL_BASE + 6),
    1 << (COMPRESSION_LEVEL_BASE + 5),
    1 << (COMPRESSION_LEVEL_BASE + 4),
    1 << (COMPRESSION_LEVEL_BASE + 3),
    1 << (COMPRESSION_LEVEL_BASE + 2),
    1 << (COMPRESSION_LEVEL_BASE + 1),
    1 << (COMPRESSION_LEVEL_BASE + 0),
    0,
];

#[derive(Debug)]
pub(crate) struct Lz4BlockHeader {
    pub(crate) compression_method: CompressionMethod,
    pub(crate) compression_level: CompressionLevel,
    pub(crate) compressed_len: u32,
    pub(crate) decompressed_len: u32,
    pub(crate) checksum: u32,
}

impl Lz4BlockHeader {
    pub(crate) fn default_checksum(buf: &[u8]) -> u32 {
        let mut hasher = XxHash32::with_seed(DEFAULT_SEED);
        hasher.write(buf);
        // drop the 1st byte: https://github.com/lz4/lz4-java/blob/1.8.0/src/java/net/jpountz/xxhash/StreamingXXHash32.java#L106
        (hasher.finish() & 0x0fffffff) as u32
    }

    pub(crate) fn read<R: Read>(reader: &mut R) -> Result<Option<Self>> {
        let mut header = [0u8; HEADER_LENGTH];
        match reader.read_exact(&mut header[..]) {
            Err(err) => {
                return if matches!(err.kind(), IoErrorKind::UnexpectedEof) {
                    Ok(None)
                } else {
                    Err(err)
                };
            }
            Ok(_) => {}
        }
        let magic = &header[MAGIC_HEADER_RANGE];
        if magic != MAGIC_HEADER {
            return Err(ErrorCorruptedStream::new());
        }
        let compression_method = CompressionMethod::from_token(header[TOKEN_INDEX])?;
        let compression_level = CompressionLevel::from_token(header[TOKEN_INDEX]);
        let compressed_len = u32::from_le_bytes(header[COMPRESSED_LEN_RANGE].try_into().unwrap());
        let decompressed_len =
            u32::from_le_bytes(header[DECOMPRESSED_LEN_RANGE].try_into().unwrap());
        let checksum = u32::from_le_bytes(header[CHECKSUM_RANGE].try_into().unwrap());
        if decompressed_len > compression_level.get_max_decompressed_buffer_len() as u32
            || compressed_len > i32::MAX as u32 // java uses signed int
            || ((compressed_len == 0) != (decompressed_len == 0))
            || (matches!(compression_method, CompressionMethod::RAW)
                && compressed_len != decompressed_len)
        {
            return Err(ErrorCorruptedStream::new());
        }
        if compressed_len == 0 && decompressed_len == 0 {
            if checksum != 0 {
                return Err(ErrorCorruptedStream::new());
            }
        }
        Ok(Some(Self {
            compression_level: compression_level,
            compression_method: compression_method,
            compressed_len: compressed_len,
            decompressed_len: decompressed_len,
            checksum: checksum,
        }))
    }

    pub(crate) fn write<W: Write>(&self, writer: &mut W) -> Result<usize> {
        let mut buf = [0u8; HEADER_LENGTH];
        buf[MAGIC_HEADER_RANGE].clone_from_slice(&MAGIC_HEADER);
        buf[TOKEN_INDEX] = self.compression_level.get_token() | self.compression_method.get_token();
        buf[COMPRESSED_LEN_RANGE].clone_from_slice(&(self.compressed_len).to_le_bytes());
        buf[DECOMPRESSED_LEN_RANGE].clone_from_slice(&(self.decompressed_len).to_le_bytes());
        buf[CHECKSUM_RANGE].clone_from_slice(&(self.checksum).to_le_bytes());
        writer.write(&buf)
    }
}

// CompressionLevel

#[derive(Debug, Clone, Copy)]
pub(crate) struct CompressionLevel {
    compression_level: u8,
}

impl CompressionLevel {
    pub(crate) fn from_block_size(block_size: usize) -> StdResult<Self, ErrorWrongBlockSize> {
        if block_size < MIN_BLOCK_SIZE || block_size > MAX_BLOCK_SIZE {
            return Err(ErrorWrongBlockSize::new(
                block_size,
                MIN_BLOCK_SIZE,
                MAX_BLOCK_SIZE,
            ));
        }

        let index = COMPRESSION_BLOCKS
            .iter()
            .position(|block| *block < block_size)
            .ok_or(ErrorWrongBlockSize::new(
                block_size,
                MIN_BLOCK_SIZE,
                MAX_BLOCK_SIZE,
            ))?;
        Ok(Self {
            compression_level: (0x0f - index) as u8,
        })
    }

    pub(crate) fn get_max_decompressed_buffer_len(&self) -> usize {
        1 << (self.compression_level as usize + COMPRESSION_LEVEL_BASE as usize)
    }

    pub(crate) fn from_token(token: u8) -> Self {
        Self {
            compression_level: token & 0x0f,
        }
    }

    pub(crate) fn get_token(&self) -> u8 {
        self.compression_level
    }

    pub(crate) fn get_max_compressed_buffer_len(&self) -> usize {
        lz4_get_maximum_output_size(self.get_max_decompressed_buffer_len())
    }
}

// CompressionMethod

#[derive(Clone, Copy, Debug)]
pub(crate) enum CompressionMethod {
    RAW = 1,
    LZ4 = 2,
}

impl CompressionMethod {
    pub(crate) fn from_token(token: u8) -> StdResult<Self, ErrorCorruptedStream> {
        let compression_method = (token as usize & 0xf0) >> 4;
        match compression_method {
            x if x == Self::RAW as usize => Ok(Self::RAW),
            x if x == Self::LZ4 as usize => Ok(Self::LZ4),
            _ => Err(ErrorCorruptedStream::new()),
        }
    }

    pub(crate) fn get_token(&self) -> u8 {
        (self.clone() as u8) << 4
    }
}

#[cfg(test)]
pub(crate) mod data {
    use super::{HEADER_LENGTH, MAGIC_HEADER};

    pub(crate) const VALID_DATA: [u8; HEADER_LENGTH + 3] = [
        MAGIC_HEADER[0],
        MAGIC_HEADER[1],
        MAGIC_HEADER[2],
        MAGIC_HEADER[3],
        MAGIC_HEADER[4],
        MAGIC_HEADER[5],
        MAGIC_HEADER[6],
        MAGIC_HEADER[7],
        // token
        0x10,
        // compressed_len
        0x03,
        0x00,
        0x00,
        0x00,
        // decompressed_len
        0x03,
        0x00,
        0x00,
        0x00,
        // hash
        0x52,
        0xe4,
        0x77,
        0x06,
        // data
        '.' as u8,
        '.' as u8,
        '.' as u8,
    ];
    pub(crate) const VALID_EMPTY: [u8; HEADER_LENGTH] = [
        MAGIC_HEADER[0],
        MAGIC_HEADER[1],
        MAGIC_HEADER[2],
        MAGIC_HEADER[3],
        MAGIC_HEADER[4],
        MAGIC_HEADER[5],
        MAGIC_HEADER[6],
        MAGIC_HEADER[7],
        // token
        0x10,
        // compressed_len
        0x00,
        0x00,
        0x00,
        0x00,
        // decompressed_len
        0x00,
        0x00,
        0x00,
        0x00,
        // hash
        0x00,
        0x00,
        0x00,
        0x00,
    ];
}

#[cfg(test)]
mod test_lz4_block_header {
    use super::data::{VALID_DATA, VALID_EMPTY};
    use super::{
        CompressionMethod, Lz4BlockHeader, DECOMPRESSED_LEN_RANGE, HEADER_LENGTH, TOKEN_INDEX,
    };

    #[test]
    fn default_checksum_basic() {
        let mut v = VALID_DATA[HEADER_LENGTH..].to_vec();
        assert_eq!(Lz4BlockHeader::default_checksum(v.as_mut()), 0x0677e452);
    }

    #[test]
    fn read_too_small() {
        for s in 0..HEADER_LENGTH {
            let mut v = VALID_DATA[..s].to_vec();
            let mut d: &[u8] = v.as_mut();
            assert!(Lz4BlockHeader::read(&mut d).unwrap().is_none());
        }
    }

    #[test]
    fn read_empty() {
        let mut v = VALID_EMPTY[..].to_vec();
        let mut d: &[u8] = v.as_mut();
        let header = Lz4BlockHeader::read(&mut d).unwrap().unwrap();

        assert!(matches!(header.compression_method, CompressionMethod::RAW));
        assert_eq!(header.compression_level.compression_level, 0);
        assert_eq!(header.compressed_len, 0);
        assert_eq!(header.decompressed_len, 0);
        assert_eq!(header.checksum, 0);
    }

    #[test]
    fn read_valid() {
        let mut v = VALID_DATA[..HEADER_LENGTH].to_vec();
        let mut d: &[u8] = v.as_mut();
        let header = Lz4BlockHeader::read(&mut d).unwrap().unwrap();

        assert!(matches!(header.compression_method, CompressionMethod::RAW));
        assert_eq!(header.compression_level.compression_level, 0);
        assert_eq!(header.compressed_len, 3);
        assert_eq!(header.decompressed_len, 3);
        assert_eq!(header.checksum, 0x0677e452);
    }

    #[test]
    fn read_raw_different_sizes() {
        let mut v = VALID_DATA[..HEADER_LENGTH].to_vec();
        // update decompressed_len 3->4
        v[DECOMPRESSED_LEN_RANGE.start] += 1;
        let mut d: &[u8] = v.as_mut();
        assert!(Lz4BlockHeader::read(&mut d).is_err());
    }

    #[test]
    fn read_lz4_different_sizes() {
        let mut v = VALID_DATA[..HEADER_LENGTH].to_vec();
        // update decompressed_len 3->4 + token
        v[TOKEN_INDEX] = (v[TOKEN_INDEX] & 0x0f) | CompressionMethod::LZ4.get_token();
        v[DECOMPRESSED_LEN_RANGE.start] += 1;
        let mut d: &[u8] = v.as_mut();
        let header = Lz4BlockHeader::read(&mut d).unwrap().unwrap();

        assert!(matches!(header.compression_method, CompressionMethod::LZ4));
        assert_eq!(header.compression_level.compression_level, 0);
        assert_eq!(header.compressed_len, 3);
        assert_eq!(header.decompressed_len, 4);
        assert_eq!(header.checksum, 0x0677e452);
    }
}

#[cfg(test)]
mod test_compression_level {
    use super::{CompressionLevel, COMPRESSION_LEVEL_BASE, MAX_BLOCK_SIZE, MIN_BLOCK_SIZE};

    #[test]
    fn from_block_size_min() {
        assert_eq!(
            CompressionLevel::from_block_size(MIN_BLOCK_SIZE)
                .unwrap()
                .compression_level,
            0
        );
    }

    #[test]
    fn from_block_size_max() {
        assert_eq!(
            CompressionLevel::from_block_size(MAX_BLOCK_SIZE)
                .unwrap()
                .compression_level,
            0x0f
        );
    }

    #[test]
    fn from_block_size_valid() {
        for i in 0x00..0x0f {
            assert_eq!(
                CompressionLevel::from_block_size(1 << (COMPRESSION_LEVEL_BASE + i))
                    .unwrap()
                    .compression_level,
                i as u8
            );
            assert_eq!(
                CompressionLevel::from_block_size(1 << (COMPRESSION_LEVEL_BASE + i) + 1)
                    .unwrap()
                    .compression_level,
                (i + 1) as u8
            );
        }
    }

    #[test]
    fn from_block_size_too_small() {
        assert!(CompressionLevel::from_block_size(MIN_BLOCK_SIZE - 1).is_err());
    }

    #[test]
    fn from_block_size_too_big() {
        assert!(CompressionLevel::from_block_size(MAX_BLOCK_SIZE + 1).is_err());
    }

    #[test]
    fn from_token() {
        for token in 0x00..=0xff {
            assert_eq!(
                CompressionLevel::from_token(token).compression_level,
                token & 0x0f
            );
        }
    }
}

#[cfg(test)]
mod test_compression_method {
    use super::CompressionMethod;

    #[test]
    fn from_token_raw() {
        for i in 0x00..=0x0f {
            assert!(matches!(
                CompressionMethod::from_token(0x10 | i).unwrap(),
                CompressionMethod::RAW
            ));
        }
    }

    #[test]
    fn from_token_lz4() {
        for i in 0x00..=0x0f {
            assert!(matches!(
                CompressionMethod::from_token(0x20 | i).unwrap(),
                CompressionMethod::LZ4
            ));
        }
    }

    #[test]
    fn from_token_invalid() {
        for i in 0x00..=0xff {
            if i & 0xf0 != 0x10 && i & 0xf0 != 0x20 {
                assert!(CompressionMethod::from_token(i).is_err());
            }
        }
    }

    #[test]
    fn to_token_raw() {
        assert_eq!(CompressionMethod::RAW.get_token(), 0x10);
    }

    #[test]
    fn to_token_lz4() {
        assert_eq!(CompressionMethod::LZ4.get_token(), 0x20);
    }
}
