//! Synchronous anonymous pipes for owned log/failure readers, not VSOCK.
#![cfg(windows)]
use std::{fs::File,io,os::windows::io::{FromRawHandle,AsRawHandle}};
use windows_sys::Win32::{Foundation::{GetHandleInformation,HANDLE_FLAG_INHERIT},System::Pipes::CreatePipe};
pub fn create() -> io::Result<(File,File)> {
    let (mut read, mut write)=(0,0);
    // Null SECURITY_ATTRIBUTES makes both endpoints non-inheritable. Actual
    // child inheritance is performed only on spawn's dedicated duplicates.
    if unsafe{CreatePipe(&mut read,&mut write,std::ptr::null(),0)}==0 {return Err(io::Error::last_os_error());}
    let reader=unsafe{File::from_raw_handle(read as _)};
    let writer=unsafe{File::from_raw_handle(write as _)};
    for file in [&reader,&writer] {
        let mut flags=0;
        if unsafe{GetHandleInformation(file.as_raw_handle() as _,&mut flags)}==0 {return Err(io::Error::last_os_error());}
        if flags&HANDLE_FLAG_INHERIT!=0 {return Err(io::Error::new(io::ErrorKind::InvalidData,"anonymous pipe unexpectedly inheritable"));}
    }
    Ok((reader,writer))
}
#[cfg(test)]mod tests{
    use super::*;use std::{io::{Read,Write},time::Duration};
    #[test]fn real_pipe_bytes_and_eof_after_all_owned_writers_close(){
        let(mut reader,mut writer)=create().unwrap();let duplicate=writer.try_clone().unwrap();
        let(tx,rx)=std::sync::mpsc::channel();let thread=std::thread::spawn(move||{let mut bytes=Vec::new();let result=reader.read_to_end(&mut bytes).map(|_|bytes);tx.send(result).unwrap();});
        writer.write_all(b"real-pipe").unwrap();drop(writer);
        assert!(matches!(rx.recv_timeout(Duration::from_millis(50)),Err(std::sync::mpsc::RecvTimeoutError::Timeout)));
        drop(duplicate);assert_eq!(rx.recv_timeout(Duration::from_secs(2)).unwrap().unwrap(),b"real-pipe");thread.join().unwrap();
    }
    #[test]fn both_original_endpoints_are_noninheritable(){
        let(r,w)=create().unwrap();for f in [&r,&w]{let mut flags=0;assert_ne!(unsafe{GetHandleInformation(f.as_raw_handle() as _,&mut flags)},0);assert_eq!(flags&HANDLE_FLAG_INHERIT,0);}
    }
    #[test]fn closed_reader_causes_real_write_error(){let(r,mut w)=create().unwrap();drop(r);assert!(w.write(b"x").is_err());}
}
