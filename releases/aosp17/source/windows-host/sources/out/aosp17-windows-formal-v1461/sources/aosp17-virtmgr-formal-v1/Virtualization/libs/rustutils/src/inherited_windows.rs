//! Explicit Windows startup HANDLE ownership, never a process-wide handle scan.
//! The launcher must retain and verify its exact inherited HANDLE allowlist.
//! Numeric labels are protocol labels only, never HANDLE or foreign CRT values.
use binder::platform_fd::{OwnedFd,RawFd};
use std::{collections::HashMap,io,os::windows::io::OwnedHandle,sync::{Mutex,OnceLock}};
static FDS:OnceLock<Mutex<HashMap<RawFd,Option<OwnedFd>>>>=OnceLock::new();
static INIT:Mutex<()>=Mutex::new(());
/// The Unix no-argument inventory contract cannot authenticate Windows handles.
/// Call initialize_handles with the verified launcher allowlist instead.
pub unsafe fn init_once()->io::Result<()> {Err(io::Error::new(io::ErrorKind::Unsupported,"Windows requires an explicit verified inherited HANDLE allowlist"))}
fn import(handles:Vec<(RawFd,OwnedHandle)>)->io::Result<HashMap<RawFd,Option<OwnedFd>>> {
 let mut out=HashMap::new();
 for (label,handle) in handles {
  if label<=2 || out.contains_key(&label){return Err(io::Error::new(io::ErrorKind::InvalidInput,"invalid or duplicate inherited label"));}
  // Native duplication is non-inheritable; consumed original closes here.
  out.insert(label,Some(OwnedFd::try_from_handle(&handle)?));
 }
 Ok(out)
}
/// Consume only handles proven to be inherited by the startup launcher contract.
/// Caller must call before worker threads; this API does not authenticate their
/// source and must never be populated by scanning or environment handle values.
pub fn initialize_handles(handles:Vec<(RawFd,OwnedHandle)>)->io::Result<()> {
 let _guard=INIT.lock().map_err(|_|io::Error::other("inherited initialization poisoned"))?;
 if FDS.get().is_some(){return Err(io::Error::other("inherited handles already initialized"));}
 let map=import(handles)?;
 FDS.set(Mutex::new(map)).map_err(|_|io::Error::other("inherited handles already initialized"))
}
pub fn take_fd_ownership(label:RawFd)->io::Result<OwnedFd> {
 let map=FDS.get().ok_or_else(||io::Error::other("inherited handles not initialized"))?;
 map.lock().map_err(|_|io::Error::other("inherited handles poisoned"))?.get_mut(&label)
  .ok_or_else(||io::Error::new(io::ErrorKind::InvalidInput,"label not inherited"))?
  .take().ok_or_else(||io::Error::new(io::ErrorKind::InvalidInput,"ownership already taken"))
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn implicit_inventory_refused(){assert_eq!(unsafe{init_once()}.unwrap_err().kind(),io::ErrorKind::Unsupported);}
 #[test] fn unknown_registry_refused(){assert!(take_fd_ownership(123).is_err());}
}
