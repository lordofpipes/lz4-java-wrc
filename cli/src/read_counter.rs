use std::io::{Read, Result};

pub(crate) struct ReadCounter<R: Read> {
    read: R,
    sum: u64,
}

impl<R: Read> ReadCounter<R> {
    pub(crate) fn new(read: R) -> Self {
        Self { read, sum: 0 }
    }

    pub(crate) fn sum(&self) -> u64 {
        self.sum
    }
}

impl<R: Read> Read for ReadCounter<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let res = self.read.read(buf);
        if let Ok(s) = res {
            self.sum += s as u64;
        }
        res
    }
}

#[cfg(test)]
mod test_read_counter {
    use super::ReadCounter;
    use std::io::{ErrorKind, Read, Result};

    #[test]
    fn read_basic() {
        let from: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut to = vec![0u8; from.len()];

        let mut reader = ReadCounter::new(from.as_ref());
        let (r1, r2, r3) = (
            reader.read(&mut to[..4]).unwrap(),
            reader.read(&mut to[4..8]).unwrap(),
            reader.read(&mut to[8..]).unwrap(),
        );

        assert_eq!(r1, 4);
        assert_eq!(r2, 4);
        assert_eq!(r3, from.len() - (r1 + r2));
        assert_eq!(to, from);
        assert_eq!(reader.sum(), from.len() as u64);
    }

    struct ReadError {}
    impl Read for ReadError {
        fn read(&mut self, _: &mut [u8]) -> Result<usize> {
            Err(ErrorKind::Other.into())
        }
    }

    #[test]
    fn read_error() {
        let mut reader = ReadCounter::new(ReadError {});
        let mut to = [0u8];
        let err = reader.read(&mut to).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::Other);
    }
}
