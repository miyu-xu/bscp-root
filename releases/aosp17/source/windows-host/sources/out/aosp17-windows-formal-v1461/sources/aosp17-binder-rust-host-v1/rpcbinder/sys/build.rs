use std::{env,path::PathBuf};
fn main(){
 println!("cargo:rerun-if-env-changed=BINDER_NATIVE_SOURCE");
 println!("cargo:rerun-if-env-changed=BINDER_NATIVE_LIBS");

 let b=PathBuf::from(env::var_os("BINDER_NATIVE_SOURCE").expect("BINDER_NATIVE_SOURCE"));
 let l=PathBuf::from(env::var_os("BINDER_NATIVE_LIBS").expect("BINDER_NATIVE_LIBS"));
 assert!(matches!(env::var("CARGO_CFG_TARGET_OS").unwrap().as_str(), "macos" | "linux" | "windows"), "unsupported desktop RPC target");
 let mut builder=bindgen::Builder::default().parse_callbacks(Box::new(bindgen::CargoCallbacks::new())).header("../BinderBindings.hpp").clang_arg("-x").clang_arg("c++").clang_arg("-std=c++17")
 .clang_arg(format!("-I{}",b.join("include_rpc_unstable").display()))
 .clang_arg(format!("-I{}",b.join("ndk/include_ndk").display()))
 .allowlist_type("ARpc.*").allowlist_function("ARpc.*")
 .blocklist_type("AIBinder").raw_line("use binder_ndk_sys::AIBinder;")
 .rustified_enum("ARpcSession_FileDescriptorTransportMode")
;
 if env::var("CARGO_CFG_TARGET_OS").unwrap()=="windows" {
 println!("cargo:rerun-if-env-changed=BINDER_WINDOWS_PLATFORM");
 let platform=PathBuf::from(env::var_os("BINDER_WINDOWS_PLATFORM").expect("BINDER_WINDOWS_PLATFORM"));
 builder=builder.clang_arg(format!("-I{}",platform.display())).clang_arg(format!("-I{}",platform.join("win32").display())).clang_arg(format!("-I{}",b.join("include").display())).clang_arg("-include").clang_arg(platform.join("WindowsHeaders.h").to_string_lossy()).clang_arg("-include").clang_arg("binder/Common.h");
 }
 builder.generate().expect("fixed RPC header bindings").write_to_file(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.rs")).unwrap();
 println!("cargo:rustc-link-search=native={}",l.display());
 println!("cargo:rustc-link-lib=dylib=binder_rpc_unstable_host");
}
