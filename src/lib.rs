//! This crate provides a simple interface to liblzma.  LZMA is more commonly known as XZ or 7zip,
//! (as in, files with the `.xz` or `.7z` file extension). LZMA compression is fast and aggressive,
//! compressing better than bzip2.  liblzma implements the XZ variant, so it can read and write
//! `.xz` files/streams.
//!
//! Two interfaces are provided.  `LzmaReader`/`LzmaWriter` are generic Readers and Writers that
//! can be composed with other `Read`/`Write` interfaces.  For example, wrap them around a `File`
//! and you can write data to a file while compressing it on the fly, or stream in an `xz` file
//! from disk.
//!
//! `compress`/`decompress` are easy to use functions for simple use cases.
//!
//! See the `LzmaReader` and `LzmaWriter` documentation for further details on that interface.
//! `compress` and `decompress` are documented here.
//!
//! # Examples
//!
//! ```
//! let test_string = "Like tears in rain";
//! let mut compressed = lzma::compress(test_string.as_bytes(), 6).unwrap();
//! let decompressed = lzma::decompress(&mut compressed).unwrap();
//! let decompressed_str = String::from_utf8(decompressed).unwrap();
//!
//! assert_eq!(test_string, decompressed_str);
//! ```

pub enum Direction {
	Compress,
	Decompress,
}

mod lzma_sys;
mod lzma_stream_wrapper;
pub mod reader;
pub mod writer;
pub mod error;

use std::io::Read;
pub use reader::LzmaReader;
pub use writer::LzmaWriter;
pub use error::LzmaError;


pub const EXTREME_PRESET: u32 = 1 << 31;


/// Compress `buf` and return the result.
///
/// preset is [0-9] and corresponds to xz's presets.
/// Binary-or with EXTREME_PRESET for --extreme (e.g. 9 | EXTREME_PRESET).
pub fn compress(buf: &[u8], preset: u32) -> Result<Vec<u8>, LzmaError> {
	let mut output: Vec<u8> = Vec::new();

	{
		let mut reader = LzmaReader::new_compressor(buf, preset)?;

		reader.read_to_end(&mut output)?;
	}

	Ok(output)
}


/// Decompress `buf` and return the result.
pub fn decompress(buf: &[u8]) -> Result<Vec<u8>, LzmaError> {
	let mut output: Vec<u8> = Vec::new();

	{
		let mut reader = LzmaReader::new_decompressor(buf)?;

		reader.read_to_end(&mut output)?;
	}

	Ok(output)
}
