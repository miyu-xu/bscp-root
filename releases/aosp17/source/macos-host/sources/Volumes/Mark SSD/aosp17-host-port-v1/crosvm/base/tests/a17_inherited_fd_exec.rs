#![cfg(all(target_os="macos",feature="hvf"))]
use std::fs::{File,OpenOptions};
use std::os::unix::io::{AsRawFd,FromRawFd};
use std::os::unix::fs::MetadataExt;
use std::io::{Read,Write};
#[test]
fn inherited_child(){
    let Ok(value)=std::env::var("A17_OWNED_FD_TEST") else{return;};
    let raw:i32=value.parse().unwrap();
    let path=format!("/dev/fd/{}",raw);
    let mut f=base::open_file_or_duplicate(path,OpenOptions::new().read(true).write(true)).unwrap();
    assert_ne!(f.as_raw_fd(),raw);
    assert_eq!(f.metadata().unwrap().ino().to_string(),std::env::var("A17_OWNED_INODE").unwrap());
    assert_eq!(f.metadata().unwrap().dev().to_string(),std::env::var("A17_OWNED_DEVICE").unwrap());
    assert_ne!(unsafe{libc::fcntl(f.as_raw_fd(),libc::F_GETFD)}&libc::FD_CLOEXEC,0);
    let mut text=String::new();f.read_to_string(&mut text).unwrap();assert_eq!(text,"exec known bytes");
    assert!(f.write_all(b"forbidden").is_err());drop(f);
    assert!(unsafe{libc::fcntl(raw,libc::F_GETFD)}>=0);
    println!("OWNED_EXEC_HELPER_VERIFIED");
}
#[test]
fn real_exec_preserves_inherited_identity_and_readonly_rights(){
    let mut temp=tempfile::NamedTempFile::new().unwrap();temp.write_all(b"exec known bytes").unwrap();
    let ro=File::open(temp.path()).unwrap();let raw=unsafe{libc::fcntl(ro.as_raw_fd(),libc::F_DUPFD,0)};assert!(raw>=0);
    let owned=unsafe{File::from_raw_fd(raw)};
    let output=std::process::Command::new(std::env::current_exe().unwrap())
       .args(["--exact","inherited_child","--nocapture"])
       .env("A17_OWNED_FD_TEST",raw.to_string())
       .env("A17_OWNED_INODE",ro.metadata().unwrap().ino().to_string())
       .env("A17_OWNED_DEVICE",ro.metadata().unwrap().dev().to_string()).output().unwrap();
    assert!(output.status.success(),"{:?}",output);
    assert!(String::from_utf8(output.stdout).unwrap().contains("OWNED_EXEC_HELPER_VERIFIED"));
    assert!(owned.metadata().is_ok());
    assert_eq!(std::fs::read(temp.path()).unwrap(),b"exec known bytes");
}
