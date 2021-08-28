use std::io::{Read, Result};

pub(crate) struct ReadCounter<'a> {
    read: &'a mut dyn Read,
    sum: u64,
}

impl<'a> ReadCounter<'a> {
    pub(crate) fn new(read: &'a mut dyn Read) -> Self {
        Self { read: read, sum: 0 }
    }

    pub(crate) fn sum(&self) -> u64 {
        self.sum
    }
}

impl<'a> Read for ReadCounter<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let res = self.read.read(buf);
        self.sum += match res {
            Ok(s) => s as u64,
            _ => 0,
        };
        res
    }
}
