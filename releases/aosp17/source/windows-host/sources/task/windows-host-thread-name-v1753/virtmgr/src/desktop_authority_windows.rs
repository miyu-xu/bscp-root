//! Windows desktop authority: listener ownership is not caller authorization.
use std::{io,mem::{size_of,MaybeUninit},sync::OnceLock};
use std::os::windows::{ffi::OsStringExt,io::{AsRawHandle,FromRawHandle,OwnedHandle}};
use binder::platform_fd::BorrowedFd;
use desktop_host::request_identity::{self,CallerIdentity,FileAccess};
use windows_sys::Win32::{Foundation::*,Security::*,System::Threading::*,Storage::FileSystem::*};
fn denied(s:&str)->io::Error{io::Error::new(io::ErrorKind::PermissionDenied,s)}
#[derive(Debug)] pub struct Authority{owner:CallerIdentity}
static OWNER:OnceLock<Authority>=OnceLock::new();
/// Only initializes the owning service's identity. Never used as a request token.
fn process_identity()->io::Result<CallerIdentity>{
 let mut raw=0;
 if unsafe{OpenProcessToken(GetCurrentProcess(),TOKEN_QUERY,&mut raw)}==0{return Err(io::Error::last_os_error());}
 let token=unsafe{OwnedHandle::from_raw_handle(raw as _)};
 let mut user=[0usize;128];let mut used=0;
 if unsafe{GetTokenInformation(token.as_raw_handle() as _,TokenUser,user.as_mut_ptr().cast(),size_of_val(&user) as u32,&mut used)}==0{return Err(io::Error::last_os_error());}
 if (used as usize)<size_of::<TOKEN_USER>() || used as usize>size_of_val(&user){return Err(denied("invalid token user size"));}
 let u=unsafe{&*user.as_ptr().cast::<TOKEN_USER>()};let base=user.as_ptr() as usize;let sid=u.User.Sid as usize;
 if sid<base || sid.checked_add(8).filter(|end|*end<=base+used as usize).is_none(){return Err(denied("invalid token SID extent"));}
 let header=unsafe{std::slice::from_raw_parts(sid as *const u8,8)};
 let length=8+4*usize::from(header[1]);
 if header[0]!=1 || header[1]>15 || sid.checked_add(length).filter(|end|*end<=base+used as usize).is_none(){return Err(denied("invalid token SID"));}
 let sid=unsafe{std::slice::from_raw_parts(sid as *const u8,length)}.to_vec();
 let mut stats=MaybeUninit::<TOKEN_STATISTICS>::uninit();used=0;
 if unsafe{GetTokenInformation(token.as_raw_handle() as _,TokenStatistics,stats.as_mut_ptr().cast(),size_of::<TOKEN_STATISTICS>() as u32,&mut used)}==0{return Err(io::Error::last_os_error());}
 if used as usize!=size_of::<TOKEN_STATISTICS>(){return Err(denied("invalid token statistics size"));}
 let stats=unsafe{stats.assume_init()};
 Ok(CallerIdentity{sid,logon_low:stats.AuthenticationId.LowPart,logon_high:stats.AuthenticationId.HighPart})
}
pub fn initialize_listener()->io::Result<()>{
 let owner=process_identity()?;
 OWNER.set(Authority{owner}).map_err(|_|denied("desktop owner already initialized"))
}
pub fn initialize(_fd:BorrowedFd<'_>,_creator:i32)->io::Result<()>{
 Err(io::Error::new(io::ErrorKind::Unsupported,"Unix inherited-FD bootstrap is not a Windows authenticated object bootstrap"))
}
pub fn current()->io::Result<&'static Authority>{OWNER.get().ok_or_else(||denied("desktop listener owner is not initialized"))}
impl Authority{
 pub fn identity(&self)->CallerIdentity{self.owner.clone()}
 pub fn request_identity(&self)->io::Result<CallerIdentity>{
  request_identity::check_owner()?;
  let caller=request_identity::current_caller()?;
  if caller!=self.owner || process_identity()?!=self.owner{return Err(denied("request or service identity differs from listener owner"));}
  Ok(caller)
 }
 pub fn check(&self,permission:&str)->io::Result<()>{
  self.request_identity()?;
  match permission{
   "android.permission.DUMP"|"android.permission.MANAGE_VIRTUAL_MACHINE"|"android.permission.USE_CUSTOM_VIRTUAL_MACHINE"=>Ok(()),
   _=>Err(denied("unsupported desktop permission"))
  }
 }
 pub fn check_image(&self,fd:BorrowedFd<'_>)->io::Result<()>{
  self.check("android.permission.MANAGE_VIRTUAL_MACHINE")?;
  let handle=fd.try_clone_to_handle()?;
  request_identity::check_file(&handle,FileAccess::ReadImage)?;Ok(())
 }
}
/// Query this actual open object; compare a metadata-only reopened path identity.
/// Data consumers retain the original descriptor. This is not an immutability claim.
pub fn image_descriptor_path(fd:BorrowedFd<'_>)->io::Result<std::path::PathBuf>{
 let handle=fd.try_clone_to_handle()?;
 let original=request_identity::check_file(&handle,FileAccess::ReadImage)?;
 let mut buf=vec![0u16;32768];
 let n=unsafe{GetFinalPathNameByHandleW(handle.as_raw_handle() as _,buf.as_mut_ptr(),buf.len() as u32,0)};
 if n==0{return Err(io::Error::last_os_error());}
 if n as usize>=buf.len(){return Err(denied("descriptor path exceeds bounded buffer"));}
 let path=std::path::PathBuf::from(std::ffi::OsString::from_wide(&buf[..n as usize]));
 if !path.is_absolute(){return Err(denied("descriptor path is not absolute"));}
 buf[n as usize]=0;
 let raw=unsafe{CreateFileW(buf.as_ptr(),FILE_READ_ATTRIBUTES,FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,std::ptr::null(),OPEN_EXISTING,FILE_FLAG_OPEN_REPARSE_POINT,0)};
 if raw==INVALID_HANDLE_VALUE{return Err(io::Error::last_os_error());}
 let reopened=unsafe{OwnedHandle::from_raw_handle(raw as _)};
 let mut info=MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
 if unsafe{GetFileInformationByHandle(reopened.as_raw_handle() as _,info.as_mut_ptr())}==0{return Err(io::Error::last_os_error());}
 let info=unsafe{info.assume_init()};let index=(u64::from(info.nFileIndexHigh)<<32)|u64::from(info.nFileIndexLow);
 if info.dwFileAttributes&(FILE_ATTRIBUTE_REPARSE_POINT|FILE_ATTRIBUTE_DIRECTORY)!=0 || info.dwVolumeSerialNumber!=original.volume_serial || index!=original.file_index{return Err(denied("descriptor pathname object changed"));}
 Ok(path)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn real_process_token_has_sid_and_logon(){let a=process_identity().unwrap();assert!(!a.sid.is_empty());assert_eq!(a,process_identity().unwrap());}
 #[test]fn process_owner_does_not_create_request_scope(){let a=Authority{owner:process_identity().unwrap()};assert!(a.request_identity().is_err());assert!(a.check("android.permission.MANAGE_VIRTUAL_MACHINE").is_err());}
}

