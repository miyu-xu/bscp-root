//! Windows OS CSPRNG. No environment seed or fallback implementation.
#![cfg(windows)]
use std::io;
use windows_sys::Win32::Security::Cryptography::{BCryptGenRandom,BCRYPT_USE_SYSTEM_PREFERRED_RNG};
fn finish(buffer: &mut [u8], status: i32) -> io::Result<()> {
    if status == 0 { Ok(()) } else {
        buffer.fill(0);
        Err(io::Error::other(format!("BCryptGenRandom failed NTSTATUS=0x{:08x}", status as u32)))
    }
}
pub fn fill(buffer: &mut [u8]) -> io::Result<()> {
    let length = u32::try_from(buffer.len()).map_err(|_| io::ErrorKind::InvalidInput)?;
    // No provider handle is used with the system-preferred flag.
    let status = unsafe { BCryptGenRandom(0, buffer.as_mut_ptr(), length, BCRYPT_USE_SYSTEM_PREFERRED_RNG) };
    finish(buffer, status)
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn real_os_fills_instance_id_buffer() { let mut id=[0u8;64]; fill(&mut id).unwrap(); }
    #[test] fn injected_failure_clears_partial_bytes_and_preserves_status() {
        let mut id=[0xa5u8;64]; let error=finish(&mut id,0xc000000du32 as i32).unwrap_err();
        assert_eq!(id,[0;64]);assert!(error.to_string().contains("0xc000000d"));
    }
}
