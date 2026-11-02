
use std::{env,path::PathBuf};
fn main() {
 println!("cargo:rerun-if-env-changed=BINDER_NATIVE_SOURCE");
 println!("cargo:rerun-if-env-changed=BINDER_NATIVE_LIBS");

    let source=PathBuf::from(env::var_os("BINDER_NATIVE_SOURCE").expect("BINDER_NATIVE_SOURCE is required"));
    let libs=PathBuf::from(env::var_os("BINDER_NATIVE_LIBS").expect("BINDER_NATIVE_LIBS is required"));
    assert!(source.join("ndk/include_ndk/android/binder_ibinder.h").is_file());
    assert!(libs.join("libbinder_ndk_sdk.dylib").is_file());
    assert_eq!(env::var("CARGO_CFG_TARGET_OS").unwrap(), "macos", "this host build has only been configured for Mac");
    let mut b=bindgen::Builder::default().header("BinderBindings.hpp")
        .clang_arg("-x").clang_arg("c++").clang_arg("-std=c++17").clang_arg("-DANDROID_PLATFORM")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .default_enum_style(bindgen::EnumVariation::Rust{non_exhaustive:true})
        .constified_enum("android::c_interface::consts::.*")
        .allowlist_type("android::c_interface::.*")
        .allowlist_type("A.*").allowlist_type("binder_.*")
        .blocklist_type("sockaddr").raw_line("use libc::sockaddr;")
        .allowlist_function("A.*");
    for inc in ["include", "ndk/include_ndk", "ndk/include_cpp", "ndk/include_platform"] {
        b=b.clang_arg(format!("-I{}",source.join(inc).display()));
    }
    b.generate().expect("fixed NDK header binding generation failed")
        .write_to_file(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.rs")).unwrap();
    println!("cargo:rerun-if-env-changed=BINDER_NATIVE_SOURCE");
    println!("cargo:rerun-if-env-changed=BINDER_NATIVE_LIBS");
    println!("cargo:rustc-link-search=native={}",libs.display());
    println!("cargo:rustc-link-lib=dylib=binder_ndk_sdk");
}
