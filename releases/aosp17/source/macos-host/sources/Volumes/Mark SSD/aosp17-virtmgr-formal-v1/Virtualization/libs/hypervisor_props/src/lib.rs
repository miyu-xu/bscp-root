// Copyright 2026 The Android Open Source Project
// SPDX-License-Identifier: Apache-2.0
//! Capabilities of the desktop Host backend, not Android boot properties.
//! This provider implements macOS HVF; unported targets fail explicitly.
use anyhow::{bail, Result};
#[cfg(target_os = "macos")]
fn query_bool(name: &std::ffi::CStr) -> Result<bool> {
    let mut value: libc::c_int = 0;
    let mut length = std::mem::size_of_val(&value);
    // SAFETY: writable int and size are valid for the duration of this query.
    let rc = unsafe { libc::sysctlbyname(name.as_ptr(), (&mut value as *mut libc::c_int).cast(), &mut length, std::ptr::null_mut(), 0) };
    if rc != 0 { return Err(std::io::Error::last_os_error().into()); }
    if length != std::mem::size_of_val(&value) { bail!("unexpected hypervisor capability length {length}"); }
    match value { 0 => Ok(false), 1 => Ok(true), _ => bail!("invalid hypervisor capability {value}") }
}
fn require_hvf_provider() -> Result<()> {
    if !cfg!(target_os = "macos") { bail!("desktop hypervisor capability provider is not ported on this target"); }
    Ok(())
}
pub fn is_vm_supported() -> Result<bool> {
    #[cfg(target_os = "macos")]
    { query_bool(std::ffi::CStr::from_bytes_with_nul(b"kern.hv_support\0").unwrap()) }
    #[cfg(not(target_os = "macos"))]
    { bail!("desktop hypervisor capability provider is not ported on this target") }
}
// The current HVF Host backend implements neither protected VMs nor Gunyah
// nor Android dynamic zero-copy memshare. These are backend feature declarations,
// not claims that those operations succeeded.
pub fn is_protected_vm_supported() -> Result<bool> { require_hvf_provider()?; Ok(false) }
pub fn is_gunyah() -> Result<bool> { require_hvf_provider()?; Ok(false) }
pub fn is_dynamic_zero_copy_memshare_supported() -> Result<bool> { require_hvf_provider()?; Ok(false) }
pub fn is_any_vm_supported() -> Result<bool> { is_vm_supported() }
pub fn version() -> Result<Option<String>> { require_hvf_provider()?; Ok(None) }
#[cfg(all(test,target_os = "macos"))]
mod tests {
    use super::*;
    #[test] fn actual_os_query_and_backend_features() {
        assert_eq!(is_any_vm_supported().unwrap(),is_vm_supported().unwrap());
        assert!(!is_protected_vm_supported().unwrap());assert!(!is_gunyah().unwrap());
        assert!(!is_dynamic_zero_copy_memshare_supported().unwrap());
    }
    #[test] fn absent_os_property_is_error() {
        assert!(query_bool(std::ffi::CStr::from_bytes_with_nul(b"kern.bscp_nonexistent_hvf_probe\0").unwrap()).is_err());
    }
}
