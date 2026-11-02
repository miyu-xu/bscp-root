//! Original-handle private directory and trace/lease creation.
//! Extracted unchanged security checks shared by service and client.
//! Opens only the approved existing root; never repairs or changes ACLs.
use std::{fs::File,io,mem::{size_of,MaybeUninit},path::{Path,PathBuf}};
use std::os::windows::{ffi::OsStrExt,io::{AsRawHandle,FromRawHandle,OwnedHandle}};
use windows_sys::Win32::{Foundation::*,Security::*,Storage::FileSystem::*,System::{Threading::*,IO::OVERLAPPED}};
fn denied(s:&str)->io::Error{io::Error::new(io::ErrorKind::PermissionDenied,s)}
fn key_valid(key:&str)->bool{!key.is_empty()&&key.len()<=200&&key.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-')}
fn process_sid()->io::Result<Vec<u8>>{
 let mut ambient=0;
 if unsafe{OpenThreadToken(GetCurrentThread(),TOKEN_QUERY,1,&mut ambient)}!=0{unsafe{CloseHandle(ambient)};return Err(denied("lease allocation cannot run impersonated"));}
 if unsafe{GetLastError()}!=ERROR_NO_TOKEN{return Err(io::Error::last_os_error());}
 let mut raw=0;if unsafe{OpenProcessToken(GetCurrentProcess(),TOKEN_QUERY,&mut raw)}==0{return Err(io::Error::last_os_error());}
 let token=unsafe{OwnedHandle::from_raw_handle(raw as _)};let mut bytes=[0usize;128];let mut n=0;
 if unsafe{GetTokenInformation(token.as_raw_handle() as _,TokenUser,bytes.as_mut_ptr().cast(),size_of_val(&bytes) as u32,&mut n)}==0{return Err(io::Error::last_os_error());}
 if n as usize>size_of_val(&bytes)||(n as usize)<size_of::<TOKEN_USER>(){return Err(denied("invalid owner token"));}
 let u=unsafe{&*bytes.as_ptr().cast::<TOKEN_USER>()};let start=bytes.as_ptr() as usize;let ptr=u.User.Sid as usize;
 if ptr<start||ptr.checked_add(8).filter(|x|*x<=start+n as usize).is_none(){return Err(denied("invalid SID extent"));}
 let header=unsafe{std::slice::from_raw_parts(ptr as *const u8,8)};let len=8+4*usize::from(header[1]);
 if header[0]!=1||header[1]>15||ptr.checked_add(len).filter(|x|*x<=start+n as usize).is_none(){return Err(denied("invalid owner SID"));}
 Ok(unsafe{std::slice::from_raw_parts(ptr as *const u8,len)}.to_vec())
}
fn trusted(sid:*mut std::ffi::c_void,owner:&[u8])->bool{
 unsafe{EqualSid(sid,owner.as_ptr() as _)!=0 || IsWellKnownSid(sid,WinLocalSystemSid)!=0 || IsWellKnownSid(sid,WinBuiltinAdministratorsSid)!=0}
}
fn check_security(handle:&impl AsRawHandle,owner:&[u8],private:bool)->io::Result<()>{check_security_impl(handle,owner,private,false)}
fn check_security_impl(handle:&impl AsRawHandle,owner:&[u8],private:bool,creating_child:bool)->io::Result<()>{check_security_details(handle,owner,private,creating_child,false)}
fn check_security_details(handle:&impl AsRawHandle,owner:&[u8],private:bool,creating_child:bool,container_child:bool)->io::Result<()>{
 let mut needed=0;
 unsafe{GetKernelObjectSecurity(handle.as_raw_handle() as _,OWNER_SECURITY_INFORMATION|DACL_SECURITY_INFORMATION,std::ptr::null_mut(),0,&mut needed)};
 if unsafe{GetLastError()}!=ERROR_INSUFFICIENT_BUFFER||needed==0||needed>65536{return Err(denied("unavailable or oversized security descriptor"));}
 let mut buffer=vec![0usize;(needed as usize+size_of::<usize>()-1)/size_of::<usize>()];let sd=buffer.as_mut_ptr().cast();
 if unsafe{GetKernelObjectSecurity(handle.as_raw_handle() as _,OWNER_SECURITY_INFORMATION|DACL_SECURITY_INFORMATION,sd,needed,&mut needed)}==0{return Err(io::Error::last_os_error());}
 if unsafe{IsValidSecurityDescriptor(sd)}==0{return Err(denied("invalid descriptor"));}
 let mut actual=std::ptr::null_mut();let mut defaulted=0;
 if unsafe{GetSecurityDescriptorOwner(sd,&mut actual,&mut defaulted)}==0||actual.is_null()||unsafe{IsValidSid(actual)}==0{return Err(denied("missing owner"));}
 if if private{(unsafe{EqualSid(actual,owner.as_ptr() as _)})==0}else{!trusted(actual,owner)}{return Err(denied("foreign owner"));}
 let(mut present,mut ddefault)=(0,0);let mut acl=std::ptr::null_mut();
 if unsafe{GetSecurityDescriptorDacl(sd,&mut present,&mut acl,&mut ddefault)}==0||present==0||acl.is_null()||unsafe{IsValidAcl(acl)}==0{return Err(denied("missing or invalid DACL"));}
 let aclref=unsafe{&*acl};let mut owner_inherits=false;
 for index in 0..u32::from(aclref.AceCount){
  let mut raw=std::ptr::null_mut();if unsafe{GetAce(acl,index,&mut raw)}==0{return Err(io::Error::last_os_error());}
  let h=unsafe{&*raw.cast::<ACE_HEADER>()};
  if h.AceType!=0&&h.AceType!=1{return Err(denied("unsupported complex ACE"));}
  if h.AceSize<16{return Err(denied("truncated ACE"));}
  let a=unsafe{&*raw.cast::<ACCESS_ALLOWED_ACE>()};let sid=std::ptr::addr_of!(a.SidStart) as *mut std::ffi::c_void;
  let sh=unsafe{std::slice::from_raw_parts(sid as *const u8,8)};
  if sh[0]!=1||sh[1]>15||8+8+4*usize::from(sh[1])>h.AceSize as usize{return Err(denied("truncated ACE SID"));}
  if h.AceType==1 || (!creating_child && h.AceFlags&INHERIT_ONLY_ACE as u8!=0){continue;}
  if container_child && h.AceFlags&CONTAINER_INHERIT_ACE as u8!=0 && unsafe{EqualSid(sid,owner.as_ptr() as _)}!=0 && (a.Mask&GENERIC_ALL!=0 || a.Mask&FILE_ALL_ACCESS==FILE_ALL_ACCESS){owner_inherits=true;}
  const FOREIGN_WRITE:u32=0x40000000|0x10000000|FILE_WRITE_DATA|FILE_APPEND_DATA|FILE_WRITE_EA|FILE_WRITE_ATTRIBUTES|FILE_DELETE_CHILD|DELETE|WRITE_DAC|WRITE_OWNER;
  if !trusted(sid,owner)&&(if private{a.Mask!=0}else{a.Mask&FOREIGN_WRITE!=0}){return Err(denied("foreign access violates lease privacy"));}
 }
 if container_child&&!owner_inherits{return Err(denied("private child requires inheritable owner full access, not default-token ACL fallback"));}
 Ok(())
}
fn open(path:&Path,directory:bool,create_file:bool)->io::Result<File>{
 let text:Vec<u16>=path.as_os_str().encode_wide().chain(Some(0)).collect();
 let desired=if directory{READ_CONTROL|FILE_READ_ATTRIBUTES}else{GENERIC_READ|GENERIC_WRITE|READ_CONTROL};
 let raw=unsafe{CreateFileW(text.as_ptr(),desired,FILE_SHARE_READ|FILE_SHARE_WRITE,std::ptr::null(),if create_file{OPEN_ALWAYS}else{OPEN_EXISTING},FILE_FLAG_OPEN_REPARSE_POINT|if directory{FILE_FLAG_BACKUP_SEMANTICS}else{0},0)};
 if raw==INVALID_HANDLE_VALUE{return Err(io::Error::last_os_error());}Ok(unsafe{File::from_raw_handle(raw as _)})
}
fn check_object(file:&File,directory:bool)->io::Result<()>{
 let mut info=MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();if unsafe{GetFileInformationByHandle(file.as_raw_handle() as _,info.as_mut_ptr())}==0{return Err(io::Error::last_os_error());}
 let info=unsafe{info.assume_init()};if info.dwFileAttributes&FILE_ATTRIBUTE_REPARSE_POINT!=0 || (info.dwFileAttributes&FILE_ATTRIBUTE_DIRECTORY!=0)!=directory || (!directory&&info.nNumberOfLinks!=1){return Err(denied("untrusted lease object type or links"));}Ok(())
}
pub fn acquire_in_existing_profile(profile:&Path,key:&str)->io::Result<File>{
 if !key_valid(key){return Err(io::Error::new(io::ErrorKind::InvalidInput,"invalid lease key"));}
 let owner=process_sid()?;
 let parent=open(&profile,true,false)?;check_object(&parent,true)?;check_security(&parent,&owner,false)?;
 let root=profile.join(".bscp-microdroid");let root_handle=open(&root,true,false)?;check_object(&root_handle,true)?;check_security(&root_handle,&owner,true)?;
 let leases=root.join("leases");let lease_root=open(&leases,true,false)?;check_object(&lease_root,true)?;check_security(&lease_root,&owner,true)?;
 // Only ordinary new lease files; inherit the verified existing private root.
 // No security descriptor is supplied and no existing ACL is changed.
 let file=open(&leases.join(key),false,true)?;check_object(&file,false)?;check_security(&file,&owner,true)?;
 let mut overlapped:OVERLAPPED=unsafe{std::mem::zeroed()};
 if unsafe{LockFileEx(file.as_raw_handle() as _,LOCKFILE_EXCLUSIVE_LOCK|LOCKFILE_FAIL_IMMEDIATELY,0,1,0,&mut overlapped)}==0{return Err(io::Error::last_os_error());}
 Ok(file)
}
/// Existing private parent, with every ordinary ancestor held against rename.
/// Never repairs ACLs or creates a root; it is not RPC request authority.
pub struct ExistingPrivateDirectoryGuard { _ancestors:Vec<File>, path:PathBuf, owner:Vec<u8> }
// Created-child validation failures retain original pins for this process lifetime.
// They are UNKNOWN retained evidence, never a successful resource release.
static FAILED_PRIVATE_CHILDREN:std::sync::LazyLock<std::sync::Mutex<Vec<ExistingPrivateDirectoryGuard>>>=std::sync::LazyLock::new(||std::sync::Mutex::new(Vec::new()));
impl ExistingPrivateDirectoryGuard {
 pub fn open(path:&Path)->io::Result<Self>{
  use std::path::{Component,Prefix};
  let mut parts=path.components();
  if !matches!(parts.next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_))) || parts.next()!=Some(Component::RootDir) || parts.any(|p|!matches!(p,Component::Normal(_))){return Err(denied("trace parent requires an absolute ordinary local disk path"));}
  let owner=process_sid()?;let mut paths:Vec<_>=path.ancestors().collect();paths.reverse();let mut ancestors=Vec::new();
  for p in paths {let f=open(p,true,false)?;check_object(&f,true)?;ancestors.push(f);}
  let parent=ancestors.last().ok_or_else(||denied("missing trace parent"))?;
  // Check inheritable-only grants too, before creating any child. A private
  // parent with foreign child grants must not briefly expose a new trace.
  check_security_impl(parent,&owner,true,true)?;
  Ok(Self{_ancestors:ancestors,path:path.to_owned(),owner})
 }
 /// Creates one ordinary child only beneath this already-private original parent.
 /// Inherits the verified ACL; no explicit security descriptor or ACL mutation.
 /// Consumes the guard so all parent/child original handles remain held by State.
 pub fn create_private_child(mut self,name:&str)->io::Result<Self>{
  if !key_valid(name){return Err(denied("invalid private child key"));}
  let upper=name.to_ascii_uppercase();
  if matches!(upper.as_str(),"CON"|"PRN"|"AUX"|"NUL") || ["COM","LPT"].iter().any(|p|upper.strip_prefix(p).is_some_and(|s|s.len()==1&&matches!(s.as_bytes()[0],b'1'..=b'9'))){return Err(denied("reserved private child name"));}
  let parent=self._ancestors.last().ok_or_else(||denied("missing private parent"))?;
  check_security_details(parent,&self.owner,true,true,true)?;
  let child=self.path.join(name);let wide:Vec<u16>=child.as_os_str().encode_wide().chain(Some(0)).collect();
  self._ancestors.try_reserve(1).map_err(|_|io::Error::new(io::ErrorKind::OutOfMemory,"cannot reserve child original-handle slot"))?;
  let mut retained=FAILED_PRIVATE_CHILDREN.lock().unwrap_or_else(|e|e.into_inner());
  retained.try_reserve(1).map_err(|_|io::Error::new(io::ErrorKind::OutOfMemory,"cannot reserve failed-child ownership slot"))?;
  if unsafe{CreateDirectoryW(wide.as_ptr(),std::ptr::null())}==0{return Err(io::Error::last_os_error());}
  self.path=child;
  // Keep parent pins, and any acquired child handle, even on post-create failure.
  let validated=(||->io::Result<()> {
   let original=open(&self.path,true,false)?;self._ancestors.push(original);
   let original=self._ancestors.last().unwrap();check_object(original,true)?;
   check_security_impl(original,&self.owner,true,true)?;
   let mut flags=0;if unsafe{GetHandleInformation(original.as_raw_handle() as _,&mut flags)}==0{return Err(io::Error::last_os_error());}
   if flags&HANDLE_FLAG_INHERIT!=0{return Err(denied("private child handle inherited unexpectedly"));}Ok(())
  })();
  if let Err(error)=validated {
   // Capacity was reserved before creation while holding this same lock.
   retained.push(self);
   let message=format!("created private child retained UNKNOWN at {:?}: {error}",retained.last().unwrap().path);
   return Err(io::Error::new(error.kind(),message));
  }
  Ok(self)
 }
 pub fn create_trace(&self,name:&std::ffi::OsStr)->io::Result<File>{
  // One filename only; no ADS, traversal, NUL, Win32 normalization or device names.
  let text=name.to_str().ok_or_else(||denied("trace filename must be Unicode"))?;
  if text.is_empty()||text=="."||text==".."||text.ends_with(['.',' '])||text.chars().any(|c|c<' '||"<>:\"/\\|?*".contains(c)){return Err(denied("invalid trace filename"));}
  let stem=text.split('.').next().unwrap().to_ascii_uppercase();
  if matches!(stem.as_str(),"CON"|"PRN"|"AUX"|"NUL")||["COM","LPT"].iter().any(|p|stem.strip_prefix(p).is_some_and(|s|s.len()==1&&matches!(s.as_bytes()[0],b'1'..=b'9'))){return Err(denied("reserved trace filename"));}
  let parent=self._ancestors.last().ok_or_else(||denied("missing trace parent"))?;check_security_impl(parent,&self.owner,true,true)?;
  let path=self.path.join(name);let wide:Vec<u16>=path.as_os_str().encode_wide().chain(Some(0)).collect();
  let raw=unsafe{CreateFileW(wide.as_ptr(),GENERIC_WRITE|READ_CONTROL|FILE_READ_ATTRIBUTES,FILE_SHARE_READ,std::ptr::null(),CREATE_NEW,FILE_FLAG_OPEN_REPARSE_POINT,0)};
  if raw==INVALID_HANDLE_VALUE{return Err(io::Error::last_os_error());}
  let file=unsafe{File::from_raw_handle(raw as _)};
  check_object(&file,false)?;check_security(&file,&self.owner,true)?;
  let mut flags=0;if unsafe{GetHandleInformation(file.as_raw_handle() as _,&mut flags)}==0{return Err(io::Error::last_os_error());}
  if flags&HANDLE_FLAG_INHERIT!=0{return Err(denied("trace handle inherited unexpectedly"));}
  // On validation failure retain the created file as evidence; never delete it.
  Ok(file)
 }
}

#[cfg(test)] mod shared_guard_tests {
 use super::*;
 #[test] fn invalid_keys_do_not_open_profile() {
  for key in ["", "../x", "x/y", "x.y", "CON:"] {
   assert!(acquire_in_existing_profile(Path::new("this-path-must-not-be-opened"),key).is_err());
  }
 }
 #[test] fn nonlocal_or_relative_directory_rejected_before_open() {
  for path in ["relative", "C:relative", r"\\host\share\x", r"C:\a\..\b"] {
   assert!(ExistingPrivateDirectoryGuard::open(Path::new(path)).is_err(), "{path}");
  }
 }
 #[test] fn invalid_names_rejected_before_any_parent_access() {
  let guard=ExistingPrivateDirectoryGuard{_ancestors:Vec::new(),path:PathBuf::new(),owner:Vec::new()};
  for name in ["", ".", "..", "bad.", "bad ", "a:b", "a/b", r"a\b", "NUL", "CON.log", "COM1.txt", "LPT9.log"] {
   let err=guard.create_trace(std::ffi::OsStr::new(name)).unwrap_err();
   assert_ne!(err.to_string(),"missing trace parent", "{name}");
  }
 }
}
