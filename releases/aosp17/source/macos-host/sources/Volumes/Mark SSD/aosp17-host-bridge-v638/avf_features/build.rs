// Copyright 2024, The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Matches Soong `rustflags` `--cfg foo` for AVF feature gates (see `lib.rs`).
    println!("cargo:rustc-check-cfg=cfg(dice_changes)");
    println!("cargo:rustc-check-cfg=cfg(llpvm_changes)");
    println!("cargo:rustc-check-cfg=cfg(multi_tenant)");
    println!("cargo:rustc-check-cfg=cfg(network)");
    println!("cargo:rustc-check-cfg=cfg(remote_attestation)");
    println!("cargo:rustc-check-cfg=cfg(vendor_modules)");
}
