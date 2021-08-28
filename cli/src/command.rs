use crate::arguments::{FileDesc, Files, Mode};
use crate::read_counter::ReadCounter;

use std::fs::{remove_file, File, OpenOptions};
use std::io::{
    self, Error as IoError, ErrorKind as IoErrorKind, Read, Result, Stdin, Stdout, Write,
};

use atty::Stream;
use lz4jb::{Lz4BlockInput, Lz4BlockOutput};

enum EitherIo<T> {
    File(File),
    Stream(T),
}

impl<T: Read> EitherIo<T> {
    fn read(&mut self) -> &mut dyn Read {
        match self {
            Self::File(f) => f,
            Self::Stream(s) => s,
        }
    }
}

impl<T: Write> EitherIo<T> {
    fn write(&mut self) -> &mut dyn Write {
        match self {
            Self::File(f) => f,
            Self::Stream(s) => s,
        }
    }
}

fn run_compress(blocksize: usize, from: &mut dyn Read, to: &mut dyn Write) -> Result<()> {
    let mut to = Lz4BlockOutput::new(to, blocksize)?;
    io::copy(from, &mut to)?;
    to.flush()
}

fn run_decompress(from: &mut dyn Read, to: &mut dyn Write) -> Result<()> {
    let mut from = Lz4BlockInput::new(from);
    io::copy(&mut from, to)?;
    to.flush()
}

fn run_test(from: &mut dyn Read) -> Result<()> {
    let mut from = Lz4BlockInput::new(from);
    let mut to = io::sink();
    io::copy(&mut from, &mut to)?;
    to.flush()
}

fn run_list(from: &mut dyn Read, file: &str) -> Result<()> {
    let mut counter = ReadCounter::new(from);
    let mut from = Lz4BlockInput::new(&mut counter);
    let mut to = io::sink();
    let decompressed_size = io::copy(&mut from, &mut to)?;
    to.flush()?;
    let compressed_size = counter.sum();
    let ratio = 100. * (compressed_size as f64) / (decompressed_size as f64);
    println!(
        "{:>19} {:>19} {:>5.1}% {}",
        compressed_size, decompressed_size, ratio, file
    );
    Ok(())
}

fn get_filename_info(f: &FileDesc) -> &str {
    match f {
        FileDesc::Filename(f) => f,
        FileDesc::Stdio => "<stdio>",
        FileDesc::None => "<none>",
    }
}

pub(crate) struct Command {
    mode: Mode,
    keep_input: bool,
    force: bool,
}

impl Command {
    pub(crate) fn new(mode: Mode, keep_input: bool, force: bool) -> Self {
        match mode {
            Mode::List => println!("         compressed        decompressed  ratio filename"),
            _ => {}
        }

        Self {
            mode: mode,
            keep_input: keep_input,
            force: force,
        }
    }

    pub(crate) fn run(&self, files: &Files) -> Result<()> {
        let mut either_in = self.get_read(&files.file_in)?;
        let fd_in = either_in.read();
        match self.mode {
            Mode::Compress { block_size: bs } => {
                run_compress(bs, fd_in, self.get_write(&files.file_out)?.write())
            }
            Mode::Decompress => run_decompress(fd_in, self.get_write(&files.file_out)?.write()),
            Mode::Test => run_test(fd_in),
            Mode::List => run_list(fd_in, get_filename_info(&files.file_in)),
        }?;
        if !self.keep_input
            && matches!(
                self.mode,
                Mode::Compress { block_size: _ } | Mode::Decompress
            )
            && matches!(files.file_out, FileDesc::Filename(_))
        {
            match &files.file_in {
                FileDesc::Filename(f) => remove_file(f)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn get_read(&self, file_in: &FileDesc) -> Result<EitherIo<Stdin>> {
        Ok(match file_in {
            FileDesc::Filename(f) => EitherIo::File(File::open(f)?),
            FileDesc::Stdio => EitherIo::Stream(io::stdin()),
            FileDesc::None => {
                return Err(IoError::new(
                    IoErrorKind::Unsupported,
                    "could not open the input.",
                ))
            }
        })
    }

    fn get_write(&self, file_out: &FileDesc) -> Result<EitherIo<Stdout>> {
        Ok(match file_out {
            FileDesc::Filename(f) => EitherIo::File(
                OpenOptions::new()
                    .write(true)
                    .create(self.force)
                    .create_new(!self.force)
                    .truncate(true)
                    .open(f)?,
            ),
            FileDesc::Stdio => {
                if !self.force
                    && matches!(self.mode, Mode::Compress { block_size: _ })
                    && atty::is(Stream::Stdout)
                {
                    return Err(IoError::new(
                        IoErrorKind::InvalidInput,
                        "stdout is a terminal. Use -f to force compression.",
                    ));
                } else {
                    EitherIo::Stream(io::stdout())
                }
            }
            FileDesc::None => {
                return Err(IoError::new(
                    IoErrorKind::Unsupported,
                    "could not open the output.",
                ))
            }
        })
    }
}
