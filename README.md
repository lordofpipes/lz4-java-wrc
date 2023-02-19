# lz4jb

[![Crate](https://img.shields.io/crates/v/lz4jb.svg)](https://crates.io/crates/lz4jb)
[![API](https://docs.rs/lz4jb/badge.svg)](https://docs.rs/lz4jb)

A streaming compression/decompression library which implements the `LZ4BlockOutputStream` format from [lz4-java](https://github.com/lz4/lz4-java).

**Beware**: this format is not compatible with the standard [LZ4 Block format](https://github.com/lz4/lz4/blob/dev/doc/lz4_Block_format.md). You should not use it unless you have some historical data compressed using the Java code.

This repository contains:

- `lz4jb`: a library which implements the `Read` and `Write` traits,
- a command line tool to compress/decompress data in this format. The parameters are similar to `gzip`,

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
lz4jb = "0.1.0"
```

### Compression

`Lz4BlockOutput` is a wrapper around a type which implements the `Write` trait.

```rust
use lz4jb::Lz4BlockOutput;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let mut output = Vec::new(); // Vec<u8> implements the Write trait
    Lz4BlockOutput::new(&mut output, 64)?
        .write_all("...".as_bytes())?;
    println!("{:?}", output);
    Ok(())
}
```

### Decompression

`Lz4BlockInput` is a wrapper around a type which implements the `Read` trait.

```rust
use lz4jb::Lz4BlockInput;
use std::io::Read;

const D: [u8; 24] = [
    76, 90, 52, 66, 108, 111, 99, 107, 16, 3, 0, 0, 0, 3, 0, 0, 0, 82, 228, 119, 6, 46, 46, 46,
];

fn main() -> std::io::Result<()> {
    let mut output = String::new();
    Lz4BlockInput::new(&D[..]) // &[u8] implements the Read trait
        .read_to_string(&mut output)?;
    println!("{}", output);
    Ok(())
}
```

### Command line

In the [cli](cli/) folder, there is a command line tool to compress and decompress using this library.

```bash
$ git clone https://github.com/trazfr/lz4jb
$ cd lz4jb
$ cargo install --path cli
...
$ lz4jb -h
lz4jb 0.1.0
A compression tool which implements the LZ4BlockOutputStream format from https://github.com/lz4/lz4-java.
This is not compatible with the standard LZ4 Block format.

USAGE:
    lz4jb [FLAGS] [OPTIONS] [file]...

FLAGS:
    -z, --compress      Compress. This is the default operation mode.
    -d, --decompress    Decompress. [aliases: uncompress]
    -l, --list          List compressed file contents.
    -t, --test          Test the integrity of compressed files.
    -f, --force         Force the compression or decompression.
    -k, --keep          Keep (don't delete) input files during compression or decompression.
    -c, --stdout        Write to the standard output.
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
    -b, --blocksize <blocksize>    Block size for compression in bytes (between 64 and 33554432).
    -E, --extension <extension>    Append this extension instead of the default lz4 for compression.
    -L, --library <library>        Use an alternative library. See --help for the list of available libraries.

ARGS:
    <file>...    Sets the input file to use.
```

## License

See the [LICENCE](LICENSE) file.
