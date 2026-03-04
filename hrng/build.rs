use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=csrc/camera.swift");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let swift_obj = out_dir.join("camera_swift.o");

    let status = Command::new("xcrun")
        .arg("swiftc")
        .arg("-parse-as-library")
        .arg("-emit-object")
        .arg("csrc/camera.swift")
        .arg("-o")
        .arg(&swift_obj)
        .status()
        .expect("failed to run xcrun swiftc");

    if !status.success() {
        panic!("swiftc failed with status: {status}");
    }

    println!("cargo:rustc-link-arg={}", swift_obj.display());
    println!("cargo:rustc-link-lib=framework=AVFoundation");
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=CoreVideo");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=objc");
}
