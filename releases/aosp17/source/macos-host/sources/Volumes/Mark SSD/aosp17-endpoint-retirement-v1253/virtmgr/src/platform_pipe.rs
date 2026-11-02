// Copyright 2026 The Android Open Source Project
// SPDX-License-Identifier: Apache-2.0
//! Darwin lacks pipe2. Owned descriptors close on all errors; CLOEXEC setup is
//! not atomic against concurrent fork and does not claim otherwise.
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
fn create(mut configure: impl FnMut(&OwnedFd) -> io::Result<()>) -> io::Result<(OwnedFd, OwnedFd)> {
    let mut fds=[-1;2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } == -1 { return Err(io::Error::last_os_error()); }
    // SAFETY: ownership of the two newly created descriptors is transferred once.
    let r=unsafe { OwnedFd::from_raw_fd(fds[0]) }; let w=unsafe { OwnedFd::from_raw_fd(fds[1]) };
    configure(&r)?; configure(&w)?; Ok((r,w))
}
pub fn pipe_cloexec() -> io::Result<(OwnedFd,OwnedFd)> {
    create(|f| {
        let n=unsafe { libc::fcntl(f.as_raw_fd(),libc::F_GETFD) };
        if n == -1 { return Err(io::Error::last_os_error()); }
        if unsafe { libc::fcntl(f.as_raw_fd(),libc::F_SETFD,n|libc::FD_CLOEXEC) } == -1 { return Err(io::Error::last_os_error()); }
        Ok(())
    })
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn real_pipe_flags_and_data() {
        let(r,w)=pipe_cloexec().unwrap();
        for f in [&r,&w] { let flags=unsafe { libc::fcntl(f.as_raw_fd(),libc::F_GETFD) }; assert!(flags >= 0); assert_ne!(flags & libc::FD_CLOEXEC,0); }
        assert_eq!(unsafe { libc::write(w.as_raw_fd(),b"x".as_ptr().cast(),1) },1);
        let mut b=[0u8;1]; assert_eq!(unsafe { libc::read(r.as_raw_fd(),b.as_mut_ptr().cast(),1) },1);assert_eq!(b,[b'x']);
    }
    #[test] fn configure_error_closes_owned_descriptors() {
        let mut seen=Vec::new(); let e=create(|f| {seen.push(f.as_raw_fd());if seen.len()==2 {Err(io::Error::from_raw_os_error(libc::EIO))}else{Ok(())}}).unwrap_err();
        assert_eq!(e.raw_os_error(),Some(libc::EIO)); assert_eq!(seen.len(),2);
        for fd in seen { assert_eq!(unsafe {libc::fcntl(fd,libc::F_GETFD)},-1);assert_eq!(io::Error::last_os_error().raw_os_error(),Some(libc::EBADF)); }
    }
}
