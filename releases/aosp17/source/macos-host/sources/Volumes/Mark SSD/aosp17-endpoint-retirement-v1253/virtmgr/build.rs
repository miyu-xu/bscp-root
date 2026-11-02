// Copyright 2026 The Android Open Source Project
// SPDX-License-Identifier: Apache-2.0
// Mirror the pinned upstream security_vm_instance_id Soong genrule.
use std::{env, fs, path::PathBuf, process::Command};
fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir")).join("..");
    let generator = root.join("guest/trusty/tools/gen_instance_id_for_vm_with_trusted_hal/gen_instance_id_for_vm_with_trusted_hal.py");
    let config = root.join("guest/trusty/security_vm/launcher/security_vm_instance_id_config.json");
    println!("cargo:rerun-if-changed={}", generator.display());
    println!("cargo:rerun-if-changed={}", config.display());
    println!("cargo:rerun-if-env-changed=PYTHON");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("output dir")).join("security_vm_instance_id");
    let status = Command::new(env::var_os("PYTHON").unwrap_or_else(|| "python3".into()))
        .arg("-B").arg(&generator).arg("-i").arg(&config).arg("-o").arg(&output)
        .status().expect("start the official instance-ID generator");
    assert!(status.success(), "official instance-ID generation failed: {status}");
    assert_eq!(fs::metadata(&output).expect("generated instance ID").len(), 64);
}
