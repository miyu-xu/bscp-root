//! Owned private listener; never removes a stale or unrelated socket.
use std::{io,path::Path,os::{fd::OwnedFd,unix::{net::UnixListener,fs::{MetadataExt,PermissionsExt}}}};
pub fn bind(path:&Path)->io::Result<OwnedFd>{
    let denied=||io::Error::new(io::ErrorKind::PermissionDenied,"listener must be in an owned private directory");
    if !path.is_absolute(){return Err(denied());}
    let parent=path.parent().ok_or_else(denied)?;let m=std::fs::symlink_metadata(parent)?;
    if !m.is_dir() || m.uid()!=unsafe{libc::getuid()} || m.mode()&0o022!=0 || unsafe{libc::getuid()}==0{return Err(denied());}
    match std::fs::symlink_metadata(path){Err(e) if e.kind()==io::ErrorKind::NotFound=>{},Err(e)=>return Err(e),Ok(_)=>return Err(io::Error::new(io::ErrorKind::AlreadyExists,"listener already exists"))}
    let listener=UnixListener::bind(path)?;
    std::fs::set_permissions(path,std::fs::Permissions::from_mode(0o600))?;
    Ok(listener.into())
}
