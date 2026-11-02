//! One original Windows control listener and bound client connection.
//! No VM request is sent here. The caller must send at most once after bind.
use std::{ffi::OsStr,fs::File,io,os::windows::{ffi::OsStrExt,io::{AsRawHandle,FromRawHandle}},sync::{Mutex,MutexGuard}};
use windows_sys::Win32::{Foundation::{GetHandleInformation,INVALID_HANDLE_VALUE,HANDLE_FLAG_INHERIT},Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE,FILE_FLAG_OVERLAPPED,PIPE_ACCESS_DUPLEX},System::{Pipes::{CreateNamedPipeW,GetNamedPipeClientProcessId,GetNamedPipeServerProcessId,PIPE_READMODE_MESSAGE,PIPE_REJECT_REMOTE_CLIENTS,PIPE_TYPE_MESSAGE,PIPE_WAIT},Threading::GetCurrentProcessId}};
pub enum GuestListenerResult {Created,Rejected(anyhow::Error),Unknown(anyhow::Error)}
pub struct ControlEndpoint{listener:File,name:String,creator:u32,connection:Mutex<()>}
pub struct BoundClient<'a>{pipe:base::named_pipes::PipeConnection,_gate:MutexGuard<'a,()>}
impl BoundClient<'_> { pub fn pipe(&self)->&base::named_pipes::PipeConnection {&self.pipe} }
impl ControlEndpoint {
 pub fn create(name:&str)->io::Result<Self>{
  if !name.starts_with(r"\\.\pipe\bscp-control-")||name.contains('\0')||name[r"\\.\pipe\".len()..].contains(['\\','/']) {return Err(io::ErrorKind::InvalidInput.into());}
  let wide:Vec<u16>=OsStr::new(name).encode_wide().chain(Some(0)).collect();
  // Null SECURITY_ATTRIBUTES: current token's default DACL, noninheritable.
  // MaxInstances=1 plus FIRST_PIPE_INSTANCE prevents name replacement while held.
  let raw=unsafe{CreateNamedPipeW(wide.as_ptr(),PIPE_ACCESS_DUPLEX|FILE_FLAG_FIRST_PIPE_INSTANCE|FILE_FLAG_OVERLAPPED,PIPE_TYPE_MESSAGE|PIPE_READMODE_MESSAGE|PIPE_WAIT|PIPE_REJECT_REMOTE_CLIENTS,1,1048576,1048576,0,std::ptr::null())};
  if raw==INVALID_HANDLE_VALUE{return Err(io::Error::last_os_error());}
  let listener=unsafe{File::from_raw_handle(raw as _)};let mut flags=0;
  if unsafe{GetHandleInformation(raw,&mut flags)}==0{return Err(io::Error::last_os_error());}
  if flags&HANDLE_FLAG_INHERIT!=0{return Err(io::Error::other("control listener unexpectedly inheritable"));}
  Ok(Self{listener,name:name.to_owned(),creator:unsafe{GetCurrentProcessId()},connection:Mutex::new(())})
 }
 pub fn listener(&self)->&File{&self.listener}
 /// Sends the official Exit request once. Caller owns the lifecycle attempt gate.
 pub fn request_exit(&self)->anyhow::Result<()> {
  use anyhow::{Context,bail};
  let bound=self.connect().context("original control endpoint connection failed")?;
  let tube=base::PipeTube::from(bound.pipe.try_clone()?,None);
  tube.send(&vm_control::VmRequest::Exit).context("Exit send failed/unknown; do not repeat")?;
  match tube.recv::<vm_control::VmResponse>().context("Exit response failed/unknown; do not repeat")? {
   vm_control::VmResponse::Ok => Ok(()),
   other => bail!("Exit rejected: {other:?}"),
  }
 }
 /// One listener-creation request; Ok proves a listener, not Guest handshake or business success.
 pub fn request_guest_listener(&self,port:u32)->GuestListenerResult {
  use GuestListenerResult::*;
  let bound=match self.connect(){Ok(p)=>p,Err(e)=>return Rejected(e.into())};
  let pipe=match bound.pipe.try_clone(){Ok(p)=>p,Err(e)=>return Rejected(e.into())};
  let tube=base::PipeTube::from(pipe,None);
  if let Err(e)=tube.send(&vm_control::VmRequest::ConnectVsock{port}){return Unknown(e.into());}
  match tube.recv::<vm_control::VmResponse>(){
   Ok(vm_control::VmResponse::Ok)=>Created,
   Ok(vm_control::VmResponse::Err(e))=>Rejected(anyhow::anyhow!("Guest listener rejected: {e:?}")),
   Ok(vm_control::VmResponse::ErrString(e))=>Rejected(anyhow::anyhow!("Guest listener rejected: {e}")),
   Ok(other)=>Unknown(anyhow::anyhow!("Unexpected Guest listener response; outcome unknown: {other:?}")),
   Err(e)=>Unknown(e.into()),
  }
 }
 pub fn connect(&self)->io::Result<BoundClient<'_>>{
  use base::AsRawDescriptor;
  let gate=self.connection.lock().map_err(|_|io::Error::other("control connection gate poisoned"))?;
  let pipe=base::named_pipes::create_client_pipe(&self.name,&base::named_pipes::FramingMode::Message,&base::named_pipes::BlockingMode::Wait,false)?;
  let mut server=0;let mut client=0;
  if unsafe{GetNamedPipeServerProcessId(pipe.as_raw_descriptor() as _,&mut server)}==0{return Err(io::Error::last_os_error());}
  if unsafe{GetNamedPipeClientProcessId(self.listener.as_raw_handle() as _,&mut client)}==0{return Err(io::Error::last_os_error());}
  if server!=self.creator || client!=unsafe{GetCurrentProcessId()} {return Err(io::Error::new(io::ErrorKind::PermissionDenied,"control pipe original endpoint binding mismatch"));}
  Ok(BoundClient{pipe,_gate:gate})
 }
}
