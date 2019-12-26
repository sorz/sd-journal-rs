use bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=systemd");
    let bindings = bindgen::Builder::default()
        .whitelist_function("sd_journal_.*")
        .whitelist_type("SD_JOURNAL_.*")
        .whitelist_var("SD_JOURNAL_.*")
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
