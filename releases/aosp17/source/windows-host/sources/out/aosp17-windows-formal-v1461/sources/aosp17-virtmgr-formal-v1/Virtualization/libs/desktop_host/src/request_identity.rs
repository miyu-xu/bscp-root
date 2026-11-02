//! Windows authenticated Binder dispatch identity. Never a Unix UID or an environment fallback.
use std::{ffi::c_void, io, mem::size_of};
use std::os::windows::io::AsRawHandle;
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CallerIdentity { pub sid: Vec<u8>, pub logon_low:u32, pub logon_high:i32 }
#[repr(C)]
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct FileEvidence { pub version:u32,pub volume_serial:u32,pub file_index:u64,pub links:u32,pub attributes:u32,pub handle_granted:u32,pub caller_granted:u32 }
const _: () = assert!(size_of::<FileEvidence>() == 32);
#[repr(u32)]
#[derive(Clone,Copy,Debug)]
pub enum FileAccess { ReadImage=0, WriteInstance=1, ReadOwnedDirectory=2 }
#[link(name="binder_sdk")]
extern "C" {
 fn bscp_rpc_check_owner()->u32;
 fn bscp_rpc_current_pid(pid:*mut u32)->u32;
 fn bscp_rpc_current_identity(sid:*mut u8,capacity:u32,required:*mut u32,low:*mut u32,high:*mut i32)->u32;
 fn bscp_rpc_check_file(file:*mut c_void,kind:u32,evidence:*mut FileEvidence,size:u32)->u32;
}
fn status(s:u32)->io::Result<()> { if s==0 {Ok(())} else {Err(io::Error::from_raw_os_error(s as i32))} }
fn validate_sid(sid:&[u8])->io::Result<()> {
 if sid.len()<8 || sid[0]!=1 || sid[1]>15 || sid.len()!=8+4*usize::from(sid[1]) {return Err(io::Error::new(io::ErrorKind::InvalidData,"invalid native caller SID"));} Ok(())
}
/// Requires both the real current request token and matching owner SID/logon.
pub fn check_owner()->io::Result<()> { status(unsafe{bscp_rpc_check_owner()}) }
/// Metadata from the authenticated request's retained original process proof.
/// This never opens a PID or falls back to the server/creator process.
pub fn current_pid()->io::Result<u32> {
 let mut pid=0;
 status(unsafe{bscp_rpc_current_pid(&mut pid)})?;
 if pid==0 {return Err(io::Error::new(io::ErrorKind::InvalidData,"native request returned zero PID"));}
 Ok(pid)
}


/// The C++ dispatch guard owns the immutable token snapshot for the entire call.
/// Absence of that guard is ERROR_NO_TOKEN; this does not query a process token.
pub fn current_caller()->io::Result<CallerIdentity> {
 let(mut required,mut low,mut high)=(0,0,0);let mut sid=[0u8;68];
 status(unsafe{bscp_rpc_current_identity(sid.as_mut_ptr(),sid.len() as u32,&mut required,&mut low,&mut high)})?;
 let size=usize::try_from(required).map_err(|_|io::Error::other("SID size overflow"))?;
 if size>sid.len(){return Err(io::Error::other("native SID size exceeds bounded buffer"));}
 validate_sid(&sid[..size])?;Ok(CallerIdentity{sid:sid[..size].to_vec(),logon_low:low,logon_high:high})
}
/// Checks the actual borrowed HANDLE, never a reopened path. The owner must keep
/// it live and must not replace/close the handle concurrently. Path/ancestor
/// binding remains a separate responsibility; success does not establish it.
pub fn check_file(file:&impl AsRawHandle,kind:FileAccess)->io::Result<FileEvidence> {
 let mut evidence=FileEvidence::default();
 status(unsafe{bscp_rpc_check_file(file.as_raw_handle(),kind as u32,&mut evidence,size_of::<FileEvidence>() as u32)})?;
 if evidence.version!=1{return Err(io::Error::new(io::ErrorKind::InvalidData,"unknown file evidence version"));}
 Ok(evidence)
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn absent_scope_never_supplies_pid(){assert_eq!(current_pid().unwrap_err().raw_os_error(),Some(1008));}
 #[test] fn absent_scope_never_becomes_owner(){assert_eq!(current_caller().unwrap_err().raw_os_error(),Some(1008));}
 #[test] fn absent_scope_never_grants_owner(){assert_eq!(check_owner().unwrap_err().raw_os_error(),Some(1008));}
 #[test] fn malformed_sid_rejected(){for s in [vec![],vec![0;8],vec![1,16,0,0,0,0,0,0],vec![1,1,0,0,0,0,0,0]]{assert!(validate_sid(&s).is_err());}}
 #[test] fn exact_sid_shape(){let s=[1,1,0,0,0,0,0,5,18,0,0,0];assert!(validate_sid(&s).is_ok());}
 #[test] fn abi_layout(){assert_eq!(size_of::<FileEvidence>(),32);assert_eq!(std::mem::offset_of!(FileEvidence,file_index),8);assert_eq!(std::mem::offset_of!(FileEvidence,caller_granted),28);}
}
