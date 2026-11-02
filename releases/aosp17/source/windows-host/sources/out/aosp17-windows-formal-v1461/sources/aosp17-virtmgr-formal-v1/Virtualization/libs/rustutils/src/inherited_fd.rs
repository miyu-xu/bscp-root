// Copyright 2026, The Android Open Source Project
// Licensed under the Apache License, Version 2.0.
//! Host implementation of A17's one-shot inherited OwnedFd registry.
//! Call init_once at process entry, before any non-stdio FD ownership or threads.
use std::collections::{HashMap,HashSet};
use std::io;
use std::os::fd::{FromRawFd,OwnedFd,RawFd};
use std::sync::{Mutex,OnceLock};

static FDS: OnceLock<Mutex<HashMap<RawFd,Option<OwnedFd>>>>=OnceLock::new();
static INIT: Mutex<()>=Mutex::new(());

#[cfg(target_os="macos")]
fn enumerate() -> io::Result<Vec<RawFd>> {
    #[repr(C)]
    #[derive(Clone,Copy,Default)]
    struct ProcFdInfo { fd:i32, kind:u32 }
    extern "C" { fn proc_pidinfo(pid:i32,flavor:i32,arg:u64,buffer:*mut libc::c_void,size:i32)->i32; }
    const PROC_PIDLISTFDS:i32=1; // Apple SDK sys/proc_info.h
    let pid=unsafe{libc::getpid()};
    let size=unsafe{proc_pidinfo(pid,PROC_PIDLISTFDS,0,std::ptr::null_mut(),0)};
    if size<=0 { return Err(io::Error::last_os_error()); }
    let width=std::mem::size_of::<ProcFdInfo>();
    if size as usize % width != 0 {return Err(io::Error::other("Invalid proc_pidinfo size"));}
    let capacity=(size as usize/width).checked_add(32).ok_or_else(||io::Error::other("FD count overflow"))?;
    let bytes=capacity.checked_mul(width).and_then(|v|i32::try_from(v).ok()).ok_or_else(||io::Error::other("FD buffer overflow"))?;
    let mut entries=vec![ProcFdInfo::default();capacity];
    let read=unsafe{proc_pidinfo(pid,PROC_PIDLISTFDS,0,entries.as_mut_ptr().cast(),bytes)};
    if read<=0 {return Err(io::Error::last_os_error());}
    if read>=bytes || read as usize % width != 0 {return Err(io::Error::other("FD inventory truncated or invalid"));}
    Ok(entries[..read as usize/width].iter().map(|e|e.fd).filter(|fd|*fd>2).collect())
}

#[cfg(any(target_os="linux",target_os="android"))]
fn enumerate() -> io::Result<Vec<RawFd>> {
    struct Directory(*mut libc::DIR);
    impl Drop for Directory { fn drop(&mut self) { unsafe {libc::closedir(self.0); } } }
    #[cfg(target_os="linux")]
    unsafe fn errno_ptr()->*mut libc::c_int {unsafe{libc::__errno_location()}}
    #[cfg(target_os="android")]
    unsafe fn errno_ptr()->*mut libc::c_int {unsafe{libc::__errno()}}
    let raw=unsafe{libc::opendir(b"/proc/self/fd\0".as_ptr().cast())};
    if raw.is_null() {return Err(io::Error::last_os_error());}
    let dir=Directory(raw);
    let own_fd=unsafe{libc::dirfd(dir.0)};
    if own_fd<0 {return Err(io::Error::last_os_error());}
    let mut out=Vec::new();
    loop {
        unsafe{*errno_ptr()=0;}
        let entry=unsafe{libc::readdir(dir.0)};
        if entry.is_null() {
            let errno=unsafe{*errno_ptr()};
            if errno!=0 {return Err(io::Error::from_raw_os_error(errno));}
            break;
        }
        let name=unsafe{std::ffi::CStr::from_ptr((*entry).d_name.as_ptr())};
        if name.to_bytes()==b"." || name.to_bytes()==b".." {continue;}
        let fd=name.to_str().ok().and_then(|v|v.parse::<RawFd>().ok()).ok_or_else(||io::Error::other("Invalid /proc fd entry"))?;
        // Exclude only this enumeration's exact owned directory descriptor.
        // An inherited descriptor pointing to the same directory is still inherited.
        if fd>2 && fd!=own_fd {out.push(fd);}
    }
    Ok(out)
}

/// # Safety
/// Must run before any non-stdio FD has a Rust owner and before other threads
/// can open/close FDs. This is the same startup precondition as A17 upstream.
pub unsafe fn init_once() -> io::Result<()> {
    let _guard=INIT.lock().map_err(|_|io::Error::other("FD initialization poisoned"))?;
    if FDS.get().is_some() {return Err(io::Error::other("Inherited FDs already initialized"));}
    let raw=enumerate()?;
    let mut unique=HashSet::new();
    for &fd in &raw {
        if fd<=2 || !unique.insert(fd) {return Err(io::Error::other("Invalid or duplicate FD inventory"));}
        let flags=unsafe{libc::fcntl(fd,libc::F_GETFD)};
        if flags<0 {return Err(io::Error::last_os_error());}
        if unsafe{libc::fcntl(fd,libc::F_SETFD,flags|libc::FD_CLOEXEC)}<0 {return Err(io::Error::last_os_error());}
    }
    let owned=raw.into_iter().map(|fd|(fd,Some(unsafe{OwnedFd::from_raw_fd(fd)}))).collect();
    FDS.set(Mutex::new(owned)).map_err(|_|io::Error::other("Inherited FDs already initialized"))
}

pub fn take_fd_ownership(fd:RawFd)->io::Result<OwnedFd> {
    let map=FDS.get().ok_or_else(||io::Error::other("Inherited FDs not initialized"))?;
    let mut map=map.lock().map_err(|_|io::Error::other("Inherited FD registry poisoned"))?;
    map.get_mut(&fd).ok_or_else(||io::Error::new(io::ErrorKind::InvalidInput,"FD was not inherited"))?
        .take().ok_or_else(||io::Error::new(io::ErrorKind::InvalidInput,"FD ownership already taken"))
}
