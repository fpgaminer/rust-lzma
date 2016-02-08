use std::io::{Write};
use lzma_sys::*;
use std::convert::From;
use std::result::Result;
use std::error;
use std::fmt;
use std::error::Error as StdError;
use std::io::Error as IoError;


/// An error produced by an operation on LZMA data
#[derive(Debug)]
pub enum LZMAError {
	/// Failed Memory Allocation
    Mem,
	/// Memory limit would be violated
	MemLimit,
	/// XZ magic bytes weren't found
	Format,
	/// Unsupported compression options
	Options,
	/// Corrupt data
	Data,
	/// Data looks truncated
	Buf,
	/// std::io::Error
	Io(IoError),
	/// An unknown error
	Other,
}

impl fmt::Display for LZMAError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			LZMAError::Io(ref err) => write!(f, "{}", err),
			_ => write!(f, "{}", self.description()),
		}
	}
}

impl StdError for LZMAError {
	fn description(&self) -> &str {
		match *self {
			LZMAError::Mem => "Memory allocation failed",
			LZMAError::MemLimit => "Memory limit would be violated",
			LZMAError::Format => "XZ magic bytes were not found",
			LZMAError::Options => "Unsupported compression options",
			LZMAError::Data => "Corrupt data",
			LZMAError::Buf => "Data look like it was truncated or possibly corrupt",
			LZMAError::Io(..) => "IO error",
			LZMAError::Other => "Unknown error",
		}
	}

	fn cause(&self) -> Option<&error::Error> {
		match *self {
			LZMAError::Io(ref err) => Some(err),
			_ => None,
		}
	}
}

impl From<IoError> for LZMAError {
	fn from(err: IoError) -> LZMAError {
		LZMAError::Io(err)
	}
}


/* Return values from liblzma are converted into this for easier handling */
pub type LZMALibResult = Result<lzma_ret, LZMAError>;

impl From<lzma_ret> for LZMALibResult {
	fn from(ret: lzma_ret) -> LZMALibResult {
		match ret {
			lzma_ret::LZMA_OK => Ok(ret),
			lzma_ret::LZMA_STREAM_END => Ok(ret),
			lzma_ret::LZMA_NO_CHECK => Ok(ret),
			lzma_ret::LZMA_UNSUPPORTED_CHECK => Ok(ret), // NOTE: This is an error in some cases.  Not sure how to handle properly.
			lzma_ret::LZMA_GET_CHECK => Ok(ret),
			lzma_ret::LZMA_MEM_ERROR => Err(LZMAError::Mem),
			lzma_ret::LZMA_MEMLIMIT_ERROR => Err(LZMAError::MemLimit),
			lzma_ret::LZMA_FORMAT_ERROR => Err(LZMAError::Format),
			lzma_ret::LZMA_OPTIONS_ERROR => Err(LZMAError::Options),
			lzma_ret::LZMA_DATA_ERROR => Err(LZMAError::Data),
			lzma_ret::LZMA_BUF_ERROR => Err(LZMAError::Buf),
			_ => Err(LZMAError::Other),
		}
	}
}
