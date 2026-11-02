
use std::{env,path::PathBuf};
fn main() {
 println!("cargo:rerun-if-env-changed=BINDER_NATIVE_SOURCE");
 println!("cargo:rerun-if-env-changed=BINDER_NATIVE_LIBS");

    let source=PathBuf::from(env::var_os("BINDER_NATIVE_SOURCE").expect("BINDER_NATIVE_SOURCE is required"));
    let libs=PathBuf::from(env::var_os("BINDER_NATIVE_LIBS").expect("BINDER_NATIVE_LIBS is required"));
    assert!(source.join("ndk/include_ndk/android/binder_ibinder.h").is_file());
    let os=env::var("CARGO_CFG_TARGET_OS").unwrap();
    let filename=match os.as_str(){"macos"=>"libbinder_ndk_sdk.dylib","linux"=>"libbinder_ndk_sdk.so","windows"=>"libbinder_ndk_sdk.dll",_=>panic!("unsupported desktop RPC target")};
    assert!(libs.join(filename).is_file());

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
    if os == "windows" {
        println!("cargo:rerun-if-env-changed=BINDER_WINDOWS_PLATFORM");
        let platform=PathBuf::from(env::var_os("BINDER_WINDOWS_PLATFORM").expect("BINDER_WINDOWS_PLATFORM"));
        assert!(platform.join("WindowsHeaders.h").is_file());
        for inc in [&platform, &platform.join("win32")] { b=b.clang_arg(format!("-I{}",inc.display())); }
        b=b.clang_arg("-include").clang_arg(platform.join("WindowsHeaders.h").to_string_lossy())
            .clang_arg("-include").clang_arg("binder/Common.h");
        b=b.header(source.join("vm_sockets.h").to_string_lossy())
            .allowlist_type("sockaddr_un").allowlist_type("sockaddr_vm")
            .derive_partialeq(true).derive_eq(true);
    }
    b.generate().expect("fixed NDK header binding generation failed")
        .write_to_file(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.rs")).unwrap();
    println!("cargo:rerun-if-env-changed=BINDER_NATIVE_SOURCE");
    println!("cargo:rerun-if-env-changed=BINDER_NATIVE_LIBS");
    println!("cargo:rustc-link-search=native={}",libs.display());
    println!("cargo:rustc-link-lib=dylib=binder_ndk_sdk");
}
