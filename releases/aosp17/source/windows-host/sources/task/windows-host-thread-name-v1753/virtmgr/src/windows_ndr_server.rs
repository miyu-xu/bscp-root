//! Windows NDR service owner. Join-loop READY is not peer authentication.
//! Only this successful listener creator may stop its listener. Dispatch threads
//! request stop; the external owner drives shutdown/join and preserves failures.
use binder::{SpIBinder,unstable_api::AsNative};
use std::{ffi::c_void,io,ptr,sync::{Mutex,Condvar,atomic::{AtomicBool,AtomicU64,Ordering}},time::Duration};
#[link(name="binder_rpc_unstable_host")]
extern "C" {
 fn ARpcServer_newWindowsNdr(root:*mut c_void,cap:*mut *mut c_void,out:*mut *mut c_void)->i32;
 fn ARpcServer_startWindowsNdr(server:*mut c_void)->i32;
 fn ARpcServer_waitWindowsNdrReady(server:*mut c_void,ms:u32,state:*mut u32,status:*mut i32)->i32;
 fn ARpcServer_shutdownJoinWindowsNdr(server:*mut c_void,ms:u32,state:*mut u32,status:*mut i32)->i32;
 fn ARpcServer_releaseWindowsNdr(server:*mut *mut c_void)->i32;
 fn ARpcServer_setMaxThreads(server:*mut c_void,count:usize);
 // AOSP binder_rpc_unstable.hpp enum class uses the native C int ABI.
 fn ARpcServer_setSupportedFileDescriptorTransportModes(server:*mut c_void,modes:*const i32,count:usize);
}
#[link(name="WindowsLocalRpcBridge",kind="raw-dylib")]
extern "system" {
 fn WlrStartOwnedListener(endpoint:*const u16,out:*mut *mut c_void)->u32;
 fn WlrReleaseAcceptor(cap:*mut *mut c_void)->u32;
 fn WlrStop()->u32;
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Observation {pub operation:i32,pub state:u32,pub loop_status:i32}
impl Observation {
 pub fn ready(self)->bool{self.operation==0&&self.state==2&&self.loop_status==0}
 pub fn joined(self)->bool{self.operation==0&&self.state==5}
 fn join_pending(self)->bool{self.operation == -110 && matches!(self.state,1|3|4)}
}
// Allocate this node before creating native resources. Failure and Drop move it
// to the exact-owner registry without allocation, never reconstructing by PID.
struct Resources {
 id:u64,server:*mut c_void,cap:*mut c_void,listener_owned:bool,
 last:Observation,joined:bool,ambiguous:bool,next:Option<Box<Resources>>,
}
unsafe impl Send for Resources {}
static NEXT_ID:AtomicU64=AtomicU64::new(1);
static RETAINED:Mutex<Option<Box<Resources>>>=Mutex::new(None);
fn retain(mut value:Box<Resources>)->u64 {
 let id=value.id;
 let mut list=RETAINED.lock().unwrap_or_else(|p|p.into_inner());
 value.next=list.take();*list=Some(value);id
}
pub fn retained_owner_ids()->Vec<u64>{
 let list=RETAINED.lock().unwrap_or_else(|p|p.into_inner());
 let mut cursor=list.as_deref();let mut ids=Vec::new();
 while let Some(value)=cursor{ids.push(value.id);cursor=value.next.as_deref();}ids
}
pub struct WindowsNdrServer {inner:Mutex<Option<Box<Resources>>>,intent:Mutex<()>,stop_requested:AtomicBool,wake:Condvar}
fn failure(where_:&str,code:impl std::fmt::Display)->io::Error {io::Error::other(format!("{where_}: {code}"))}
fn millis(d:Duration)->io::Result<u32>{
 let ms=u32::try_from(d.as_millis()).map_err(|_|io::Error::new(io::ErrorKind::InvalidInput,"observation budget too large"))?;
 if ms==u32::MAX{return Err(io::ErrorKind::InvalidInput.into());}Ok(ms)
}
impl WindowsNdrServer {
 fn from_resources(value:Box<Resources>,stop:bool)->Self {
  Self{inner:Mutex::new(Some(value)),intent:Mutex::new(()),stop_requested:AtomicBool::new(stop),wake:Condvar::new()}
 }
 pub fn recover_retained(id:u64)->io::Result<Self>{
  let mut list=RETAINED.lock().unwrap_or_else(|p|p.into_inner());let mut link=&mut *list;
  loop {
   if link.as_ref().is_some_and(|v|v.id==id){
    let mut value=link.take().expect("checked exact node");*link=value.next.take();
    return Ok(Self::from_resources(value,true));
   }
   match link {Some(value)=>link=&mut value.next,None=>return Err(io::ErrorKind::NotFound.into())}
  }
 }
 pub fn new_listener(mut root:SpIBinder,endpoint:&str)->io::Result<Self>{
  Self::create(root.as_native_mut().cast(),endpoint)
 }
 fn create(root:*mut c_void,endpoint:&str)->io::Result<Self>{
  if endpoint.is_empty()||endpoint.encode_utf16().any(|x|x==0){return Err(io::ErrorKind::InvalidInput.into());}
  let text:Vec<u16>=endpoint.encode_utf16().chain([0]).collect();
  let id=NEXT_ID.fetch_update(Ordering::Relaxed,Ordering::Relaxed,|v|v.checked_add(1)).map_err(|_|failure("owner IDs","exhausted"))?;
  let mut value=Box::new(Resources{id,server:ptr::null_mut(),cap:ptr::null_mut(),listener_owned:false,
   last:Observation{operation:0,state:0,loop_status:0},joined:false,ambiguous:false,next:None});
  let status=unsafe{WlrStartOwnedListener(text.as_ptr(),&mut value.cap)};
  if status!=0{
   if !value.cap.is_null(){value.ambiguous=true;let id=retain(value);return Err(failure("listener failure retained owner",format!("{id}, status={status}")));}
   return Err(failure("listener creation",status));
  }
  value.listener_owned=true;
  let result=unsafe{ARpcServer_newWindowsNdr(root,&mut value.cap,&mut value.server)};
  if result!=0 {
   if !value.server.is_null()||value.cap.is_null(){value.ambiguous=true;}
   // Keep the exact failed owner for explicit cleanup, including the real
   // unconsumed acceptor. No Stop before verified acceptor/server release.
   let id=retain(value);return Err(failure("server factory retained owner",format!("{id}, factory={result}")));
  }
  if value.server.is_null()||!value.cap.is_null(){value.ambiguous=true;let id=retain(value);return Err(failure("native ownership contract violated; retained owner",id));}
  Ok(Self::from_resources(value,false))
 }
 #[cfg(test)] pub fn reject_null_root_for_test(endpoint:&str)->io::Result<Self>{Self::create(ptr::null_mut(),endpoint)}
 pub fn start(&self,max_threads:usize)->io::Result<()> {
  if max_threads==0{return Err(io::ErrorKind::InvalidInput.into());}
  let _intent=self.intent.lock().map_err(|_|failure("intent mutex","poisoned"))?;
  let mut guard=self.inner.lock().map_err(|_|failure("owner mutex","poisoned"))?;
  let g=guard.as_mut().ok_or_else(||failure("start","owner absent"))?;
  if self.stop_requested.load(Ordering::Acquire){return Err(failure("start","stop already requested"));}
  if g.ambiguous||g.server.is_null()||g.last.state!=0{return Err(failure("start","not verified New"));}
  // UNIX=1 is the existing RPC logical descriptor mode. On this Windows
  // factory the actual transport is typed NDR HANDLE transfer, not Unix sockets.
  // Match the original Host FD contract; NONE is not silently permitted here.
  let modes=[1i32];
  unsafe{ARpcServer_setSupportedFileDescriptorTransportModes(g.server,modes.as_ptr(),modes.len())};
  unsafe{ARpcServer_setMaxThreads(g.server,max_threads)};
  let status=unsafe{ARpcServer_startWindowsNdr(g.server)};
  if status!=0{return Err(failure("start",status));}g.last.state=1;Ok(())
 }
 pub fn wait_ready(&self,budget:Duration)->io::Result<Observation>{
  let ms=millis(budget)?;
  let mut guard=self.inner.lock().map_err(|_|failure("owner mutex","poisoned"))?;
  let g=guard.as_mut().ok_or_else(||failure("observe","owner absent"))?;
  if g.ambiguous||g.server.is_null(){return Err(failure("observe","server not available"));}
  let mut state=0;let mut sticky=0;let op=unsafe{ARpcServer_waitWindowsNdrReady(g.server,ms,&mut state,&mut sticky)};
  g.last=Observation{operation:op,state,loop_status:sticky};
  // A stop intent is not a native loop error, but it prevents publishing READY.
  if self.stop_requested.load(Ordering::Acquire)&&g.last.ready(){return Err(failure("observe","stop requested; READY not publishable"));}
  Ok(g.last)
 }
 /// Dispatch callback records intent only, never waits on its own dispatch.
 pub fn shutdown(&self)->io::Result<()> {let _intent=self.intent.lock().map_err(|_|failure("intent mutex","poisoned"))?;self.stop_requested.store(true,Ordering::Release);self.wake.notify_all();Ok(())}
 pub fn shutdown_join(&self,budget:Duration)->io::Result<Observation>{
  let ms=millis(budget)?;self.shutdown()?;
  let mut guard=self.inner.lock().map_err(|_|failure("owner mutex","poisoned"))?;
  let g=guard.as_mut().ok_or_else(||failure("shutdown/join","owner absent"))?;
  if g.ambiguous||g.server.is_null(){return Err(failure("shutdown/join","server not available"));}
  let mut state=0;let mut sticky=0;let op=unsafe{ARpcServer_shutdownJoinWindowsNdr(g.server,ms,&mut state,&mut sticky)};
  g.last=Observation{operation:op,state,loop_status:sticky};g.joined|=g.last.joined();Ok(g.last)
 }
 pub fn drive_until_joined(&self)->io::Result<Observation>{
  loop {
   if self.stop_requested.load(Ordering::Acquire) {
    let value=self.shutdown_join(Duration::from_millis(250))?;
    if value.joined(){return Ok(value);}
    if !value.join_pending(){return Err(failure("shutdown/join retained owner",format!("{value:?}")));}
   } else {
    let value=self.wait_ready(Duration::ZERO)?;
    if matches!(value.state,3..=5){self.shutdown()?;continue;}
    if !value.ready() && !(value.operation == -110 && matches!(value.state,0|1)){
     return Err(failure("loop observation retained owner",format!("{value:?}")));
    }
   }
   let g=self.inner.lock().map_err(|_|failure("owner mutex","poisoned"))?;
   let _=self.wake.wait_timeout(g,Duration::from_millis(100)).map_err(|_|failure("owner mutex","poisoned"))?;
  }
 }
 /// External controller only, after native borrowers are quiescent. Also
 /// cleans a retained failed factory's unconsumed acceptor, without starting it.
 pub fn release_joined(&self)->io::Result<()> {
  let mut guard=self.inner.lock().map_err(|_|failure("owner mutex","poisoned"))?;
  let g=guard.as_mut().ok_or_else(||failure("release","owner absent"))?;
  if g.ambiguous{return Err(failure("release","ambiguous native contract; retained for diagnosis"));}
  if !g.server.is_null(){
   if !g.joined{return Err(failure("release","not Joined"));}
   let rc=unsafe{ARpcServer_releaseWindowsNdr(&mut g.server)};if rc!=0{return Err(failure("release",rc));}
   if !g.server.is_null(){g.ambiguous=true;return Err(failure("release","success did not consume server"));}
  }
  if !g.cap.is_null(){let rc=unsafe{WlrReleaseAcceptor(&mut g.cap)};if rc!=0{return Err(failure("acceptor release",rc));}
   if !g.cap.is_null(){g.ambiguous=true;return Err(failure("release","success did not consume acceptor"));}}
  if g.listener_owned {let rc=unsafe{WlrStop()};if rc!=0{return Err(failure("listener stop",rc));}g.listener_owned=false;}
  Ok(())
 }
}
impl Drop for WindowsNdrServer {
 fn drop(&mut self){
  let slot=self.inner.get_mut().unwrap_or_else(|p|p.into_inner());
  if let Some(value)=slot.take(){if !value.server.is_null()||!value.cap.is_null()||value.listener_owned {
   let id=retain(value);log::error!("NDR resources retained under exact owner {id}; not REAPED");
  }}
 }
}
