// Copyright 2026 The Android Open Source Project
// SPDX-License-Identifier: Apache-2.0
//! Process-local resource limits. Errors are never converted to success.
use std::io;
fn set_memlock(limit: libc::rlimit) -> io::Result<()> {
    // SAFETY: a valid borrowed structure, affecting only this process.
    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &limit) } == -1 {
        Err(io::Error::last_os_error())
    } else { Ok(()) }
}
pub fn remove_memlock_limit() -> io::Result<()> {
    set_memlock(libc::rlimit { rlim_cur: libc::RLIM_INFINITY, rlim_max: libc::RLIM_INFINITY })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn current() -> libc::rlimit {
        let mut v = std::mem::MaybeUninit::<libc::rlimit>::uninit();
        assert_eq!(unsafe { libc::getrlimit(libc::RLIMIT_MEMLOCK, v.as_mut_ptr()) }, 0);
        unsafe { v.assume_init() }
    }
    #[test]
    fn same_limit_preserves_state() {
        let before = current(); set_memlock(before).unwrap(); let after = current();
        assert_eq!(before.rlim_cur, after.rlim_cur); assert_eq!(before.rlim_max, after.rlim_max);
    }
    #[test]
    fn invalid_limit_reports_error_without_change() {
        let before = current();
        let err = set_memlock(libc::rlimit { rlim_cur: 1, rlim_max: 0 }).unwrap_err();
        assert_eq!(err.raw_os_error(), Some(libc::EINVAL));
        let after = current(); assert_eq!(before.rlim_cur, after.rlim_cur); assert_eq!(before.rlim_max, after.rlim_max);
    }
}
