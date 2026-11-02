//! Platform descriptor ownership for Binder's native integer ABI.
//! Windows descriptors are owned by binder_sdk's CRT/typed registry, never raw HANDLE integers.
#[cfg(unix)]
pub use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
#[cfg(windows)]
pub use windows::*;
#[cfg(windows)]
mod windows {
    use std::{ffi::c_void,io,marker::PhantomData,mem::ManuallyDrop,num::NonZeroU32};
    use std::os::windows::io::{AsRawHandle,FromRawHandle,OwnedHandle};
    pub type RawFd=i32;
    #[link(name="binder_sdk")]
    extern "C" {
        fn bscp_fd_close(fd:i32)->i32;
        fn bscp_fd_dup(fd:i32,out:*mut i32)->i32;
        fn bscp_fd_from_handle(handle:*mut c_void,out:*mut i32)->i32;
        fn bscp_fd_to_handle(fd:i32,out:*mut *mut c_void)->i32;
    }
    fn checked(status:i32)->io::Result<()> {
        if status==0 {Ok(())} else {Err(io::Error::other(format!("native Binder descriptor error {status}")))}
    }
    pub trait AsRawFd {fn as_raw_fd(&self)->RawFd;}
    pub trait IntoRawFd {fn into_raw_fd(self)->RawFd;}
    pub trait FromRawFd {unsafe fn from_raw_fd(fd:RawFd)->Self;}
    pub trait AsFd {fn as_fd(&self)->BorrowedFd<'_>;}
    #[derive(Debug)]
    #[repr(transparent)]
    pub struct OwnedFd(NonZeroU32);
    // A17 DeserializeArray uses Option<Self> as an uninitialized slot. Preserve
    // a niche without changing the native i32 descriptor ABI. fd+1 fits u32,
    // including i32::MAX; no HANDLE is encoded here.
    fn encode_fd(fd: RawFd) -> NonZeroU32 {
        assert!(fd >= 0, "invalid native descriptor");
        NonZeroU32::new(fd as u32 + 1).expect("nonnegative descriptor plus one")
    }
    fn decode_fd(fd: NonZeroU32) -> RawFd { (fd.get() - 1) as RawFd }
    #[derive(Debug,Clone,Copy)]
    pub struct BorrowedFd<'a>{fd:RawFd,_owner:PhantomData<&'a OwnedFd>}
    impl OwnedFd {
        pub fn try_clone(&self)->io::Result<Self>{self.as_fd().try_clone_to_owned()}
        /// Duplicate a borrowed OS handle into the native descriptor registry. Original unchanged.
        pub fn try_from_handle(handle:&impl AsRawHandle)->io::Result<Self>{
            let mut fd=-1;checked(unsafe{bscp_fd_from_handle(handle.as_raw_handle(),&mut fd)})?;
            if fd<0{return Err(io::Error::other("native success returned invalid descriptor"));}
            Ok(Self(encode_fd(fd)))
        }
        pub fn try_clone_to_handle(&self)->io::Result<OwnedHandle>{self.as_fd().try_clone_to_handle()}
    }
    impl<'a> BorrowedFd<'a>{
        /// # Safety
        /// Caller retains the original native descriptor throughout 'a, including every
        /// concurrent borrower. No thread may close, replace, or recycle that registry
        /// slot while borrowed. It is not a HANDLE or a descriptor in Rust's CRT;
        /// consumers must use this module's native ABI, never libc/another CRT.
        pub unsafe fn borrow_raw(fd:RawFd)->Self{assert!(fd>=0);Self{fd,_owner:PhantomData}}
        pub fn try_clone_to_owned(&self)->io::Result<OwnedFd>{let mut fd=-1;checked(unsafe{bscp_fd_dup(self.fd,&mut fd)})?;if fd<0{return Err(io::Error::other("native invalid duplicate"));}Ok(OwnedFd(encode_fd(fd)))}
        pub fn try_clone_to_handle(&self)->io::Result<OwnedHandle>{
            let mut handle=std::ptr::null_mut();checked(unsafe{bscp_fd_to_handle(self.fd,&mut handle)})?;
            if handle.is_null() || handle as isize == -1{return Err(io::Error::other("native invalid handle"));}
            Ok(unsafe{OwnedHandle::from_raw_handle(handle)})
        }
    }
    impl Drop for OwnedFd{fn drop(&mut self){let r=unsafe{bscp_fd_close(decode_fd(self.0))};if r!=0{log::error!("native Binder descriptor close failed: {r}");}}}
    impl AsRawFd for OwnedFd{fn as_raw_fd(&self)->RawFd{decode_fd(self.0)}}
    impl AsRawFd for BorrowedFd<'_>{fn as_raw_fd(&self)->RawFd{self.fd}}
    impl IntoRawFd for OwnedFd{fn into_raw_fd(self)->RawFd{decode_fd(ManuallyDrop::new(self).0)}}
    impl FromRawFd for OwnedFd{unsafe fn from_raw_fd(fd:RawFd)->Self{Self(encode_fd(fd))}}
    impl AsFd for OwnedFd{fn as_fd(&self)->BorrowedFd<'_>{BorrowedFd{fd:decode_fd(self.0),_owner:PhantomData}}}
    #[cfg(test)]
    mod windows_descriptor_encoding_tests {
        use super::*;
        #[test] fn pure_boundary_encoding() {
            for raw in [0,1,i32::MAX] { assert_eq!(decode_fd(encode_fd(raw)),raw); }
            assert_eq!(std::mem::size_of::<Option<OwnedFd>>(),std::mem::size_of::<OwnedFd>());
            assert_eq!(std::mem::align_of::<Option<OwnedFd>>(),std::mem::align_of::<OwnedFd>());
        }
        #[test] #[should_panic(expected="invalid native descriptor")]
        fn negative_is_rejected_without_ownership() { let _=encode_fd(-1); }
    }
    impl AsFd for BorrowedFd<'_>{fn as_fd(&self)->BorrowedFd<'_>{*self}}
}
