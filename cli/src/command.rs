use crate::arguments::{FileDesc, Files, Mode};
use crate::read_counter::ReadCounter;

use std::fs::{metadata, remove_file, set_permissions, File, OpenOptions};
use std::io::{
    self, Error as IoError, ErrorKind as IoErrorKind, Read, Result, Stdin, Stdout, Write,
};

use atty::Stream;
use lz4jb::{Context as Lz4Context, Lz4BlockInput, Lz4BlockOutput};

pub enum EitherIo<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Read for EitherIo<L, R>
where
    L: Read,
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::Left(l) => l.read(buf),
            Self::Right(r) => r.read(buf),
        }
    }
}

impl<L, R> Write for EitherIo<L, R>
where
    L: Write,
    R: Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            Self::Left(l) => l.write(buf),
            Self::Right(r) => r.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            Self::Left(l) => l.flush(),
            Self::Right(r) => r.flush(),
        }
    }
}

fn run_compress<R: Read, W: Write>(
    context: Lz4Context,
    blocksize: Option<usize>,
    mut from: R,
    to: W,
) -> Result<()> {
    let mut to = match blocksize {
        Some(bs) => Lz4BlockOutput::with_context(to, context, bs)?,
        None => Lz4BlockOutput::new(to),
    };
    io::copy(&mut from, &mut to)?;
    to.flush()
}

fn run_decompress<R: Read, W: Write>(context: Lz4Context, from: R, mut to: W) -> Result<()> {
    let mut from = Lz4BlockInput::with_context(from, context);
    io::copy(&mut from, &mut to)?;
    to.flush()
}

fn run_test<R: Read>(context: Lz4Context, from: R) -> Result<()> {
    let mut from = Lz4BlockInput::with_context(from, context);
    let mut to = io::sink();
    io::copy(&mut from, &mut to)?;
    to.flush()
}

fn run_list<R: Read>(context: Lz4Context, from: R, file: &str) -> Result<()> {
    let mut counter = ReadCounter::new(from);
    let mut from = Lz4BlockInput::with_context(&mut counter, context);
    let mut to = io::sink();
    let decompressed_size = io::copy(&mut from, &mut to)?;
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
    context: Lz4Context,
    mode: Mode,
    keep_input: bool,
    force: bool,
}

impl Command {
    pub(crate) fn new(context: Lz4Context, mode: Mode, keep_input: bool, force: bool) -> Self {
        if let Mode::List = mode {
            println!("         compressed        decompressed  ratio filename");
        }

        Self {
            context,
            mode,
            keep_input,
            force,
        }
    }

    pub(crate) fn run(&self, files: &Files) -> Result<()> {
        let read = self.get_read(&files.file_in)?;

        match self.mode {
            Mode::Compress { block_size: bs } => {
                run_compress(self.context, bs, read, self.get_write(&files.file_out)?)
            }
            Mode::Decompress => {
                run_decompress(self.context, read, self.get_write(&files.file_out)?)
            }
            Mode::Test => run_test(self.context, read),
            Mode::List => run_list(self.context, read, get_filename_info(&files.file_in)),
        }?;

        if let (FileDesc::Filename(f_in), FileDesc::Filename(f_out)) =
            (&files.file_in, &files.file_out)
        {
            metadata(f_in).and_then(|meta| set_permissions(f_out, meta.permissions()))?;
            if !self.keep_input
                && matches!(
                    self.mode,
                    Mode::Compress { block_size: _ } | Mode::Decompress
                )
            {
                remove_file(f_in)?;
            }
        }
        Ok(())
    }

    fn get_read(&self, file_in: &FileDesc) -> Result<EitherIo<File, Stdin>> {
        Ok(match file_in {
            FileDesc::Filename(f) => EitherIo::Left(File::open(f)?),
            FileDesc::Stdio => EitherIo::Right(io::stdin()),
            FileDesc::None => {
                return Err(IoError::new(
                    IoErrorKind::Unsupported,
                    "could not open the input.",
                ))
            }
        })
    }

    fn get_write(&self, file_out: &FileDesc) -> Result<EitherIo<File, Stdout>> {
        Ok(match file_out {
            FileDesc::Filename(f) => EitherIo::Left(
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
                    EitherIo::Right(io::stdout())
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
