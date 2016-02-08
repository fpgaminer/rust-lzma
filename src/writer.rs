//! This module implements `LZMAWriter`.
//!
//! `LZMAWriter` implements the LZMA (XZ) compression/decompression algorithm as a generic
//! Writer.  In other words, it behaves similar to BufWriter.  Instead of buffering the `Write`
//! object passed to it, `LZMAWriter` applies compression or decompression to it as it's write.
//!
//!
//! # Examples
//!
//! ```no_run
//! use lzma::LZMAWriter;
//! use std::io::prelude::*;
//! use std::fs::File;
//!
//! let f = File::create("foo.xz").unwrap();
//! let mut f = LZMAWriter::new_compressor(f, 6).unwrap();
//!
//! write!(f, "It's a small world!").unwrap();
//! f.finish().unwrap();
//! ```

use std::io::{self, Write};
use std::ops::Drop;
use lzma_sys::*;
use std;
use error::{LZMAError, LZMALibResult};
use ::Direction;


const DEFAULT_BUF_SIZE: usize = 4 * 1024;


pub struct LZMAWriter<T> {
	inner: T,
	stream: lzma_stream,
	buffer: Vec<u8>,
	direction: Direction,
}


impl<T: Write> LZMAWriter<T> {
	pub fn new_compressor(inner: T, preset: u32) -> Result<LZMAWriter<T>, LZMAError> {
		LZMAWriter::with_capacity(DEFAULT_BUF_SIZE, inner, Direction::Compress, preset)
	}

	pub fn new_decompressor(inner: T) -> Result<LZMAWriter<T>, LZMAError> {
		LZMAWriter::with_capacity(DEFAULT_BUF_SIZE, inner, Direction::Decompress, 0)
	}

	pub fn with_capacity(capacity: usize, inner: T, direction: Direction, preset: u32) -> Result<LZMAWriter<T>, LZMAError> {
		let mut writer = LZMAWriter {
			inner: inner,
			stream: lzma_stream::new(),
			buffer: vec![0; capacity],
			direction: direction,
		};

		match writer.direction {
			Direction::Compress => {
				unsafe {
					try!(LZMALibResult::from(lzma_easy_encoder(&mut writer.stream, preset, lzma_check::LZMA_CHECK_CRC64)).map(|_| ()));
				}
			},
			Direction::Decompress => {
				unsafe {
					try!(LZMALibResult::from(lzma_stream_decoder(&mut writer.stream, std::u64::MAX, 0)).map(|_| ()));
				}
			},
		}

		Ok(writer)
	}
}

impl<T> Drop for LZMAWriter<T> {
	fn drop(&mut self) {
		unsafe {
			lzma_end(&mut self.stream);
		}
	}
}


impl<W: Write> LZMAWriter<W> {
	/// Finalizes the LZMA stream so that it finishes compressing or decompressing.
	///
	/// This *must* be called after all writing is done to ensure the last pieces of the compressed
	/// or decompressed stream get written out.
	pub fn finish(&mut self) -> Result<(), LZMAError> {
		self.stream.avail_in = 0;

		loop {
			match self.lzma_code_and_write(lzma_action::LZMA_FINISH) {
				Ok(lzma_ret::LZMA_STREAM_END) => break,
				Ok(_) => (),
				Err(err) => return Err(err),
			}
		}

		Ok(())
	}

	fn lzma_code_and_write(&mut self, action: lzma_action) -> Result<lzma_ret, LZMAError> {
		self.stream.next_out = self.buffer.as_mut_ptr();
		self.stream.avail_out = self.buffer.capacity();

		let ret = unsafe {
			try!(LZMALibResult::from(lzma_code(&mut self.stream, action)))
		};

		let written = self.buffer.capacity() - self.stream.avail_out;

		if written > 0 {
			unsafe {
				self.buffer.set_len(written);
			}

			try!(Write::write_all(&mut self.inner, &self.buffer));
		}

		Ok(ret)
	}
}


impl<W: Write> Write for LZMAWriter<W> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.stream.next_in = buf.as_ptr();
		self.stream.avail_in = buf.len();

		match self.lzma_code_and_write(lzma_action::LZMA_RUN) {
			Ok(_) => (),
			Err(LZMAError::Io(err)) => return Err(err),
			Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
		}

		Ok(buf.len() - self.stream.avail_in)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner.flush()
	}
}
