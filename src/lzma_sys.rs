use std::os::raw::c_void;
use std::mem;


#[repr(C)]
pub struct lzma_stream {
	/// < Pointer to the next input byte.
	pub next_in: *const u8,
	/// < Number of available input bytes in next_in.
	pub avail_in: usize,
	/// < Total number of bytes read by liblzma.
	pub total_in: u64,

	/// < Pointer to the next output position.
	pub next_out: *mut u8,
	/// < Amount of free space in next_out.
	pub avail_out: usize,
	/// < Total number of bytes written by liblzma.
	pub total_out: u64,

	/// \brief       Custom memory allocation functions
	///
	/// In most cases this is NULL which makes liblzma use
	/// the standard malloc() and free().
	///
	/// \note        In 5.0.x this is not a const pointer.
	pub allocator: *const lzma_allocator,

	/// Internal state is not visible to applications.
	pub internal: *mut c_void,    // Actually a pointer to lzma_internal, but lzma_internal is opaque

	pub reserved_ptr1: *mut c_void,
	pub reserved_ptr2: *mut c_void,
	pub reserved_ptr3: *mut c_void,
	pub reserved_ptr4: *mut c_void,
	pub reserved_int1: u64,
	pub reserved_int2: u64,
	pub reserved_int3: usize,
	pub reserved_int4: usize,
	pub reserved_enum1: u32,    // Actually an enum, but it's opaque so we stub with u32
	pub reserved_enum2: u32,    // Actually an enum, but it's opaque so we stub with u32
}

impl lzma_stream {
	// base.h defines LZMA_STREAM_INIT; we declare new instead.
	pub fn new() -> lzma_stream {
		unsafe {
			mem::zeroed()
		}
	}
}


#[repr(C)]
pub struct lzma_allocator {
	pub alloc: *mut extern fn(opaque: *mut c_void, nmemb: usize, size: usize),
	pub free: extern fn(opaque: *mut c_void, ptr: *mut c_void),
	pub opaque: *mut c_void,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[must_use]
pub enum lzma_ret {
	LzmaOk                  = 0,
	LzmaStreamEnd           = 1,
	LzmaNoCheck             = 2,
	LzmaUnsupportedCheck    = 3,
	LzmaGetCheck            = 4,
	LzmaMemError            = 5,
	LzmaMemlimitError       = 6,
	LzmaFormatError         = 7,
	LzmaOptionsError        = 8,
	LzmaDataError           = 9,
	LzmaBufError            = 10,
	LzmaProgError           = 11,
}


#[repr(C)]
#[derive(Clone, Copy)]
pub enum lzma_action {
	LzmaRun           = 0,
	LzmaSyncFlush     = 1,
	LzmaFullFlush     = 2,
	LzmaFullBarrier   = 4,
	LzmaFinish        = 3,
}


#[repr(C)]
#[derive(Clone, Copy)]
pub enum lzma_check {
	LzmaCheckNone      = 0,
	LzmaCheckCrc32     = 1,
	LzmaCheckCrc64     = 4,
	LzmaCheckSha256    = 10,
}


extern {
	pub fn lzma_easy_encoder(stream: *mut lzma_stream, preset: u32, check: lzma_check) -> lzma_ret;
	pub fn lzma_code(stream: *mut lzma_stream, action: lzma_action) -> lzma_ret;
	pub fn lzma_end(stream: *mut lzma_stream);
	pub fn lzma_stream_decoder(stream: *mut lzma_stream, memlimit: u64, flags: u32) -> lzma_ret;
}
