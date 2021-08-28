use clap::{App, Arg};

use std::fmt;

const DEFAULT_BLOCKSIZE: usize = 1 << 16;
const DEFAULT_SUFFIX: &str = ".lz4";

#[derive(Debug, Copy, Clone)]
pub(crate) enum Mode {
    Compress { block_size: usize },
    Decompress,
    List,
    Test,
}
impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Compress { block_size: _ } => write!(f, "compress"),
            Mode::Decompress => write!(f, "decompress"),
            Mode::List => write!(f, "list"),
            Mode::Test => write!(f, "test"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum FileDesc {
    Filename(String),
    Stdio,
    None,
}

impl FileDesc {
    fn decompressed(
        decompressed_name: &str,
        suffix: &str,
        to_stdout: bool,
    ) -> Result<Self, &'static str> {
        if to_stdout {
            Ok(Self::Stdio)
        } else {
            Ok(Self::Filename(
                decompressed_name
                    .strip_suffix(suffix)
                    .map(|f| Ok(f.to_string()))
                    .unwrap_or_else(|| Err("Could not guess the output filename"))?,
            ))
        }
    }
    fn compressed(
        compressed_name: &str,
        suffix: &str,
        to_stdout: bool,
    ) -> Result<Self, &'static str> {
        if to_stdout {
            Ok(Self::Stdio)
        } else {
            Ok(Self::Filename(compressed_name.to_string() + suffix))
        }
    }
}
impl fmt::Display for FileDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Filename(filename) => write!(f, "filename={}", filename),
            Self::Stdio => write!(f, "stdio"),
            Self::None => write!(f, "<none>"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Files {
    pub(crate) file_in: FileDesc,
    pub(crate) file_out: FileDesc,
}
impl Files {
    fn stdio() -> Self {
        Self {
            file_in: FileDesc::Stdio,
            file_out: FileDesc::Stdio,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Arguments {
    pub(crate) files: Vec<Files>,
    pub(crate) mode: Mode,
    pub(crate) keep_input: bool,
    pub(crate) force: bool,
}

fn build_cli() -> App<'static, 'static> {
    App::new("lz4jb")
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("compress")
                .short("z")
                .long("compress")
                .conflicts_with_all(&["decompress", "list", "test"])
                .help("Compress. This is the default operation mode."),
        )
        .arg(
            Arg::with_name("decompress")
                .short("d")
                .long("decompress")
                .visible_alias("uncompress")
                .conflicts_with_all(&["compress", "list", "test"])
                .help("Decompress."),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .conflicts_with_all(&["compress", "decompress", "test"])
                .help("Test the integrity of compressed files."),
        )
        .arg(
            Arg::with_name("test")
                .short("t")
                .long("test")
                .conflicts_with_all(&["compress", "decompress", "list"])
                .help("Test the integrity of compressed files."),
        )
        .arg(
            Arg::with_name("stdout")
                .short("c")
                .long("stdout")
                .conflicts_with_all(&["list", "test"])
                .help("Write output on standard output; keep original files unchanged."),
        )
        .arg(
            Arg::with_name("keep")
                .short("k")
                .long("keep")
                .conflicts_with_all(&["list", "test"])
                .help("Keep (don't delete) input files during compression or decompression."),
        )
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .conflicts_with_all(&["list", "test"])
                .help("Force the compression or decompression."),
        )
        .arg(
            Arg::with_name("suffix")
                .short("S")
                .long("suffix")
                .takes_value(true)
                .conflicts_with_all(&["list", "test"])
                .help("Append this suffix instead of the default .lz4 for compression."),
        )
        .arg(
            Arg::with_name("blocksize")
                .short("b")
                .long("blocksize")
                .takes_value(true)
                .conflicts_with_all(&["decompress", "list", "test"])
                .help("Block size for compression in bytes (between 64 and 33554432)."),
        )
        .arg(
            Arg::with_name("file")
                .help("Sets the input file to use.")
                .multiple(true),
        )
}

pub(crate) fn parse_cli() -> Result<Arguments, &'static str> {
    let matches = build_cli().get_matches();

    let mode = match (
        matches.is_present("compress"),
        matches.is_present("decompress"),
        matches.is_present("list"),
        matches.is_present("test"),
    ) {
        (_, false, false, false) => Mode::Compress {
            block_size: matches
                .value_of("blocksize")
                .map(str::parse::<usize>)
                .unwrap_or_else(|| Ok(DEFAULT_BLOCKSIZE))
                .map_err(|_| "Failed to parse the blocksize argument as integer")?,
        },
        (false, true, false, false) => Mode::Decompress,
        (false, false, true, false) => Mode::List,
        (false, false, false, true) => Mode::Test,
        _ => return Err(
            "Maximum 1 amongst the following arguments: --compress, --decompress, --list, --test",
        ),
    };

    let suffix = matches.value_of("suffix").unwrap_or(DEFAULT_SUFFIX);
    let to_stdout = matches.is_present("stdout");
    let keep_input = matches.is_present("keep");
    let force = matches.is_present("force");
    let files = matches
        .values_of("file")
        .into_iter()
        .flatten()
        .map(|f| {
            Ok(Files {
                file_in: FileDesc::Filename(f.into()),
                file_out: match mode {
                    Mode::Compress { block_size: _ } => FileDesc::compressed(f, suffix, to_stdout)?,
                    Mode::Decompress => FileDesc::decompressed(f, suffix, to_stdout)?,
                    _ => FileDesc::None,
                },
            })
        })
        .collect::<Result<Vec<Files>, &'static str>>()?;
    Ok(Arguments {
        files: if files.is_empty() {
            vec![Files::stdio()]
        } else {
            files
        },
        mode: mode,
        keep_input: keep_input,
        force: force,
    })
}
