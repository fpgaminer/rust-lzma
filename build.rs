extern crate pkg_config;

use pkg_config::find_library;


fn main() {
	if find_library("liblzma").is_ok() {
		return
	} else {
		panic!("Could not find liblzma using pkg-config")
	}
}
