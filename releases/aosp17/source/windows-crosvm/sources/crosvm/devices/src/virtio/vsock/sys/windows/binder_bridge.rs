// Copyright 2026 The ChromiumOS Authors
// BSD-style license; see LICENSE.
//! Guest stream / Windows Binder pipe framing and original service identity.
use std::io;

pub const MAX_FRAME: usize = 64 * 1024 * 1024;

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub fn pipe_path(cid: u64, port: u32) -> io::Result<String> {
    if !(3..u32::MAX as u64).contains(&cid) || port == 0 || port == u32::MAX {
        return Err(invalid("unsupported Binder vsock address"));
    }
    Ok(format!(r"\\.\pipe\binder_rpc_vsock_{cid}_{port}"))
}

pub fn encode(data: &[u8]) -> io::Result<Vec<u8>> {
    if data.is_empty() || data.len() > MAX_FRAME { return Err(invalid("empty or oversize Binder frame")); }
    let mut frame = Vec::with_capacity(8 + data.len());
    frame.extend_from_slice(&(data.len() as u32).to_le_bytes());
    frame.extend_from_slice(&0u32.to_le_bytes());
    frame.extend_from_slice(data);
    Ok(frame)
}

/// Bounded incremental decoder: never allocates the advertised frame length.
/// Payload is a byte stream; headers/handles never enter the guest stream.
#[derive(Default)]
pub struct Decoder { header: [u8; 8], used: usize, remaining: usize, failed: bool }

impl Decoder {
    pub fn consume(&mut self, mut input: &[u8]) -> io::Result<Vec<u8>> {
        if self.failed { return Err(invalid("failed Binder frame decoder")); }
        let mut output = Vec::with_capacity(input.len());
        while !input.is_empty() {
            if self.remaining != 0 {
                let n = self.remaining.min(input.len());
                output.extend_from_slice(&input[..n]);
                self.remaining -= n; input = &input[n..];
                continue;
            }
            let n = (8-self.used).min(input.len());
            self.header[self.used..self.used+n].copy_from_slice(&input[..n]);
            self.used += n; input = &input[n..];
            if self.used != 8 { continue; }
            let length = u32::from_le_bytes(self.header[..4].try_into().unwrap()) as usize;
            let handles = u32::from_le_bytes(self.header[4..].try_into().unwrap());
            if length == 0 || length > MAX_FRAME || handles != 0 {
                self.failed = true;
                return Err(invalid("oversize frame or unsupported attached handles"));
            }
            self.remaining = length; self.used = 0;
        }
        Ok(output)
    }
    pub fn finish(&self) -> io::Result<()> {
        if self.failed || self.used != 0 || self.remaining != 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated Binder pipe frame"));
        }
        Ok(())
    }
}

