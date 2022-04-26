use std::process::Command;
use std::path::Path;

use autotools;

fn main() {

    if !Path::new("mypkg/configure").exists() {
        Command::new("autoreconf").args(&["-i", "mypkg"]).status().unwrap();
    }

    let dst = autotools::build("mypkg");

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=mypkg");
}
