//! This module implements `LzmaReader`.
//!
//! `LzmaReader` implements the LZMA (XZ) compression/decompression algorithm as a generic
//! Reader.  In other words, it behaves similar to BufReader.  Instead of buffering the `Read`
//! object passed to it, `LzmaReader` applies compression or decompression to it as it's read.
//!
//!
//! # Examples
//!
//! ```no_run
//! use lzma::LzmaReader;
//! use std::io::prelude::*;
//! use std::fs::File;
//!
//! let f = File::open("foo.xz").unwrap();
//! let mut f = LzmaReader::new_decompressor(f).unwrap();
//! let mut s = String::new();
//!
//! f.read_to_string(&mut s).unwrap();
//! println!("{}", s);
//! ```

use std::io::{self, Read};
use lzma_sys::*;
use std;
use error::LzmaError;
use ::Direction;
use lzma_stream_wrapper::LzmaStreamWrapper;


const DEFAULT_BUF_SIZE: usize = 4 * 1024;


pub struct LzmaReader<T> {
	inner: T,
	stream: LzmaStreamWrapper,
	buffer: Vec<u8>,
	buffer_offset: usize,
	buffer_len: usize,
	direction: Direction,
}


impl<T: Read> LzmaReader<T> {
	pub fn new_compressor(inner: T, preset: u32) -> Result<LzmaReader<T>, LzmaError> {
		LzmaReader::with_capacity(DEFAULT_BUF_SIZE, inner, Direction::Compress, preset)
	}

	pub fn new_decompressor(inner: T) -> Result<LzmaReader<T>, LzmaError> {
		LzmaReader::with_capacity(DEFAULT_BUF_SIZE, inner, Direction::Decompress, 0)
	}

	pub fn with_capacity(capacity: usize, inner: T, direction: Direction, preset: u32) -> Result<LzmaReader<T>, LzmaError> {
		let mut reader = LzmaReader {
			inner: inner,
			stream: LzmaStreamWrapper::new(),
			buffer: vec![0; capacity],
			buffer_offset: 0,
			buffer_len: 0,
			direction: direction,
		};

		match reader.direction {
			Direction::Compress => {
				reader.stream.easy_encoder(preset, lzma_check::LzmaCheckCrc64)?
			},
			Direction::Decompress => {
				reader.stream.stream_decoder(std::u64::MAX, 0)?
			},
		}

		Ok(reader)
	}

	pub fn into_inner(self) -> T { self.inner }
}


impl<R: Read> Read for LzmaReader<R> {
	/// Reads data from the wrapped object, applies compression/decompression, and puts the results
	/// into buf.
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		// Our code doesn't handle buf.len() being 0, so exit early
		if buf.len() == 0 {
			return Ok(0);
		}

		loop {
			let mut action = lzma_action::LzmaRun;

			// If our internal read buffer is empty, re-fill it by calling read on the inner Read object.
			if self.buffer_len == 0 {
				self.buffer_offset = 0;
				self.buffer_len = self.inner.read(&mut self.buffer)?;

				if self.buffer_len == 0 {
					action = lzma_action::LzmaFinish;
				}
			}

			// Instruct liblzma to compress/decompress data from the buffer, and write the results to buf
			let result = self.stream.code(&self.buffer[self.buffer_offset..(self.buffer_offset+self.buffer_len)], buf, action);
			self.buffer_offset += result.bytes_read;
			self.buffer_len -= result.bytes_read;

			let stream_end = match result.ret {
				Ok(lzma_ret::LzmaStreamEnd) => true,
				Ok(_) => false,
				Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
			};

			// We have to loop until we get at least 1 byte or EOF, because most users of
		 	// Read::read assume that a return value of 0 is EOF.
			if stream_end || result.bytes_written > 0 {
				return Ok(result.bytes_written);
			}
		}
	}
}