#[cfg(windows)]
pub mod process {
    use super::invalid;
    use std::ffi::c_void;
    use std::io;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle};
    #[repr(C)] #[derive(Default)]
    struct FileTime { low: u32, high: u32 }
    #[link(name="kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> RawHandle;
        fn DuplicateHandle(source: RawHandle, handle: RawHandle, target: RawHandle,
            output: *mut RawHandle, access: u32, inherit: i32, options: u32) -> i32;
        fn CloseHandle(handle: RawHandle) -> i32;
        fn GetProcessId(handle: RawHandle) -> u32;
        fn GetProcessTimes(handle: RawHandle, creation: *mut FileTime, exit: *mut FileTime,
            kernel: *mut FileTime, user: *mut FileTime) -> i32;
        fn GetHandleInformation(handle: RawHandle, flags: *mut u32) -> i32;
        fn WaitForSingleObject(handle: RawHandle, milliseconds: u32) -> u32;
        fn GetNamedPipeClientProcessId(pipe: RawHandle, pid: *mut u32) -> i32;
        fn GetNamedPipeServerProcessId(pipe: RawHandle, pid: *mut u32) -> i32;
        #[cfg(test)]
        fn CreateEventW(attributes: *const c_void, manual: i32, initial: i32, name: *const u16) -> RawHandle;
    }
    #[derive(Debug, PartialEq)]
    pub struct Specification { pub raw: usize, pub pid: u32, pub creation: u64 }
    impl Specification {
        pub fn parse(value: &str) -> io::Result<Self> {
            let parts: Vec<_> = value.split(':').collect();
            if parts.len()!=3 || parts.iter().any(|s|s.is_empty() || !s.bytes().all(|b|b.is_ascii_digit())) {
                return Err(invalid("process handle requires decimal HANDLE:PID:creation"));
            }
            let raw=parts[0].parse().map_err(|_|invalid("invalid process HANDLE"))?;
            let pid=parts[1].parse().map_err(|_|invalid("invalid process PID"))?;
            let creation=parts[2].parse().map_err(|_|invalid("invalid process creation"))?;
            if raw==0 || raw>isize::MAX as usize || pid==0 || creation==0 {
                return Err(invalid("null, pseudo or invalid process identity"));
            }
            Ok(Self{raw,pid,creation})
        }
    }
    #[derive(Debug)]
    pub struct ServerProcess { handle: OwnedHandle, pid: u32, creation: u64 }
    impl ServerProcess {
        /// Consume only an inherited process HANDLE proven to have this identity.
        /// Invalid/unverified input is borrowed and never closed by this method.
        pub fn from_inherited(value: &str) -> io::Result<Self> {
            let s=Specification::parse(value)?;
            let raw=s.raw as *mut c_void;
            let mut flags=0;
            if unsafe{GetHandleInformation(raw,&mut flags)}==0 {return Err(io::Error::last_os_error());}
            if flags&1==0 {return Err(invalid("process HANDLE not explicitly inherited"));}
            let mut copy=std::ptr::null_mut();
            if unsafe{DuplicateHandle(GetCurrentProcess(),raw,GetCurrentProcess(),&mut copy,
                0x00101000,0,0)}==0 {return Err(io::Error::last_os_error());}
            // Only the new successful duplicate is owned until identity checks finish.
            let result=Self{handle:unsafe{OwnedHandle::from_raw_handle(copy)},pid:s.pid,creation:s.creation};
            result.check_live()?;
            let mut copy_flags=0;
            if unsafe{GetHandleInformation(copy,&mut copy_flags)}==0 {return Err(io::Error::last_os_error());}
            if copy_flags&1!=0 {return Err(invalid("retained process HANDLE inherits"));}
            if unsafe{CloseHandle(raw)}==0 {return Err(io::Error::last_os_error());}
            Ok(result)
        }
        pub fn check_live(&self) -> io::Result<()> {
            let h=self.handle.as_raw_handle();
            let pid=unsafe{GetProcessId(h)};
            if pid==0 {return Err(io::Error::last_os_error());}
            let (mut c,mut e,mut k,mut u)=(FileTime::default(),FileTime::default(),FileTime::default(),FileTime::default());
            if unsafe{GetProcessTimes(h,&mut c,&mut e,&mut k,&mut u)}==0 {return Err(io::Error::last_os_error());}
            if pid!=self.pid || ((c.high as u64)<<32|c.low as u64)!=self.creation {
                return Err(invalid("original service generation mismatch"));
            }
            match unsafe{WaitForSingleObject(h,0)} {
                258=>Ok(()), 0=>Err(invalid("original service exited")),
                _=>Err(io::Error::last_os_error()),
            }
        }
        pub fn verify_pipe(&self, pipe: RawHandle) -> io::Result<()> {
            self.check_live()?;
            let mut pid=0;
            if unsafe{GetNamedPipeServerProcessId(pipe,&mut pid)}==0 {return Err(io::Error::last_os_error());}
            if pid!=self.pid {return Err(invalid("pipe server differs from original service"));}
            self.check_live()
        }
        pub fn verify_client(&self, pipe: RawHandle) -> io::Result<()> {
            self.check_live()?;
            let mut pid=0;
            if unsafe{GetNamedPipeClientProcessId(pipe,&mut pid)}==0 {return Err(io::Error::last_os_error());}
            if pid!=self.pid {return Err(invalid("pipe client differs from original service"));}
            self.check_live()
        }
        pub fn raw_handle(&self) -> RawHandle { self.handle.as_raw_handle() }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn inherited_self() -> (RawHandle,u32,u64) {
            let mut h=std::ptr::null_mut();
            assert_ne!(unsafe{DuplicateHandle(GetCurrentProcess(),GetCurrentProcess(),GetCurrentProcess(),&mut h,0x101000,1,0)},0);
            let(mut c,mut e,mut k,mut u)=(FileTime::default(),FileTime::default(),FileTime::default(),FileTime::default());
            assert_ne!(unsafe{GetProcessTimes(h,&mut c,&mut e,&mut k,&mut u)},0);
            (h,unsafe{GetProcessId(h)},(c.high as u64)<<32|c.low as u64)
        }
        #[test] fn real_original_process_duplicate_is_noninherited() {
            let(h,pid,time)=inherited_self();
            let owner=ServerProcess::from_inherited(&format!("{}:{pid}:{time}",h as usize)).unwrap();
            owner.check_live().unwrap();let mut flags=0;
            assert_eq!(unsafe{GetHandleInformation(h,&mut flags)},0);
            assert_ne!(unsafe{GetHandleInformation(owner.raw_handle(),&mut flags)},0);
            assert_eq!(flags&1,0);
        }
        #[test] fn wrong_generation_does_not_close_unverified_original() {
            let(h,pid,time)=inherited_self();
            assert!(ServerProcess::from_inherited(&format!("{}:{pid}:{}",h as usize,time+1)).is_err());
            let mut flags=0;assert_ne!(unsafe{GetHandleInformation(h,&mut flags)},0);
            assert_ne!(unsafe{CloseHandle(h)},0);
        }
        #[test] fn wrong_object_type_does_not_close_inherited_event() {
            let event=unsafe{CreateEventW(std::ptr::null(),1,0,std::ptr::null())};assert!(!event.is_null());
            let mut h=std::ptr::null_mut();
            assert_ne!(unsafe{DuplicateHandle(GetCurrentProcess(),event,GetCurrentProcess(),&mut h,0,1,2)},0);
            assert!(ServerProcess::from_inherited(&format!("{}:1:1",h as usize)).is_err());
            let mut flags=0;assert_ne!(unsafe{GetHandleInformation(h,&mut flags)},0);
            assert_ne!(unsafe{CloseHandle(h)},0);assert_ne!(unsafe{CloseHandle(event)},0);
        }
        #[test] fn noninherited_process_is_rejected_without_close() {
            let mut h=std::ptr::null_mut();
            assert_ne!(unsafe{DuplicateHandle(GetCurrentProcess(),GetCurrentProcess(),GetCurrentProcess(),&mut h,0x101000,0,0)},0);
            assert!(ServerProcess::from_inherited(&format!("{}:1:1",h as usize)).is_err());
            let mut flags=0;assert_ne!(unsafe{GetHandleInformation(h,&mut flags)},0);assert_ne!(unsafe{CloseHandle(h)},0);
        }
        #[test] fn exited_process_is_rejected_without_close() {
            let mut child=std::process::Command::new("C:/Windows/System32/cmd.exe").args(["/d","/c","exit","0"]).spawn().unwrap();
            let mut h=std::ptr::null_mut();
            assert_ne!(unsafe{DuplicateHandle(GetCurrentProcess(),child.as_raw_handle(),GetCurrentProcess(),&mut h,0x101000,1,0)},0);
            child.wait().unwrap();
            let(mut c,mut e,mut k,mut u)=(FileTime::default(),FileTime::default(),FileTime::default(),FileTime::default());
            assert_ne!(unsafe{GetProcessTimes(h,&mut c,&mut e,&mut k,&mut u)},0);
            let time=(c.high as u64)<<32|c.low as u64;let pid=unsafe{GetProcessId(h)};
            assert!(ServerProcess::from_inherited(&format!("{}:{pid}:{time}",h as usize)).is_err());
            let mut flags=0;assert_ne!(unsafe{GetHandleInformation(h,&mut flags)},0);assert_ne!(unsafe{CloseHandle(h)},0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn fragmented_and_concatenated_frames() {
        let bytes=[encode(b"abc").unwrap(),encode(b"defgh").unwrap()].concat();
        for step in 1..=bytes.len() {let mut d=Decoder::default();let mut out=Vec::new();for s in bytes.chunks(step){out.extend(d.consume(s).unwrap());}d.finish().unwrap();assert_eq!(out,b"abcdefgh");}
    }
    #[test] fn handles_and_oversize_poison_decoder() {
        for h in [[0u32,0u32],[1u32,1u32],[(MAX_FRAME+1) as u32,0]] {let mut d=Decoder::default();let bytes=[h[0].to_le_bytes(),h[1].to_le_bytes()].concat();assert!(d.consume(&bytes).is_err());assert!(d.consume(&encode(b"x").unwrap()).is_err());}
        assert!(encode(b"").is_err());
    }
    #[test] fn truncated_frame_is_not_eof_success() {
        for n in 1..11 {let mut d=Decoder::default();d.consume(&encode(b"abc").unwrap()[..n]).unwrap();assert!(d.finish().is_err());}
    }
    #[test] fn exact_address_no_cid_truncation() {
        assert_eq!(pipe_path(2048,2048).unwrap(),r"\\.\pipe\binder_rpc_vsock_2048_2048");assert!(pipe_path(1<<32,2048).is_err());
    }
    #[cfg(windows)] #[test] fn invalid_cli_rejected_without_handle_operations() {
        for s in ["","1:2","0:2:3","-1:2:3","1:0:3","1:2:0","1:2:3:4"," 1:2:3","18446744073709551615:2:3"] {assert!(process::Specification::parse(s).is_err());}
    }
}
