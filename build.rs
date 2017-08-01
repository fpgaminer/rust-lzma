extern crate pkg_config;

use pkg_config::find_library;


fn main() {
	match find_library("liblzma") {
		Ok(_) => {},
        Err(err) => {
    		panic!("Could not find liblzma using pkg-config: {}", err);
    	}
    }
}
