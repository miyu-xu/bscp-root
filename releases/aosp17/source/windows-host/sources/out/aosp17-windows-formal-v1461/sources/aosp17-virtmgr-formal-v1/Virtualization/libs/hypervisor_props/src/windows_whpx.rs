// Copyright 2026 The Android Open Source Project
// SPDX-License-Identifier: Apache-2.0
//! Read-only WHPX availability. Never creates a partition or enables a feature.
use anyhow::{bail, Context, Result};
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::Hypervisor::{WHV_CAPABILITY, WHvCapabilityCodeHypervisorPresent};
use windows_sys::Win32::System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32};

type GetCapability = unsafe extern "system" fn(i32, *mut core::ffi::c_void, u32, *mut u32) -> i32;
struct Library(HMODULE);
impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: one successfully loaded module reference owned by this guard.
        unsafe { FreeLibrary(self.0); }
    }
}

fn interpret(hr: i32, written: u32, present: i32) -> Result<bool> {
    if hr < 0 { bail!("WHvGetCapability failed: HRESULT 0x{:08x}", hr as u32); }
    if written != core::mem::size_of::<i32>() as u32 {
        bail!("WHvGetCapability returned unexpected HypervisorPresent size {written}");
    }
    // BOOL follows the Win32 convention: zero is false, nonzero is true.
    Ok(present != 0)
}

pub(super) fn query() -> Result<bool> {
    let name: Vec<u16> = "WinHvPlatform.dll\0".encode_utf16().collect();
    // SAFETY: terminated fixed system DLL name. System32-only lookup prevents
    // the application/current directory from supplying this capability provider.
    let module = unsafe { LoadLibraryExW(name.as_ptr(), 0, LOAD_LIBRARY_SEARCH_SYSTEM32) };
    if module == 0 { return Err(std::io::Error::last_os_error()).context("load system WHPX provider"); }
    let library = Library(module);
    // SAFETY: static terminated export name and live module reference.
    let symbol = unsafe { GetProcAddress(library.0, b"WHvGetCapability\0".as_ptr()) };
    let Some(symbol) = symbol else {
        return Err(std::io::Error::last_os_error()).context("resolve WHvGetCapability");
    };
    // SAFETY: official Windows SDK signature for this exact export. The library
    // guard stays alive through the call; no pointer escapes this function.
    let function: GetCapability = unsafe { core::mem::transmute(symbol) };
    let mut capability: WHV_CAPABILITY = unsafe { core::mem::zeroed() };
    let mut written = 0;
    let hr = unsafe { function(WHvCapabilityCodeHypervisorPresent,
        (&mut capability as *mut WHV_CAPABILITY).cast(),
        core::mem::size_of::<WHV_CAPABILITY>() as u32, &mut written) };
    // Only the requested union member is read; errors and short replies cannot
    // report availability. The zero initialization also avoids uninitialized data.
    interpret(hr, written, unsafe { capability.HypervisorPresent })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn failed_and_malformed_results_are_errors() {
        assert!(interpret(0x80004005_u32 as i32, 4, 1).is_err());
        assert!(interpret(0, 0, 1).is_err());
        assert!(interpret(0, 8, 1).is_err());
    }
    #[test] fn absent_is_false_and_win32_bool_is_respected() {
        assert!(!interpret(0, 4, 0).unwrap());
        assert!(interpret(0, 4, 1).unwrap());
        assert!(interpret(0, 4, -1).unwrap());
    }
    #[test] fn real_read_only_query_and_backend_scope() {
        let result = query();
        println!("actual WHPX HypervisorPresent={result:?}");
        assert!(result.is_ok(), "actual provider query failed: {result:?}");
        assert_eq!(result.unwrap(), crate::is_vm_supported().unwrap());
        assert!(!crate::is_protected_vm_supported().unwrap());
        assert!(!crate::is_gunyah().unwrap());
        assert!(!crate::is_dynamic_zero_copy_memshare_supported().unwrap());
    }
}
