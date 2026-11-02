//! A dedicated inherited anonymous-pipe writer carries status, never credentials.
use std::{fs::File,io,os::windows::io::FromRawHandle};
use base::sys::windows::inherited_handles::native_snapshot;
use windows_sys::Win32::Foundation::{SetHandleInformation,HANDLE_FLAG_INHERIT};
/// Adopt the single dedicated startup slot from the launcher's exact handle list.
/// Validation never scans or closes other handles. This proves a pipe capability,
/// not the caller's identity or the authenticity of a later RPC connection.
///
/// # Safety
/// `raw` must be uniquely owned by this startup role: no other Rust owner or
/// concurrent close/reuse may exist. The launcher passes a dedicated duplicate.
pub unsafe fn take_inherited(raw:usize)->io::Result<File>{
 if raw==0||raw==usize::MAX{return Err(io::ErrorKind::InvalidInput.into());}
 let snapshot=native_snapshot::snapshot(raw as *mut std::ffi::c_void,true)?;
 // WRITE_DATA without READ_DATA; byte pipe; synchronous handle required by File.
 if snapshot.kind!=3||snapshot.access&2==0||snapshot.access&1!=0||snapshot.pipe_flags&4!=0||snapshot.pipe_state&3!=0||snapshot.mode&0x30==0 {
  return Err(io::Error::new(io::ErrorKind::PermissionDenied,"ready role requires inherited synchronous byte-pipe writer"));
 }
 let owned=File::from_raw_handle(raw as *mut std::ffi::c_void);
 if SetHandleInformation(raw as _,HANDLE_FLAG_INHERIT,0)==0{return Err(io::Error::last_os_error());}
 Ok(owned)
}
