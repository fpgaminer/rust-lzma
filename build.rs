use std::env;

fn main() {
    let statik = env::var("CARGO_FEATURE_STATIC").is_ok();

    pkg_config::Config::new()
        .statik(statik)
        .probe("liblzma")
        .expect("Could not find liblzma using pkg-config");
    if statik {
        println!("cargo:rustc-link-lib=static=lzma");
    }
}
