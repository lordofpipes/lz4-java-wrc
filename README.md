# lz4-java-wrc

[![Crate](https://img.shields.io/crates/v/lz4-java-wrc.svg)](https://crates.io/crates/lz4-java-wrc)
[![API](https://docs.rs/lz4-java-wrc/badge.svg)](https://docs.rs/lz4-java-wrc)

This is a fork of `lz4jb` that accepts the writer as a pointer instead of consuming it.

A streaming compression/decompression library which implements the `LZ4BlockOutputStream` format from [lz4-java](https://github.com/lz4/lz4-java).

**Beware**: this format is not compatible with the standard [LZ4 Block format](https://github.com/lz4/lz4/blob/dev/doc/lz4_Block_format.md). The Minecraft 1.20.5 lz4 chunk compression format is an example of a place this is used.

This repository contains:

- `lz4_java_wrc`: a library which implements the `Read` and `Write` traits,

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
lz4-java-wrc = "0.2.0"
```

### Compression

`Lz4BlockOutput` is a wrapper around a type which implements the `Write` trait.

```rust
use lz4_java_wrc::Lz4BlockOutput;
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
use lz4_java_wrc::Lz4BlockInput;
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

## License

See the [LICENCE](LICENSE) file.
