use libc;
use std::mem;


#[repr(C)]
pub struct lzma_stream {
	pub next_in: *const u8,
    pub avail_in: usize,
    pub total_in: u64,

    pub next_out: *mut u8,
    pub avail_out: usize,
    pub total_out: u64,

    pub allocator: *const lzma_allocator,

    pub internal: *mut libc::c_void,   /* Actually lzma_internal, but lzma_internal is opaque */

    pub reserved_ptr1: *mut libc::c_void,
    pub reserved_ptr2: *mut libc::c_void,
    pub reserved_ptr3: *mut libc::c_void,
    pub reserved_ptr4: *mut libc::c_void,
    pub reserved_int1: *mut u64,
    pub reserved_int2: *mut u64,
    pub reserved_int3: *mut usize,
    pub reserved_int4: *mut usize,
    pub reserved_enum1: u32,  // TODO: in base.h this is lzma_reserved_enum, which seems to compile to 4-bytes
    pub reserved_enum2: u32,  // TODO: in base.h this is lzma_reserved_enum, which seems to compile to 4-bytes
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
    pub alloc: *mut extern fn(opaque: *mut libc::c_void, nmemb: usize, size: usize),
    pub free: extern fn(opaque: *mut libc::c_void, ptr: *mut libc::c_void),
    pub opaque: *mut libc::c_void,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[must_use]
pub enum lzma_ret {
    LZMA_OK                  = 0,
    LZMA_STREAM_END          = 1,
    LZMA_NO_CHECK            = 2,
    LZMA_UNSUPPORTED_CHECK   = 3,
    LZMA_GET_CHECK           = 4,
    LZMA_MEM_ERROR           = 5,
    LZMA_MEMLIMIT_ERROR      = 6,
    LZMA_FORMAT_ERROR        = 7,
    LZMA_OPTIONS_ERROR       = 8,
    LZMA_DATA_ERROR          = 9,
    LZMA_BUF_ERROR           = 10,
    LZMA_PROG_ERROR          = 11,
}


#[repr(C)]
#[derive(Clone, Copy)]
pub enum lzma_action {
	LZMA_RUN           = 0,
	LZMA_SYNC_FLUSH    = 1,
	LZMA_FULL_FLUSH    = 2,
	LZMA_FULL_BARRIER  = 4,
	LZMA_FINISH        = 3,
}


#[repr(C)]
#[derive(Clone, Copy)]
pub enum lzma_check {
	LZMA_CHECK_NONE     = 0,
    LZMA_CHECK_CRC32    = 1,
    LZMA_CHECK_CRC64    = 4,
    LZMA_CHECK_SHA256   = 10,
}


pub const LZMA_CONCATENATED: u32            = 0x08;
pub const LZMA_TELL_NO_CHECK: u32           = 0x01;
pub const LZMA_TELL_UNSUPPORTED_CHECK: u32  = 0x02;
pub const LZMA_TELL_ANY_CHECK: u32          = 0x04;


extern {
	pub fn lzma_easy_encoder(stream: *mut lzma_stream, preset: u32, check: lzma_check) -> lzma_ret;
	pub fn lzma_code(stream: *mut lzma_stream, action: lzma_action) -> lzma_ret;
	pub fn lzma_end(stream: *mut lzma_stream);
	pub fn lzma_stream_decoder(stream: *mut lzma_stream, memlimit: u64, flags: u32) -> lzma_ret;
}
