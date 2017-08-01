use lzma_sys::*;
use std::convert::From;
use std::result::Result;
use std::error;
use std::fmt;
use std::error::Error as StdError;
use std::io::Error as IoError;


/// An error produced by an operation on LZMA data
#[derive(Debug)]
pub enum LzmaError {
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

impl fmt::Display for LzmaError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			LzmaError::Io(ref err) => write!(f, "{}", err),
			_ => write!(f, "{}", self.description()),
		}
	}
}

impl StdError for LzmaError {
	fn description(&self) -> &str {
		match *self {
			LzmaError::Mem => "Memory allocation failed",
			LzmaError::MemLimit => "Memory limit would be violated",
			LzmaError::Format => "XZ magic bytes were not found",
			LzmaError::Options => "Unsupported compression options",
			LzmaError::Data => "Corrupt data",
			LzmaError::Buf => "Data look like it was truncated or possibly corrupt",
			LzmaError::Io(..) => "IO error",
			LzmaError::Other => "Unknown error",
		}
	}

	fn cause(&self) -> Option<&error::Error> {
		match *self {
			LzmaError::Io(ref err) => Some(err),
			_ => None,
		}
	}
}

impl From<IoError> for LzmaError {
	fn from(err: IoError) -> LzmaError {
		LzmaError::Io(err)
	}
}


/* Return values from liblzma are converted into this for easier handling */
pub type LzmaLibResult = Result<lzma_ret, LzmaError>;

impl From<lzma_ret> for LzmaLibResult {
	fn from(ret: lzma_ret) -> LzmaLibResult {
		match ret {
			lzma_ret::LZMA_OK => Ok(ret),
			lzma_ret::LZMA_STREAM_END => Ok(ret),
			lzma_ret::LZMA_NO_CHECK => Ok(ret),
			lzma_ret::LZMA_UNSUPPORTED_CHECK => Ok(ret), // NOTE: This is an error in some cases.  Not sure how to handle properly.
			lzma_ret::LZMA_GET_CHECK => Ok(ret),
			lzma_ret::LZMA_MEM_ERROR => Err(LzmaError::Mem),
			lzma_ret::LZMA_MEMLIMIT_ERROR => Err(LzmaError::MemLimit),
			lzma_ret::LZMA_FORMAT_ERROR => Err(LzmaError::Format),
			lzma_ret::LZMA_OPTIONS_ERROR => Err(LzmaError::Options),
			lzma_ret::LZMA_DATA_ERROR => Err(LzmaError::Data),
			lzma_ret::LZMA_BUF_ERROR => Err(LzmaError::Buf),
			_ => Err(LzmaError::Other),
		}
	}
}
