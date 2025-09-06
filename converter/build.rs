use std::{env::var, process::Command};

fn main() {
    let out_dir = var("OUT_DIR").unwrap();
    println!("cargo::rerun-if-changed=go/main.go");
    let mut gobuild = Command::new("go");
    gobuild
        .args(&["build", "-buildmode=c-archive", "-o"])
        .arg(format!("{}/libconverter.a", out_dir))
        .arg("main.go")
        .current_dir("go")
        .status()
        .unwrap();
    println!("cargo::rustc-link-search=native={}", out_dir);
    println!("cargo::rustc-link-lib=converter");
}
