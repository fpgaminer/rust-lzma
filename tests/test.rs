extern crate lzma;

use lzma::error::LzmaError;
use std::io::{Read, Cursor, Write};
use std::thread;


// A large text file used for testing
const TEST_STRING: &'static str = include_str!("test_file.txt");
// Should be test_file.txt compressed in the legacy lzma format.
const TEST_LEGACY_DATA: &'static [u8] = include_bytes!("test_file.lzma");


#[test]
fn simple_compress_decompress() {
	let mut compressed = lzma::compress(&TEST_STRING.to_string().into_bytes(), 6).unwrap();
	let decompressed = String::from_utf8(lzma::decompress(&mut compressed).unwrap()).unwrap();

	assert!(compressed.len() < TEST_STRING.len());
	assert_eq!(TEST_STRING, decompressed);
}


#[test]
fn extreme() {
	let mut compressed = lzma::compress(&TEST_STRING.to_string().into_bytes(), 9).unwrap();
	let mut extreme_compressed = lzma::compress(&TEST_STRING.to_string().into_bytes(), 9 | lzma::EXTREME_PRESET).unwrap();
	let decompressed = String::from_utf8(lzma::decompress(&mut compressed).unwrap()).unwrap();
	let extreme_decompressed = String::from_utf8(lzma::decompress(&mut extreme_compressed).unwrap()).unwrap();

	// TODO: This test is not great.  We just want to know if the EXTREME_PRESET flag is working.
	// It might occur that the len's are equal because EXTREME wasn't able to compress more (or less).
	assert!(extreme_compressed.len() != compressed.len());
	assert_eq!(TEST_STRING, decompressed);
	assert_eq!(TEST_STRING, extreme_decompressed);
}


#[test]
fn reader_wormhole() {
	let compressor = lzma::LzmaReader::new_compressor(Cursor::new(TEST_STRING), 5).unwrap();
	let mut decompressor = lzma::LzmaReader::new_decompressor(compressor).unwrap();
	let mut s = String::new();

	decompressor.read_to_string(&mut s).unwrap();

	assert_eq!(TEST_STRING, s);
}


#[test]
fn writer_wormhole() {
	let mut output = vec![0u8; 0];
	{
		let decompressor = lzma::LzmaWriter::new_decompressor(&mut output).unwrap();
		let mut compressor = lzma::LzmaWriter::new_compressor(decompressor, 2).unwrap();

		write!(compressor, "{}", TEST_STRING).unwrap();
		compressor.finish().unwrap();
	}

	assert_eq!(TEST_STRING, String::from_utf8(output).unwrap());
}


#[test]
fn truncation_causes_error() {
	let mut compressed = lzma::compress(&"Like tears in rain".to_string().into_bytes(), 6).unwrap();
	let bad_len = compressed.len() - 1;
	compressed.truncate(bad_len);
	match lzma::decompress(&mut compressed) {
		Err(lzma::LzmaError::Io(err)) => {
			match *err.get_ref().unwrap().downcast_ref::<LzmaError>().unwrap() {
				LzmaError::Buf => (),
				_ => panic!("Decompressing a truncated buffer should return an LzmaError::Buf error"),
			}
		},
		_ => panic!("Decompressing a truncated buffer should return an LzmaError::Buf error"),
	}
}


// Test to make sure that LzmaReader implements the Send trait correctly
#[test]
fn reader_thread_send() {
	let compressor = lzma::LzmaReader::new_compressor(Cursor::new(TEST_STRING), 5).unwrap();
	let mut decompressor = lzma::LzmaReader::new_decompressor(compressor).unwrap();

	let output = thread::spawn(move || {
		let mut s = String::new();
		decompressor.read_to_string(&mut s).unwrap();
		s

	}).join().unwrap();

	assert_eq!(TEST_STRING, output);
}


// Decompressing the test string in here causes LZMA to return 0 (bytes consumed from input) occasionally.
// Good for testing that our Writer implementation handles that correctly.
#[test]
fn test_string_1() {
	let input = b"\xfd\x37\x7a\x58\x5a\x00\x00\x04\xe6\xd6\xb4\x46\x02\x00\x21\x01\x12\x00\x00\x00\x23\xb8\x87\x2c\xe1\x20\xff\x00\x4c\x5d\x00\x30\xef\xfb\xbf\xfe\xa3\xb1\x5e\xe5\xf8\x3f\xb2\xaa\x26\x55\xf8\x68\x70\x41\x70\x15\x0f\x8d\xfd\x1e\x4c\x1b\x8a\x42\xb7\x19\xf4\x69\x18\x71\xae\x66\x23\x8a\x8a\x4d\x2f\xa3\x0d\xd9\x7f\xa6\xe3\x8c\x23\x11\x53\xe0\x59\x18\xc5\x75\x8a\xe2\x77\xf8\xb6\x94\x7f\x0c\x6a\xc0\xde\x74\x49\x64\xe2\xe8\x22\xa0\xfd\x00\xb3\xfd\x27\xbc\xcd\x64\x62\xb1\x00\x01\x68\x80\xc2\x04\x00\x00\x29\x67\x59\x4d\xb1\xc4\x67\xfb\x02\x00\x00\x00\x00\x04\x59\x5a";
	let mut writer = lzma::LzmaWriter::new_decompressor(Vec::new()).unwrap();
	writer.write_all(input).unwrap();
	let buffer = writer.finish().unwrap();
	assert_eq!(buffer.len(), 73984);    // Original string is 73,984 * b'a'.
}


// Test that we can decompress a legacy .lzma file
#[test]
fn test_legacy_format() {
	let decompressed = lzma::decompress(TEST_LEGACY_DATA).unwrap();
	assert_eq!(decompressed, TEST_STRING.as_bytes());
}