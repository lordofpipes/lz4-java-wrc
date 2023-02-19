use clap::{arg, command, value_parser, ArgAction};
use lz4jb::Context as Lz4Context;

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};

const DEFAULT_EXTENSION: &str = "lz4";

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
    Filename(PathBuf),
    Stdio,
    None,
}

impl FileDesc {
    fn decompressed(
        compressed_name: &Path,
        extension: &OsStr,
        to_stdout: bool,
    ) -> Result<Self, &'static str> {
        if to_stdout {
            Ok(Self::Stdio)
        } else {
            Option::from(compressed_name)
                .filter(|p| p.extension() == Some(extension))
                .and_then(Path::file_stem)
                .map(PathBuf::from)
                .map(Self::Filename)
                .map(Ok)
                .unwrap_or_else(|| Err("Could not guess the output filename"))
        }
    }
    fn compressed(
        decompressed_name: &Path,
        extension: &OsStr,
        to_stdout: bool,
    ) -> Result<Self, &'static str> {
        if to_stdout {
            Ok(Self::Stdio)
        } else {
            let mut compressed_name = decompressed_name.as_os_str().to_os_string();
            compressed_name.push(".");
            compressed_name.push(extension);
            Ok(Self::Filename(PathBuf::from(compressed_name)))
        }
    }
}
impl fmt::Display for FileDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Filename(filename) => write!(f, "filename={:?}", filename),
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

fn get_library(name: &String) -> Option<Lz4Context> {
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
    let matches = command!("lz4jb")
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(
            arg!(-z --compress "Compress. This is the default operation mode.")
                .conflicts_with_all(&["decompress", "list", "test"])
                .display_order(1),
        )
        .arg(
            arg!(-d --decompress "Decompress.")
                .visible_alias("uncompress")
                .conflicts_with_all(&["compress", "list", "test"])
                .display_order(1),
        )
        .arg(
            arg!(-l --list "List compressed file contents.")
                .conflicts_with_all(&["compress", "decompress", "test"])
                .display_order(1),
        )
        .arg(
            arg!(-t --test "Test the integrity of compressed files.")
                .conflicts_with_all(&["compress", "decompress", "list"])
                .display_order(1),
        )
        .arg(
            arg!(-c --stdout "Write output on standard output; keep original files unchanged.")
                .conflicts_with_all(&["list", "test"])
                .display_order(100),
        )
        .arg(
            arg!(-k --keep "Keep (don't delete) input files during compression or decompression.")
                .conflicts_with_all(&["list", "test"])
                .display_order(100),
        )
        .arg(
            arg!(-f --force "Force the compression or decompression.")
                .conflicts_with_all(&["list", "test"])
                .display_order(100),
        )
        .arg(
            arg!(-E --extension <VALUE> "Append this extension instead of the default lz4 for compression.")
                .conflicts_with_all(&["list", "test"])
                .display_order(100),
        )
        .arg(
            arg!(-b --blocksize <VALUE> "Block size for compression in bytes (between 64 and 33554432).")
                .conflicts_with_all(&["decompress", "list", "test"])
                .display_order(100),
        )
        .arg(
            arg!(-L --library <VALUE> "Use an alternative library. See --help for more information.")
                .long_help(library_long_help)
                .value_parser(AVAILABLE_LIBRARIES
                    .iter()
                    .filter(|t| t.1.is_some())
                    .map(|t| t.0)
                    .collect::<Vec<&str>>())
        )
        .arg(
            arg!([file] "Sets the input file to use.")
                .value_parser(value_parser!(PathBuf))
                .long_help("Sets the input files to use. By default read from stdin and write to stdout.\nThe output file is determined this way:\n - <file>.<extension> when compressing\n - <file> with the .<extension> removed when decompressing")
                .action(ArgAction::Append),
        )
        .get_matches();

    let mode = match (
        matches.get_flag("compress"),
        matches.get_flag("decompress"),
        matches.get_flag("list"),
        matches.get_flag("test"),
    ) {
        (_, false, false, false) => Mode::Compress {
            block_size: matches.get_one::<usize>("blocksize").cloned(),
        },
        (false, true, false, false) => Mode::Decompress,
        (false, false, true, false) => Mode::List,
        (false, false, false, true) => Mode::Test,
        (a, b, c, d) => {
            println!("{} {} {} {}", a, b, c, d);
            return Err(
            "Maximum 1 amongst the following arguments: --compress, --decompress, --list, --test",
        );
        }
    };

    let extension = matches
        .get_one::<OsString>("extension")
        .map(|ext| ext.as_ref())
        .unwrap_or_else(|| OsStr::new(DEFAULT_EXTENSION));
    let to_stdout = matches.get_flag("stdout");
    let keep_input = matches.get_flag("keep");
    let force = matches.get_flag("force");
    let files = matches
        .get_many::<PathBuf>("file")
        .unwrap_or_default()
        .map(|f| {
            Ok(Files {
                file_in: FileDesc::Filename(f.into()),
                file_out: match mode {
                    Mode::Compress { block_size: _ } => {
                        FileDesc::compressed(f, extension, to_stdout)?
                    }
                    Mode::Decompress => FileDesc::decompressed(f, extension, to_stdout)?,
                    _ => FileDesc::None,
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let lz4jb_context = matches
        .get_one::<String>("library")
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

#[cfg(test)]
mod test_arguments {

    use super::FileDesc;
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn filedesc_decompressed_basic() {
        if let Ok(FileDesc::Filename(filename)) =
            FileDesc::decompressed(Path::new("filename.ext"), OsStr::new("ext"), false)
        {
            assert_eq!(filename.to_str(), Some("filename"));
        } else {
            panic!("Wrong output");
        }
    }

    #[test]
    fn filedesc_decompressed_double_compressed() {
        if let Ok(FileDesc::Filename(filename)) =
            FileDesc::decompressed(Path::new("filename.ext.ext"), OsStr::new("ext"), false)
        {
            assert_eq!(filename.to_str(), Some("filename.ext"));
        } else {
            panic!("Wrong output");
        }
    }

    #[test]
    fn filedesc_decompressed_bad_extension() {
        assert!(matches!(
            FileDesc::decompressed(Path::new("filename.bad"), OsStr::new("ext"), false),
            Err(_)
        ));
    }

    #[test]
    fn filedesc_compressed_basic() {
        if let Ok(FileDesc::Filename(filename)) =
            FileDesc::compressed(Path::new("filename"), OsStr::new("ext"), false)
        {
            assert_eq!(filename.to_str(), Some("filename.ext"));
        } else {
            panic!("Wrong output");
        }
    }

    #[test]
    fn filedesc_compressed_double_compress() {
        if let Ok(FileDesc::Filename(filename)) =
            FileDesc::compressed(Path::new("filename.ext"), OsStr::new("ext"), false)
        {
            assert_eq!(filename.to_str(), Some("filename.ext.ext"));
        } else {
            panic!("Wrong output");
        }
    }
}
