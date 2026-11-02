fn main() {
    assert!(matches!(std::env::var("CARGO_CFG_TARGET_OS").unwrap().as_str(), "macos" | "linux"), "unsupported desktop RPC target");
    assert!(std::env::var_os("CARGO_FEATURE_KERNEL_IPC").is_none(), "kernel Binder IPC is unsupported in the desktop RPC-only build");
    println!("cargo::rustc-check-cfg=cfg(single_threaded)");
}
