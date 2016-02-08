# rust-lzma [![Build Status](https://travis-ci.org/fpgaminer/rust-lzma.svg?branch=master)](https://travis-ci.org/fpgaminer/rust-lzma) [![Crates.io](https://img.shields.io/crates/v/rust-lzma.svg)](https://crates.io/crates/rust-lzma) #

[Documentation](https://fpgaminer.github.io/doc/lzma/)

This crate provides a simple interface to liblzma.  LZMA is more commonly known
as XZ or 7zip, (as in, files with the `.xz` or `.7z` file extension). LZMA
compression is fast and aggressive, compressing better than bzip2.  liblzma
implements the XZ variant, so it can read and write `.xz` files/streams.

Two interfaces are provided.  `LZMAReader`/`LZMAWriter` are generic Readers and
Writers that can be composed with other `Read`/`Write` interfaces.  For example,
wrap them around a `File` and you can write data to a file while compressing it
on the fly, or stream in an `xz` file from disk.

`compress`/`decompress` are easy to use functions for simple use cases.

See the documentation for details on usage.


## Example ##
Cargo.toml:
```toml
[dependencies]
rust-lzma = "0.1"
```
main.rs:
```Rust
extern crate lzma;

use lzma::LZMAWriter;
use std::io::prelude::*;
use std::fs::File;

fn main() {
	let f = File::create("foo.xz").unwrap();
	let mut f = LZMAWriter::new_compressor(f, 6).unwrap();

	write!(f, "It's a small world!").unwrap();
	f.finish().unwrap();
}
```
