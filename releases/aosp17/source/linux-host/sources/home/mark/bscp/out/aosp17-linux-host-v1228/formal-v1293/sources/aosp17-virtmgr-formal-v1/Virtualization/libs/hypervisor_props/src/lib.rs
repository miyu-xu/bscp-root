// Copyright 2026 The Android Open Source Project
// SPDX-License-Identifier: Apache-2.0
//! Capabilities of the desktop Host backend, not Android boot properties.
//! This provider implements macOS HVF and Linux normal-VM KVM; other targets fail explicitly.
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
fn require_desktop_provider() -> Result<()> {
    if !cfg!(any(target_os = "macos", target_os = "linux")) { bail!("desktop hypervisor capability provider is not ported on this target"); }
    Ok(())
}
pub fn is_vm_supported() -> Result<bool> {
    #[cfg(target_os = "macos")]
    { query_bool(std::ffi::CStr::from_bytes_with_nul(b"kern.hv_support\0").unwrap()) }
    #[cfg(target_os = "linux")]
    { query_kvm_api()?; Ok(true) }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    { bail!("desktop hypervisor capability provider is not ported on this target") }
}
// The selected desktop normal-VM backends implement neither protected VMs nor Gunyah
// nor Android dynamic zero-copy memshare. These are backend feature declarations,
// not claims that those operations succeeded.
pub fn is_protected_vm_supported() -> Result<bool> { require_desktop_provider()?; Ok(false) }
pub fn is_gunyah() -> Result<bool> { require_desktop_provider()?; Ok(false) }
pub fn is_dynamic_zero_copy_memshare_supported() -> Result<bool> { require_desktop_provider()?; Ok(false) }
pub fn is_any_vm_supported() -> Result<bool> { is_vm_supported() }
pub fn version() -> Result<Option<String>> {
 require_desktop_provider()?;
 #[cfg(target_os="linux")] { return Ok(Some(format!("KVM API {}",query_kvm_api()?))); }
 #[cfg(not(target_os="linux"))] { Ok(None) }
}
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

#[cfg(target_os="linux")]
fn kvm_api_from_fd(fd:std::os::fd::BorrowedFd<'_>)->Result<i32>{
 use std::os::fd::AsRawFd;
 // Linux UAPI: KVM_GET_API_VERSION = _IO(KVMIO=0xAE, 0x00).
 // This query creates no VM and has no pointer argument.
 let version=unsafe{libc::ioctl(fd.as_raw_fd(),0xAE00 as libc::c_ulong)};
 if version<0{return Err(std::io::Error::last_os_error().into());}
 if version!=12 {bail!("unsupported KVM API version {version}");}
 Ok(version)
}
#[cfg(target_os="linux")]
fn query_kvm_api()->Result<i32>{
 use std::os::{fd::AsFd,unix::fs::OpenOptionsExt};
 let file=std::fs::OpenOptions::new().read(true).write(true).custom_flags(libc::O_CLOEXEC).open("/dev/kvm")?;
 kvm_api_from_fd(file.as_fd())
}
#[cfg(all(test,target_os="linux"))]
mod linux_tests{
 use super::*;use std::os::fd::AsFd;
 #[test]fn real_kvm_api_without_creating_vm(){assert_eq!(query_kvm_api().unwrap(),12);assert!(is_vm_supported().unwrap());}
 #[test]fn wrong_device_is_error(){let f=std::fs::File::open("/dev/null").unwrap();assert!(kvm_api_from_fd(f.as_fd()).is_err());}
 #[test]fn explicit_normal_backend_scope(){assert!(!is_protected_vm_supported().unwrap());assert!(!is_gunyah().unwrap());assert!(!is_dynamic_zero_copy_memshare_supported().unwrap());}
}
