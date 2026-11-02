fn main() {
    assert_eq!(std::env::var("CARGO_CFG_TARGET_OS").unwrap(), "macos");
    assert!(std::env::var_os("CARGO_FEATURE_KERNEL_IPC").is_none(), "kernel Binder IPC is unsupported in the Mac RPC-only build");
    println!("cargo::rustc-check-cfg=cfg(single_threaded)");
}
