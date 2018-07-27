//! This module implements `LzmaWriter`.
//!
//! `LzmaWriter` implements the LZMA (XZ) compression/decompression algorithm as a generic
//! Writer.  In other words, it behaves similar to BufWriter.  Instead of buffering the `Write`
//! object passed to it, `LzmaWriter` applies compression or decompression to it as it's write.
//!
//!
//! # Examples
//!
//! ```no_run
//! use lzma::LzmaWriter;
//! use std::io::prelude::*;
//! use std::fs::File;
//!
//! let f = File::create("foo.xz").unwrap();
//! let mut f = LzmaWriter::new_compressor(f, 6).unwrap();
//!
//! write!(f, "It's a small world!").unwrap();
//! f.finish().unwrap();
//! ```

use std::io::{self, Write};
use lzma_sys::*;
use std;
use error::LzmaError;
use ::Direction;
use lzma_stream_wrapper::{LzmaStreamWrapper, LzmaCodeResult};


const DEFAULT_BUF_SIZE: usize = 4 * 1024;


pub struct LzmaWriter<T> {
	inner: T,
	stream: LzmaStreamWrapper,
	buffer: Vec<u8>,
	direction: Direction,
}


impl<T: Write> LzmaWriter<T> {
	pub fn new_compressor(inner: T, preset: u32) -> Result<LzmaWriter<T>, LzmaError> {
		LzmaWriter::with_capacity(DEFAULT_BUF_SIZE, inner, Direction::Compress, preset)
	}

	pub fn new_decompressor(inner: T) -> Result<LzmaWriter<T>, LzmaError> {
		LzmaWriter::with_capacity(DEFAULT_BUF_SIZE, inner, Direction::Decompress, 0)
	}

	pub fn with_capacity(capacity: usize, inner: T, direction: Direction, preset: u32) -> Result<LzmaWriter<T>, LzmaError> {
		let mut writer = LzmaWriter {
			inner: inner,
			stream: LzmaStreamWrapper::new(),
			buffer: vec![0; capacity],
			direction: direction,
		};

		match writer.direction {
			Direction::Compress => {
				try!(writer.stream.easy_encoder(preset, lzma_check::LzmaCheckCrc64))
			},
			Direction::Decompress => {
				try!(writer.stream.stream_decoder(std::u64::MAX, 0))
			},
		}

		Ok(writer)
	}
}

impl<W: Write> LzmaWriter<W> {
	/// Finalizes the LZMA stream so that it finishes compressing or decompressing.
	///
	/// This *must* be called after all writing is done to ensure the last pieces of the compressed
	/// or decompressed stream get written out.
	pub fn finish(mut self) -> Result<W, LzmaError> {
		loop {
			match self.lzma_code_and_write(&[], lzma_action::LzmaFinish) {
				Ok(LzmaCodeResult {
					ret: Ok(lzma_ret::LzmaStreamEnd),
					bytes_read: _,
					bytes_written: _,
				}) => break,
				Ok(_) => continue,
				Err(err) => return Err(err),
			}
		}

		self.stream.end();

		Ok(self.inner)
	}

	fn lzma_code_and_write(&mut self, input: &[u8], action: lzma_action) -> Result<LzmaCodeResult, LzmaError> {
		let result = self.stream.code(input, &mut self.buffer, action);
		let _ = try!(result.ret);

		if result.bytes_written > 0 {
			try!(Write::write_all(&mut self.inner, &self.buffer[..result.bytes_written]));
		}

		Ok(result)
	}
}


impl<W: Write> Write for LzmaWriter<W> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		// Loop until at least one byte from buf was consumed in order to be
		// compliant with std::io::Write trait API.
		loop {
			match self.lzma_code_and_write(buf, lzma_action::LzmaRun) {
				Ok(result) => if result.bytes_read == 0 && result.bytes_written > 0 {
					continue
				} else {
					// If result.bytes_read is zero, then neither was something
					// written nor read. This indicates, something went wrong.
					return Ok(result.bytes_read)
				},
				Err(LzmaError::Io(err)) => return Err(err),
				Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
			}
		}
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner.flush()
	}
}
