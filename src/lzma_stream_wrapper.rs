//! Wraps the underlying FFI struct `lzma_stream` to provide various safety guarantees, like the Send trait.

use lzma_sys::*;
use error::{LzmaError, LzmaLibResult};
use std::ptr;
use std::ops::Drop;


pub struct LzmaStreamWrapper {
	stream: lzma_stream,
}

pub struct LzmaCodeResult {
	/// The return value of lzma_code
	pub ret: LzmaLibResult,
	/// The number of bytes read from input
	pub bytes_read: usize,
	/// The number of bytes written to output
	pub bytes_written: usize,
}


// I believe liblzma is at least Send thread safe, though using it like that will result in
// malloc being called in one thread and free being called in another.  That's usually safe,
// but depends on how liblzma was compiled.
unsafe impl Send for LzmaStreamWrapper {}


impl LzmaStreamWrapper {
	pub fn new() -> LzmaStreamWrapper {
		LzmaStreamWrapper {
			stream: lzma_stream::new(),
		}
	}

	pub fn easy_encoder(&mut self, preset: u32, check: lzma_check) -> Result<(), LzmaError> {
		unsafe {
			LzmaLibResult::from(lzma_easy_encoder(&mut self.stream, preset, check)).map(|_| ())
		}
	}

	pub fn stream_decoder(&mut self, memlimit: u64, flags: u32) -> Result<(), LzmaError> {
		unsafe {
			LzmaLibResult::from(lzma_auto_decoder(&mut self.stream, memlimit, flags)).map(|_| ())
		}
	}

	pub fn end(&mut self) {
		unsafe {
			lzma_end(&mut self.stream)
		}
	}

	/// Pointers to input and output are given to liblzma during execution of this function,
	/// but they are removed before returning.  So that should keep everything safe.
	pub fn code(&mut self, input: &[u8], output: &mut [u8], action: lzma_action) -> LzmaCodeResult {
		// Prepare lzma_stream
		self.stream.next_in = input.as_ptr();
		self.stream.avail_in = input.len();
		self.stream.next_out = output.as_mut_ptr();
		self.stream.avail_out = output.len();

		// Execute lzma_code and get results
		let ret = unsafe {
			LzmaLibResult::from(lzma_code(&mut self.stream, action))
		};
		let bytes_read = input.len() - self.stream.avail_in;
		let bytes_written = output.len() - self.stream.avail_out;

		// Clear pointers from lzma_stream
		self.stream.next_in = ptr::null();
		self.stream.avail_in = 0;
		self.stream.next_out = ptr::null_mut();
		self.stream.avail_out = 0;

		LzmaCodeResult {
			ret,
			bytes_read,
			bytes_written,
		}
	}
}

// This makes sure to call lzma_end, which frees memory that liblzma has allocated internally
// Note: It appears to be safe to call lzma_end multiple times; so this Drop is safe
// even if the user has already called end.
impl Drop for LzmaStreamWrapper {
	fn drop(&mut self) {
		self.end();
	}
}