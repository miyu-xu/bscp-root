//! The existing Windows Guest byte-pipe endpoint, bound to the original child.
//! Raw Guest RPC carries no desktop NDR caller identity or delegated credentials.
use std::{ffi::c_void,io,os::windows::io::{BorrowedHandle,AsRawHandle},ptr::NonNull};
use binder::{SpIBinder,unstable_api::AsNative};
extern "C" {
    fn ARpcServer_newVsock(service:*mut c_void,cid:u32,port:u32,assigned:*mut u32)->*mut c_void;
    fn ARpcServer_start(server:*mut c_void);
    fn ARpcServer_shutdown(server:*mut c_void)->bool;
    fn ARpcServer_free(server:*mut c_void);
    fn BSCP_RpcServer_bindGuestProcess(server:*mut c_void,original:*mut c_void)->i32;
}
pub struct GuestRpcServer {server:NonNull<c_void>,bound:bool,started:bool}
// All access is through the VmInstance mutex. Native server owns its session
// synchronization and duplicates the borrowed original process before returning.
unsafe impl Send for GuestRpcServer {}
impl GuestRpcServer {
    pub fn new(mut service:SpIBinder,cid:u32)->io::Result<Self>{
        let mut assigned=0;
        let raw=unsafe{ARpcServer_newVsock(service.as_native_mut().cast(),cid,cid,&mut assigned)};
        let server=NonNull::new(raw).ok_or_else(||io::Error::other("Windows Guest listener creation failed"))?;
        let owner=Self{server,bound:false,started:false};
        if assigned!=cid{return Err(io::Error::other("Guest listener port mismatch"));}
        Ok(owner)
    }
    pub fn bind(&mut self,original:BorrowedHandle<'_>)->io::Result<()>{
        if self.bound||self.started{return Err(io::Error::other("Guest listener already bound/started"));}
        let status=unsafe{BSCP_RpcServer_bindGuestProcess(self.server.as_ptr(),original.as_raw_handle())};
        if status!=0{return Err(io::Error::other(format!("Original Guest process binding failed: {status}")));}
        self.bound=true;Ok(())
    }
    pub fn start(&mut self)->io::Result<()>{
        if !self.bound||self.started{return Err(io::Error::other("Guest listener requires original bound process and single start"));}
        self.started=true;
        unsafe{ARpcServer_start(self.server.as_ptr())};Ok(())
    }
    pub fn shutdown(&self)->io::Result<()>{
        if !self.started{return Ok(());}
        if unsafe{ARpcServer_shutdown(self.server.as_ptr())}{Ok(())}
        else{Err(io::Error::other("Guest listener shutdown not confirmed"))}
    }
}
impl Drop for GuestRpcServer {fn drop(&mut self){unsafe{ARpcServer_free(self.server.as_ptr())}}}