/// Stable per-user profile from the actual primary token, not environment variables
/// or sandbox-redirected LocalAppData. This only queries a path; it creates nothing.
pub fn process_profile()->io::Result<std::path::PathBuf>{
 #[link(name="userenv")] extern "system"{fn GetUserProfileDirectoryW(token:isize,path:*mut u16,size:*mut u32)->i32;}
 let mut raw=0;
 if unsafe{OpenProcessToken(GetCurrentProcess(),TOKEN_QUERY,&mut raw)}==0{return Err(io::Error::last_os_error());}
 let token=unsafe{OwnedHandle::from_raw_handle(raw as _)};
 let mut path=vec![0u16;32768];let mut size=path.len() as u32;
 if unsafe{GetUserProfileDirectoryW(token.as_raw_handle() as _,path.as_mut_ptr(),&mut size)}==0{return Err(io::Error::last_os_error());}
 if size==0 || size as usize>path.len() || path[size as usize-1]!=0{return Err(denied("invalid process profile path length"));}
 let path=std::path::PathBuf::from(std::ffi::OsString::from_wide(&path[..size as usize-1]));
 if !path.is_absolute(){return Err(denied("process profile path is not absolute"));}Ok(path)
}
#[cfg(test)]mod profile_tests{
 use super::*;
 #[test]fn actual_profile_query_only(){let p=process_profile().unwrap();assert!(p.is_absolute());println!("actual_token_profile={}",p.display());}
}
