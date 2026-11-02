//! Windows image-header reads without changing a shared original HANDLE's file pointer.
use std::{fs::File,io::{self,ErrorKind},os::windows::{fs::FileExt,io::{AsRawHandle,FromRawHandle,OwnedHandle}}};
use windows_sys::Win32::{Foundation::{INVALID_HANDLE_VALUE,SetHandleInformation,GetHandleInformation,HANDLE_FLAG_INHERIT},Storage::FileSystem::{ReOpenFile,GetFileType,FILE_TYPE_DISK,FILE_READ_DATA,FILE_READ_ATTRIBUTES,GetFileInformationByHandle,BY_HANDLE_FILE_INFORMATION,FILE_SHARE_READ,FILE_SHARE_WRITE,FILE_SHARE_DELETE}};
// PUBLIC_OBJECT_BASIC_INFORMATION from documented NtQueryObject(ObjectBasicInformation).
#[repr(C)] #[derive(Default)]
struct ObjectBasic { attributes:u32, granted:u32, handles:u32, pointers:u32, reserved:[u32;10] }
#[link(name="ntdll")] extern "system" {
 fn NtQueryObject(handle:isize,class:u32,info:*mut std::ffi::c_void,len:u32,returned:*mut u32)->i32;
}
const READ_MASK:u32=FILE_READ_DATA|FILE_READ_ATTRIBUTES|0x0010_0000; // SYNCHRONIZE for synchronous FileExt reads.
fn granted(file:&File)->io::Result<u32>{
 let mut info=ObjectBasic::default();let mut returned=0;
 let status=unsafe{NtQueryObject(file.as_raw_handle() as _,0,(&mut info as *mut ObjectBasic).cast(),std::mem::size_of::<ObjectBasic>() as u32,&mut returned)};
 if status<0 || returned<std::mem::size_of::<ObjectBasic>() as u32{return Err(io::Error::other(format!("NtQueryObject rejected original access: {status:#x}")));}
 Ok(info.granted)
}
fn identity(file:&File)->io::Result<(u32,u32,u32)>{
 let mut info=std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
 if unsafe{GetFileInformationByHandle(file.as_raw_handle() as _,info.as_mut_ptr())}==0{return Err(io::Error::last_os_error());}
 let info=unsafe{info.assume_init()};
 if info.nFileIndexHigh==0 && info.nFileIndexLow==0{return Err(io::Error::other("file identity unavailable"));}
 Ok((info.dwVolumeSerialNumber,info.nFileIndexHigh,info.nFileIndexLow))
}
pub trait FileReadAt { fn read_exact_at(&self,buf:&mut [u8],offset:u64)->io::Result<()>; }
fn exact(mut buf:&mut [u8],mut offset:u64,mut read:impl FnMut(&mut [u8],u64)->io::Result<usize>)->io::Result<()> {
 while !buf.is_empty() {
  match read(buf,offset) {
   Ok(0)=>return Err(io::Error::from(ErrorKind::UnexpectedEof)),
   Ok(n) if n<=buf.len()=>{offset=offset.checked_add(n as u64).ok_or_else(||io::Error::from(ErrorKind::InvalidInput))?;buf=&mut buf[n..];},
   Ok(_)=>return Err(io::Error::from(ErrorKind::InvalidData)),
   Err(e) if e.kind()==ErrorKind::Interrupted=>continue,
   Err(e)=>return Err(e),
  }
 }
 Ok(())
}
impl FileReadAt for File {
 fn read_exact_at(&self,buf:&mut [u8],offset:u64)->io::Result<()> {
  if buf.is_empty(){return Ok(());}
  if unsafe{GetFileType(self.as_raw_handle() as _)}!=FILE_TYPE_DISK || !self.metadata()?.is_file(){return Err(io::Error::new(ErrorKind::InvalidInput,"image must be a regular disk file"));}
  let original_granted=granted(self)?;
  if original_granted&READ_MASK!=READ_MASK{return Err(io::Error::new(ErrorKind::PermissionDenied,"original image HANDLE lacks required read/attribute/synchronize rights"));}
  let original_id=identity(self)?;
  // ReOpenFile refers to the already-held object, not a mutable path. Unlike
  // DuplicateHandle, this provides a separate file pointer. No fallback to seek
  // on the caller's shared file is allowed. The OS checks requested read rights.
  let raw=unsafe{ReOpenFile(self.as_raw_handle() as _,READ_MASK,FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,0)};
  if raw==INVALID_HANDLE_VALUE{return Err(io::Error::last_os_error());}
  let owned=unsafe{OwnedHandle::from_raw_handle(raw as _)};
  if unsafe{SetHandleInformation(owned.as_raw_handle() as _,HANDLE_FLAG_INHERIT,0)}==0{return Err(io::Error::last_os_error());}
  let independent=File::from(owned);
  let mut flags=0;
  if unsafe{GetHandleInformation(independent.as_raw_handle() as _,&mut flags)}==0{return Err(io::Error::last_os_error());}
  if flags&HANDLE_FLAG_INHERIT!=0 || identity(&independent)?!=original_id || granted(&independent)? & !original_granted !=0 {
   return Err(io::Error::new(ErrorKind::PermissionDenied,"reopened image identity, inheritance, or rights changed"));
  }
  exact(buf,offset,|buf,pos|independent.seek_read(buf,pos))
 }
}
#[cfg(test)] mod tests {
 use super::*;
 use std::io::{Seek,SeekFrom};
 #[test] fn real_header_read_preserves_original_cursor(){let mut f=File::open(std::env::current_exe().unwrap()).unwrap();f.seek(SeekFrom::Start(37)).unwrap();let mut b=[0;2];f.read_exact_at(&mut b,0).unwrap();assert_eq!(&b,b"MZ");assert_eq!(f.stream_position().unwrap(),37);let len=f.metadata().unwrap().len();assert_eq!(f.read_exact_at(&mut b,len).unwrap_err().kind(),ErrorKind::UnexpectedEof);assert_eq!(f.stream_position().unwrap(),37);}
 #[test] fn real_concurrent_shared_handle_read(){let mut f=File::open(std::env::current_exe().unwrap()).unwrap();f.seek(SeekFrom::Start(29)).unwrap();let shared=std::sync::Arc::new(f);let hs:Vec<_>=(0..8).map(|_|{let f=shared.clone();std::thread::spawn(move||{let mut b=[0;2];f.read_exact_at(&mut b,0).unwrap();assert_eq!(&b,b"MZ");})}).collect();for h in hs{h.join().unwrap();}let mut view=&*shared;assert_eq!(view.stream_position().unwrap(),29);}
 #[test] fn partial_interrupted_and_offset_contract(){let mut b=[0;3];let mut calls=0;exact(&mut b,4,|buf,pos|{calls+=1;if calls==1{return Err(io::Error::from(ErrorKind::Interrupted));}assert_eq!(pos,4+(calls-2) as u64);buf[0]=calls;Ok(1)}).unwrap();assert_eq!(b,[2,3,4]);}
 #[test] fn zero_and_overflow_reject(){let mut b=[0;1];assert_eq!(exact(&mut b,0,|_,_|Ok(0)).unwrap_err().kind(),ErrorKind::UnexpectedEof);assert_eq!(exact(&mut b,u64::MAX,|_,_|Ok(1)).unwrap_err().kind(),ErrorKind::InvalidInput);}
 #[test] fn original_rights_gate_and_minimum_read() {
  use std::os::windows::fs::OpenOptionsExt;
  use std::io::Write;
  let path=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("rights-fixture-{}.bin",std::process::id()));
  let mut created=std::fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();created.write_all(b"DATA").unwrap();drop(created);
  let metadata=std::fs::OpenOptions::new().access_mode(FILE_READ_ATTRIBUTES).open(&path).unwrap();
  let mut bytes=[0;4];assert_eq!(metadata.read_exact_at(&mut bytes,0).unwrap_err().kind(),ErrorKind::PermissionDenied);
  let write=std::fs::OpenOptions::new().write(true).open(&path).unwrap();assert_eq!(write.read_exact_at(&mut bytes,0).unwrap_err().kind(),ErrorKind::PermissionDenied);
  let minimum=std::fs::OpenOptions::new().access_mode(READ_MASK).open(&path).unwrap();
  assert_eq!(granted(&minimum).unwrap()&READ_MASK,READ_MASK);
  minimum.read_exact_at(&mut bytes,0).unwrap();assert_eq!(&bytes,b"DATA");
 }

}
