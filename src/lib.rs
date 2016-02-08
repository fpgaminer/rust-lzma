//! This crate provides a simple interface to liblzma.  LZMA is more commonly known as XZ or 7zip,
//! (as in, files with the `.xz` or `.7z` file extension). LZMA compression is fast and aggressive,
//! compressing better than bzip2.  liblzma implements the XZ variant, so it can read and write
//! `.xz` files/streams.
//!
//! Two interfaces are provided.  `LZMAReader`/`LZMAWriter` are generic Readers and Writers that
//! can be composed with other `Read`/`Write` interfaces.  For example, wrap them around a `File`
//! and you can write data to a file while compressing it on the fly, or stream in an `xz` file
//! from disk.
//!
//! `compress`/`decompress` are easy to use functions for simple use cases.
//!
//! See the `LZMAReader` and `LZMAWriter` documentation for further details on that interface.
//! `compress` and `decompress` are documented here.
//!
//! # Examples
//!
//! ```
//! let test_string = "Like tears in rain".to_string();
//! let mut compressed = lzma::compress(&test_string.into_bytes(), 6).unwrap();
//! let decompressed = lzma::decompress(&mut compressed).unwrap();
//! let decompressed_str = String::from_utf8(decompressed).unwrap();
//!
//! assert_eq!("Like tears in rain", decompressed_str);
//! ```

extern crate libc;

pub enum Direction {
	Compress,
	Decompress,
}

pub mod lzma_sys;
pub mod reader;
pub mod writer;
pub mod error;

use std::io::{Write, Read, Cursor};
pub use reader::LZMAReader;
pub use writer::LZMAWriter;
pub use error::LZMAError;


pub const EXTREME_PRESET: u32 = (1 << 31);


/// Compress `buf` and return the result.
///
/// preset is [0-9] and corresponds to xz's presets.
/// Binary-or with EXTREME_PRESET for --extreme (e.g. 9 | EXTREME_PRESET).
pub fn compress(buf: &[u8], preset: u32) -> Result<Vec<u8>, LZMAError> {
	let mut output: Vec<u8> = Vec::new();

	{
		let mut pipe = try!(LZMAWriter::new_compressor(&mut output, preset));

		try!(pipe.write_all(buf));
		try!(pipe.finish());
	}

	Ok(output)
}


/// Decompress `buf` and return the result.
pub fn decompress(buf: &[u8]) -> Result<Vec<u8>, LZMAError> {
	let mut output: Vec<u8> = Vec::new();

	{
		let mut cursor = Cursor::new(buf);
		let mut pipe = try!(LZMAReader::new_decompressor(&mut cursor));

		try!(pipe.read_to_end(&mut output));
	}

	Ok(output)
}
