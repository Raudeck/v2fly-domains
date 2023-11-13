use std::process::Command;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("cargo:rerun-if-changed=libs/converter.go");
    let mut go_build = Command::new("go");
    go_build
        .args(&["build", "-buildmode=c-archive", "-o"])
        .arg(format!("{}/libconverter.a", out_dir))
        .arg("converter.go")
        .current_dir("libs").status().unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=converter");
}