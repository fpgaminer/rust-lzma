//! This module implements `LZMAReader`.
//!
//! `LZMAReader` implements the LZMA (XZ) compression/decompression algorithm as a generic
//! Reader.  In other words, it behaves similar to BufReader.  Instead of buffering the `Read`
//! object passed to it, `LZMAReader` applies compression or decompression to it as it's read.
//!
//!
//! # Examples
//!
//! ```no_run
//! use lzma::LZMAReader;
//! use std::io::prelude::*;
//! use std::fs::File;
//!
//! let f = File::open("foo.xz").unwrap();
//! let mut f = LZMAReader::new_decompressor(f).unwrap();
//! let mut s = String::new();
//!
//! f.read_to_string(&mut s).unwrap();
//! println!("{}", s);
//! ```

use std::io::{self, Read};
use std::ops::Drop;
use lzma_sys::*;
use std;
use error::{LZMAError, LZMALibResult};
use ::Direction;


const DEFAULT_BUF_SIZE: usize = 4 * 1024;


pub struct LZMAReader<T> {
	inner: T,
	stream: lzma_stream,
	buffer: Vec<u8>,
	direction: Direction,
}


impl<T: Read> LZMAReader<T> {
	pub fn new_compressor(inner: T, preset: u32) -> Result<LZMAReader<T>, LZMAError> {
		LZMAReader::with_capacity(DEFAULT_BUF_SIZE, inner, Direction::Compress, preset)
	}

	pub fn new_decompressor(inner: T) -> Result<LZMAReader<T>, LZMAError> {
		LZMAReader::with_capacity(DEFAULT_BUF_SIZE, inner, Direction::Decompress, 0)
	}

	pub fn with_capacity(capacity: usize, inner: T, direction: Direction, preset: u32) -> Result<LZMAReader<T>, LZMAError> {
		let mut reader = LZMAReader {
			inner: inner,
			stream: lzma_stream::new(),
			buffer: vec![0; capacity],
			direction: direction,
		};

		match reader.direction {
			Direction::Compress => {
				unsafe {
					try!(LZMALibResult::from(lzma_easy_encoder(&mut reader.stream, preset, lzma_check::LZMA_CHECK_CRC64)).map(|_| ()));
				}
			},
			Direction::Decompress => {
				unsafe {
					try!(LZMALibResult::from(lzma_stream_decoder(&mut reader.stream, std::u64::MAX, 0)).map(|_| ()));
				}
			},
		}

		Ok(reader)
	}
}

impl<T> Drop for LZMAReader<T> {
	fn drop(&mut self) {
		unsafe {
			lzma_end(&mut self.stream);
		}
	}
}


impl<R: Read> Read for LZMAReader<R> {
	/// Reads data from the wrapped object, applies compression/decompression, and puts the results
	/// into buf.
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		/* Our code doesn't handle buf.len() being 0, so exit early */
		if buf.len() == 0 {
			return Ok(0);
		}

		self.stream.next_out = buf.as_mut_ptr();
		self.stream.avail_out = buf.len();

		loop {
			let mut action = lzma_action::LZMA_RUN;

			if self.stream.avail_in == 0 {
				self.stream.next_in = self.buffer.as_ptr();
				self.stream.avail_in = try!(self.inner.read(&mut self.buffer));

				if self.stream.avail_in == 0 {
					action = lzma_action::LZMA_FINISH;
				}
			}

			let stream_end = unsafe {
				match LZMALibResult::from(lzma_code(&mut self.stream, action)) {
					Ok(lzma_ret::LZMA_STREAM_END) => true,
					Ok(_) => false,
					Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
				}
			};

			let bytes_read = buf.len() - self.stream.avail_out;

			/* We have to loop until we get at least 1 byte or EOF, because most users of
			 * Read::read assume that a return value of 0 is EOF. */
			if stream_end || bytes_read > 0 {
				return Ok(bytes_read);
			}
		}
	}
}
