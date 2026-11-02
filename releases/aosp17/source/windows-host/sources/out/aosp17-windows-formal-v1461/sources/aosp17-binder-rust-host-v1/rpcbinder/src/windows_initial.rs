//! Owned major3 Windows initial association. Configured does not authenticate a peer.
//! Every native setup connection verifies its own association to the borrowed
//! original launch process. There is no PID discovery or raw transport fallback.
use binder::{SpIBinder,unstable_api::new_spibinder};
use std::{ffi::c_void,io,os::windows::io::{AsRawHandle,BorrowedHandle},ptr,
    sync::{Mutex,atomic::{AtomicU64,Ordering}},time::Duration};

#[link(name="WindowsLocalRpcBridge",kind="raw-dylib")]
extern "system" {
    fn InitialCreate(endpoint:*const u16,original_launch:*mut c_void,out:*mut *mut c_void)->u32;
    fn InitialRelease(owned:*mut *mut c_void)->u32;
}
#[link(name="binder_rpc_unstable_host")]
extern "C" {
    fn ARpcSession_newWindowsInitial(association:*mut *mut c_void,out:*mut *mut c_void)->i32;
    fn ARpcSession_setupWindowsInitial(session:*mut c_void,out:*mut *mut c_void)->i32;
    fn ARpcSession_shutdownJoinWindowsInitial(session:*mut c_void,ms:u32,state:*mut u32,status:*mut i32)->i32;
    fn ARpcSession_releaseWindowsInitial(owned:*mut *mut c_void)->i32;
    fn ARpcSession_setFileDescriptorTransportMode(session:*mut c_void,mode:i32);
    fn ARpcSession_setMaxIncomingThreads(session:*mut c_void,threads:usize);
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct InitialObservation {pub operation:i32,pub state:u32,pub setup_status:i32}
impl InitialObservation {
    /// Only native resource closure. A sticky setup/task error is independent.
    pub fn closed(self)->bool {self.operation==0&&self.state==5}
}
struct Resources {
    id:u64,association:*mut c_void,session:*mut c_void,unknown_root:*mut c_void,active:usize,
    setup_attempted:bool,stop_requested:bool,closed:bool,ambiguous:bool,
    last:InitialObservation,next:Option<Box<Resources>>,
}
// Opaque pointers are always protected by the state lock; only verified native
// concurrent setup/stop calls borrow the stable session while active > 0.
unsafe impl Send for Resources {}
static NEXT_ID:AtomicU64=AtomicU64::new(1);
static RETAINED:Mutex<Option<Box<Resources>>>=Mutex::new(None);
fn retain(mut r:Box<Resources>)->u64 {
    let id=r.id;let mut list=RETAINED.lock().unwrap_or_else(|p|p.into_inner());
    r.next=list.take();*list=Some(r);id
}
pub fn retained_initial_owner_ids()->Vec<u64>{
    let list=RETAINED.lock().unwrap_or_else(|p|p.into_inner());let mut p=list.as_deref();let mut ids=Vec::new();
    while let Some(r)=p {ids.push(r.id);p=r.next.as_deref();}ids
}
fn error(operation:&str,status:impl std::fmt::Display)->io::Error {io::Error::other(format!("{operation}: {status}"))}
fn milliseconds(budget:Duration)->io::Result<u32>{
    let v=u32::try_from(budget.as_millis()).map_err(|_|io::ErrorKind::InvalidInput)?;
    if v==u32::MAX{return Err(io::ErrorKind::InvalidInput.into());}Ok(v)
}
pub struct WindowsInitialSession {resources:Mutex<Option<Box<Resources>>>}
struct Borrow<'a> {owner:&'a WindowsInitialSession,session:*mut c_void}
impl Drop for Borrow<'_>{fn drop(&mut self){
    let mut g=self.owner.resources.lock().unwrap_or_else(|p|p.into_inner());
    if let Some(r)=g.as_mut(){debug_assert!(r.active>0);r.active-=1;}
}}
impl WindowsInitialSession {
    /// Construct a pending association, not an authenticated connection.
    ///
    /// # Safety
    /// `original_launch` must be the actual still-owned HANDLE returned by this
    /// launch's CreateProcess, never OpenProcess(PID), a stored number or a
    /// service-returned handle. The native factory duplicates only QUERY and
    /// SYNCHRONIZE rights, non-inheritable; the caller retains its original.
    pub unsafe fn configured(endpoint:&str,original_launch:BorrowedHandle<'_>)->io::Result<Self>{
        if endpoint.is_empty()||endpoint.encode_utf16().any(|x|x==0){return Err(io::ErrorKind::InvalidInput.into());}
        let text:Vec<u16>=endpoint.encode_utf16().chain([0]).collect();
        let id=NEXT_ID.fetch_update(Ordering::Relaxed,Ordering::Relaxed,|v|v.checked_add(1)).map_err(|_|error("owner IDs","exhausted"))?;
        // Preallocate exact-owner retention before any native side effect.
        let mut r=Box::new(Resources{id,association:ptr::null_mut(),session:ptr::null_mut(),unknown_root:ptr::null_mut(),active:0,
            setup_attempted:false,stop_requested:false,closed:false,ambiguous:false,
            last:InitialObservation{operation:0,state:0,setup_status:0},next:None});
        let status=InitialCreate(text.as_ptr(),original_launch.as_raw_handle(),&mut r.association);
        if status!=0 {
            if !r.association.is_null(){r.ambiguous=true;let id=retain(r);return Err(error("InitialCreate UNKNOWN retained owner",format!("{id}, status={status}")));}
            // A null output does not prove native partial-start rollback.
            return Err(error("InitialCreate failed; native partial outcome not REAPED",status));
        }
        if r.association.is_null(){r.ambiguous=true;let id=retain(r);return Err(error("InitialCreate contract retained owner",id));}
        let status=ARpcSession_newWindowsInitial(&mut r.association,&mut r.session);
        if status!=0 {
            r.ambiguous=!r.session.is_null()||r.association.is_null();
            let id=retain(r);return Err(error("session factory retained owner",format!("{id}, status={status}")));
        }
        if r.session.is_null()||!r.association.is_null(){r.ambiguous=true;let id=retain(r);return Err(error("session ownership contract retained owner",id));}
        // Existing C enum UNIX=1 is the logical descriptor mode. Windows moves
        // typed HANDLE capabilities through NDR, not a Unix socket/CRT bridge.
        ARpcSession_setFileDescriptorTransportMode(r.session,1);
        ARpcSession_setMaxIncomingThreads(r.session,2);
        Ok(Self{resources:Mutex::new(Some(r))})
    }
    pub fn recover_retained(id:u64)->io::Result<Self>{
        let mut list=RETAINED.lock().unwrap_or_else(|p|p.into_inner());let mut link=&mut *list;
        loop {
            if link.as_ref().is_some_and(|r|r.id==id){let mut r=link.take().expect("checked node");*link=r.next.take();return Ok(Self{resources:Mutex::new(Some(r))});}
            match link {Some(r)=>link=&mut r.next,None=>return Err(io::ErrorKind::NotFound.into())}
        }
    }
    fn borrow(&self,setup:bool)->io::Result<Borrow<'_>>{
        let mut g=self.resources.lock().map_err(|_|error("owner mutex","poisoned"))?;
        let r=g.as_mut().ok_or_else(||error("borrow","owner absent"))?;
        if r.ambiguous||r.session.is_null(){return Err(error("borrow","unverified/released session"));}
        if setup {
            if r.setup_attempted||r.stop_requested||r.closed{return Err(error("setup","one-shot/stop gate"));}
            r.setup_attempted=true;
        }else {r.stop_requested=true;}
        r.active=r.active.checked_add(1).ok_or_else(||error("borrowers","overflow"))?;
        Ok(Borrow{owner:self,session:r.session})
    }
    /// One setup attempt. Only successful native actual-peer association returns
    /// an owned Binder reference. Mutex is not held over the blocking setup, so
    /// an external controller can request cancellation on this same owner.
    pub fn setup(&self)->io::Result<SpIBinder>{
        let call=self.borrow(true)?;let mut root=ptr::null_mut();
        let status=unsafe{ARpcSession_setupWindowsInitial(call.session,&mut root)};
        if status!=0 {
            if !root.is_null(){
                let mut g=self.resources.lock().unwrap_or_else(|p|p.into_inner());
                if let Some(r)=g.as_mut(){r.ambiguous=true;r.unknown_root=root;}
                // An unexpected pointer on failure has no documented ownership
                // transfer. Do not decStrong an unproven reference.
                return Err(error("setup contract UNKNOWN root pointer",status));
            }
            return Err(error("setup failed (no replay)",status));
        }
        // Native AIBinder_fromPlatformBinder explicitly transfers one +1 ref.
        if root.is_null(){
            let mut g=self.resources.lock().unwrap_or_else(|p|p.into_inner());
            if let Some(r)=g.as_mut(){r.ambiguous=true;}
            return Err(error("setup","success returned null root"));
        }
        unsafe{new_spibinder(root.cast())}.ok_or_else(||error("setup","owned root conversion failed"))
    }
    /// May run concurrently with setup. The native API latches cancellation;
    /// callback/self-join WOULD_BLOCK is not closure. Observation budget is not
    /// a hard OS deadline and no business request is replayed.
    pub fn shutdown_join(&self,budget:Duration)->io::Result<InitialObservation>{
        let ms=milliseconds(budget)?;let call=self.borrow(false)?;let mut state=0;let mut sticky=0;
        let operation=unsafe{ARpcSession_shutdownJoinWindowsInitial(call.session,ms,&mut state,&mut sticky)};
        let observed=InitialObservation{operation,state,setup_status:sticky};
        let mut g=self.resources.lock().map_err(|_|error("owner mutex","poisoned"))?;
        let r=g.as_mut().ok_or_else(||error("observe","owner absent"))?;
        r.last=observed;r.closed|=observed.closed();Ok(observed)
    }
    /// # Safety
    /// All returned Binder proxies/clones, callbacks and native borrowers must
    /// be quiescent; no external session API can run. &mut alone cannot prove
    /// quiescence of independently cloned Binder objects. Closed is resource
    /// closure only, not success of the earlier setup/business operation.
    pub unsafe fn release_closed(&mut self)->io::Result<()> {
        let slot=self.resources.get_mut().map_err(|_|error("owner mutex","poisoned"))?;
        let r=slot.as_mut().ok_or_else(||error("release","owner absent"))?;
        if r.ambiguous||r.active!=0{return Err(error("release","UNKNOWN or active borrowers"));}
        if !r.session.is_null(){
            if !r.closed{return Err(error("release","not Closed"));}
            let rc=ARpcSession_releaseWindowsInitial(&mut r.session);
            if rc!=0 {if r.session.is_null(){r.ambiguous=true;}return Err(error("session release",rc));}
            if !r.session.is_null(){r.ambiguous=true;return Err(error("release","success did not consume session"));}
        }
        if !r.association.is_null(){
            let rc=InitialRelease(&mut r.association);
            if rc!=0 {if r.association.is_null(){r.ambiguous=true;}return Err(error("association release",rc));}
            if !r.association.is_null(){r.ambiguous=true;return Err(error("release","success did not consume association"));}
        }
        Ok(())
    }
}
impl Drop for WindowsInitialSession {fn drop(&mut self){
    let slot=self.resources.get_mut().unwrap_or_else(|p|p.into_inner());
    if let Some(r)=slot.take(){if !r.session.is_null()||!r.association.is_null()||r.ambiguous {
        let id=retain(r);log::error!("Windows initial session retained under owner {id}, not REAPED");
    }}
}}
