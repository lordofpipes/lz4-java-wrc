use clap::{App, Arg};
use lz4jb::Context as Lz4Context;

use std::fmt;

const DEFAULT_SUFFIX: &str = ".lz4";

#[cfg(feature = "lz4_flex")]
const AVAILABLE_LIBRARY_LZ4_FLEX: Option<Lz4Context> = Some(Lz4Context::Lz4Flex);
#[cfg(not(feature = "lz4_flex"))]
const AVAILABLE_LIBRARY_LZ4_FLEX: Option<Lz4Context> = None;
#[cfg(feature = "lz4-sys")]
const AVAILABLE_LIBRARY_LZ4_SYS: Option<Lz4Context> = Some(Lz4Context::Lz4Sys);
#[cfg(not(feature = "lz4-sys"))]
const AVAILABLE_LIBRARY_LZ4_SYS: Option<Lz4Context> = None;

const AVAILABLE_LIBRARIES: [(&str, Option<Lz4Context>, &str); 2] = [
    (
        "lz4_flex",
        AVAILABLE_LIBRARY_LZ4_FLEX,
        "use the lz4_flex library (https://crates.io/crates/lz4_flex).",
    ),
    (
        "lz4-sys",
        AVAILABLE_LIBRARY_LZ4_SYS,
        "use the lz4-sys library (https://crates.io/crates/lz4-sys).",
    ),
];

#[derive(Debug, Copy, Clone)]
pub(crate) enum Mode {
    Compress { block_size: Option<usize> },
    Decompress,
    List,
    Test,
}
impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Compress { block_size: _ } => write!(f, "compress"),
            Self::Decompress => write!(f, "decompress"),
            Self::List => write!(f, "list"),
            Self::Test => write!(f, "test"),
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
    pub(crate) lz4jb_context: Lz4Context,
}

fn get_library(name: &str) -> Option<Lz4Context> {
    AVAILABLE_LIBRARIES
        .iter()
        .find(|t| name == t.0)
        .map(|t| t.1)
        .flatten()
}

pub(crate) fn parse_cli() -> Result<Arguments, &'static str> {
    let library_long_help = format!(
        "Use an alternative library. Available libraries:\n{}",
        AVAILABLE_LIBRARIES
            .iter()
            .filter(|t| t.1.is_some())
            .map(|t| format!(" - {}: {}", t.0, t.2))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let app = App::new("lz4jb")
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("compress")
                .short("z")
                .long("compress")
                .conflicts_with_all(&["decompress", "list", "test"])
                .help("Compress. This is the default operation mode.")
                .display_order(1),
        )
        .arg(
            Arg::with_name("decompress")
                .short("d")
                .long("decompress")
                .visible_alias("uncompress")
                .conflicts_with_all(&["compress", "list", "test"])
                .help("Decompress.")
                .display_order(1),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .conflicts_with_all(&["compress", "decompress", "test"])
                .help("Test the integrity of compressed files.")
                .display_order(1),
        )
        .arg(
            Arg::with_name("test")
                .short("t")
                .long("test")
                .conflicts_with_all(&["compress", "decompress", "list"])
                .help("Test the integrity of compressed files.")
                .display_order(1),
        )
        .arg(
            Arg::with_name("stdout")
                .short("c")
                .long("stdout")
                .conflicts_with_all(&["list", "test"])
                .help("Write output on standard output; keep original files unchanged.")
                .display_order(100),
        )
        .arg(
            Arg::with_name("keep")
                .short("k")
                .long("keep")
                .conflicts_with_all(&["list", "test"])
                .help("Keep (don't delete) input files during compression or decompression.")
                .display_order(100),
        )
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .conflicts_with_all(&["list", "test"])
                .help("Force the compression or decompression.")
                .display_order(100),
        )
        .arg(
            Arg::with_name("suffix")
                .short("S")
                .long("suffix")
                .takes_value(true)
                .conflicts_with_all(&["list", "test"])
                .help("Append this suffix instead of the default .lz4 for compression.")
                .display_order(100),
        )
        .arg(
            Arg::with_name("blocksize")
                .short("b")
                .long("blocksize")
                .takes_value(true)
                .conflicts_with_all(&["decompress", "list", "test"])
                .help("Block size for compression in bytes (between 64 and 33554432).")
                .display_order(100),
        )
        .arg(
            Arg::with_name("library")
                .short("L")
                .long("library")
                .takes_value(true)
                .help("Use an alternative library. See --help for the list of available libraries.")
                .long_help(library_long_help.as_str())
                .validator(|v| {
                    get_library(v.as_str()).map(|_| ()).ok_or(format!(
                        "library {} is not available.\nAvailable values: {}",
                        v,
                        AVAILABLE_LIBRARIES
                            .iter()
                            .filter(|t| t.1.is_some())
                            .map(|t| t.0)
                            .collect::<Vec<&str>>()
                            .join(", ")
                    ))
                }),
        )
        .arg(
            Arg::with_name("file")
                .help("Sets the input file to use.")
                .long_help("Sets the input files to use. By default read from stdin and write to stdout.\nThe output file is determined this way:\n - <file> plus <suffix> when compressing\n - <file> with the <suffix> removed when decompressing")
                .multiple(true),
        );

    let matches = app.get_matches();

    let mode = match (
        matches.is_present("compress"),
        matches.is_present("decompress"),
        matches.is_present("list"),
        matches.is_present("test"),
    ) {
        (_, false, false, false) => Mode::Compress {
            block_size: match matches
                .value_of("blocksize")
                .map(str::parse::<usize>)
                .transpose()
            {
                Ok(b) => b,
                Err(_) => return Err("could not parse blocksize"),
            },
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
        .collect::<Result<Vec<_>, _>>()?;
    let lz4jb_context = matches
        .value_of("library")
        .map(get_library)
        .flatten()
        .unwrap_or_default();
    Ok(Arguments {
        files: if files.is_empty() {
            vec![Files::stdio()]
        } else {
            files
        },
        mode,
        keep_input,
        force,
        lz4jb_context,
    })
}
