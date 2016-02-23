extern crate lzma;

use lzma::error::LzmaError;
use std::io::{Read, Cursor, Write};


const TEST_STRING: &'static str = include_str!("test_file.txt");


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
