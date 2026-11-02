//! Classification only, after the original image HANDLE and path identity checks.
//! Configured desktop roots retain Android logical partition metadata semantics.
use std::{ffi::OsStr,io,path::{Component,Path,PathBuf,Prefix}};
use std::os::windows::ffi::OsStrExt;
fn invalid(message:&str)->io::Error {io::Error::new(io::ErrorKind::InvalidInput,message)}
#[link(name="kernel32")]
extern "system" {fn CompareStringOrdinal(a:*const u16,alen:i32,b:*const u16,blen:i32,ignore_case:i32)->i32;}
fn equal(a:&OsStr,b:&OsStr)->io::Result<bool>{
 let a:Vec<u16>=a.encode_wide().collect();let b:Vec<u16>=b.encode_wide().collect();
 let an=i32::try_from(a.len()).map_err(|_|invalid("oversize path component"))?;
 let bn=i32::try_from(b.len()).map_err(|_|invalid("oversize path component"))?;
 let result=unsafe{CompareStringOrdinal(a.as_ptr(),an,b.as_ptr(),bn,1)};
 if result==0{return Err(io::Error::last_os_error());}Ok(result==2)
}
fn parts(path:&Path)->io::Result<(u8,Vec<&OsStr>)>{
 let mut c=path.components();
 let drive=match c.next(){Some(Component::Prefix(p))=>match p.kind(){Prefix::Disk(d)|Prefix::VerbatimDisk(d)=>d.to_ascii_uppercase(),_=>return Err(invalid("unsupported non-local path prefix"))},_=>return Err(invalid("partition path must be drive absolute"))};
 if c.next()!=Some(Component::RootDir){return Err(invalid("partition path is not rooted"));}
 let mut rest=Vec::new();
 for part in c {match part {Component::Normal(p)=>rest.push(p),_=>return Err(invalid("non-normal partition path component"))}}
 Ok((drive,rest))
}
pub(crate) fn logical_from_roots(path:&Path,roots:&[(PathBuf,&str)])->io::Result<Option<PathBuf>>{
 let (drive,components)=parts(path)?;
 let mut found=None;
 for (root,logical) in roots {
  let (rd,rc)=parts(root)?;
  if rd!=drive || rc.len()>components.len(){continue;}
  let mut matches=true;
  for (a,b) in rc.iter().zip(&components){if !equal(a,b)?{matches=false;break;}}
  if !matches{continue;}
  if found.is_some(){return Err(invalid("ambiguous overlapping desktop partition roots"));}
  let mut mapped=PathBuf::from(logical);
  for component in &components[rc.len()..]{mapped.push(component);}
  found=Some(mapped);
 }
 Ok(found)
}
pub(crate) fn logical_path(path:&Path)->io::Result<Option<PathBuf>>{
 let mut roots=Vec::new();
 for (key,logical) in [("VIRTMGR_APEX_ROOT","/apex"),("VIRTMGR_SYSTEM_ROOT","/system"),("VIRTMGR_SYSTEM_EXT_ROOT","/system_ext")] {
  if let Some(root)=std::env::var_os(key){
   let root=PathBuf::from(root);parts(&root)?;
   let canonical=std::fs::canonicalize(&root)?;
   if !canonical.is_dir(){return Err(invalid("desktop partition root is not a directory"));}
   roots.push((canonical,logical));
  }
 }
 logical_from_roots(path,&roots)
}
