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
			_ => write!(f, "{}", self.to_string()),
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

	fn cause(&self) -> Option<&dyn error::Error> {
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
			lzma_ret::LzmaOk => Ok(ret),
			lzma_ret::LzmaStreamEnd => Ok(ret),
			lzma_ret::LzmaNoCheck => Ok(ret),
			lzma_ret::LzmaUnsupportedCheck => Ok(ret), // NOTE: This is an error in some cases.  Not sure how to handle properly.
			lzma_ret::LzmaGetCheck => Ok(ret),
			lzma_ret::LzmaMemError => Err(LzmaError::Mem),
			lzma_ret::LzmaMemlimitError => Err(LzmaError::MemLimit),
			lzma_ret::LzmaFormatError => Err(LzmaError::Format),
			lzma_ret::LzmaOptionsError => Err(LzmaError::Options),
			lzma_ret::LzmaDataError => Err(LzmaError::Data),
			lzma_ret::LzmaBufError => Err(LzmaError::Buf),
			_ => Err(LzmaError::Other),
		}
	}
}
