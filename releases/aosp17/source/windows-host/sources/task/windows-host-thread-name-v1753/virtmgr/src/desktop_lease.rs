//! Same-user desktop allocation leases. These are not persistent-disk recovery claims.
use std::{fs::{File,OpenOptions},io,path::PathBuf};
use std::os::unix::fs::{OpenOptionsExt,DirBuilderExt,MetadataExt};
use std::os::fd::AsRawFd;
pub fn acquire(key:&str)->io::Result<File> {
    if key.is_empty() || !key.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-') {return Err(io::Error::new(io::ErrorKind::InvalidInput,"invalid lease key"));}
    let uid=unsafe{libc::getuid()};
    // Fixed per-user namespace, not configurable by a per-instance environment variable.
    let root=PathBuf::from("/tmp").join(format!("bscp-virtmgr-leases-{uid}"));
    match std::fs::DirBuilder::new().mode(0o700).create(&root) {Ok(())=>(),Err(e) if e.kind()==io::ErrorKind::AlreadyExists=>(),Err(e)=>return Err(e)}
    let m=std::fs::symlink_metadata(&root)?;
    if !m.is_dir()||m.uid()!=uid||m.mode()&0o077!=0 {return Err(io::Error::new(io::ErrorKind::PermissionDenied,"untrusted lease directory"));}
    let f=OpenOptions::new().read(true).write(true).create(true).mode(0o600).custom_flags(libc::O_NOFOLLOW|libc::O_CLOEXEC).open(root.join(key))?;
    let m=f.metadata()?;
    if !m.is_file()||m.uid()!=uid||m.mode()&0o077!=0||m.nlink()!=1 {return Err(io::Error::new(io::ErrorKind::PermissionDenied,"untrusted lease file"));}
    if unsafe{libc::flock(f.as_raw_fd(),libc::LOCK_EX|libc::LOCK_NB)}!=0 {return Err(io::Error::last_os_error());}
    Ok(f)
}
#[cfg(test)]mod tests {
 use super::*;
 #[test]fn actual_exclusive_lease_and_release(){
  let key=format!("test-{}",std::process::id());let first=acquire(&key).unwrap();assert!(acquire(&key).is_err());drop(first);let second=acquire(&key).unwrap();drop(second);
 }
 #[test]fn path_injection_rejected(){assert!(acquire("../unknown").is_err());assert!(acquire("").is_err());}
 #[test]fn lease_child(){
  let Ok(key)=std::env::var("TEST_LEASE_KEY")else{return;};assert!(acquire(&key).is_err());println!("lease conflict observed");
 }
 #[test]fn real_other_process_cannot_claim(){
  let key=format!("child-test-{}",std::process::id());let _owner=acquire(&key).unwrap();
  let name=format!("{}::lease_child",module_path!().split_once("::").unwrap().1);
  let q=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact",&name,"--nocapture"]).env("TEST_LEASE_KEY",&key).output().unwrap();
  assert!(q.status.success());assert!(String::from_utf8_lossy(&q.stdout).contains("lease conflict observed"));
 }
}
