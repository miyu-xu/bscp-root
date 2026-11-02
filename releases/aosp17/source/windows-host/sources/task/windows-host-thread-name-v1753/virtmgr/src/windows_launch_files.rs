//! Stable, typed file capabilities for one Windows crosvm launch.
/// Pass the original service process as a separate typed capability, never as
/// a file/CRT descriptor or a PID to reopen. Only query and observation rights.
pub fn preserve_guest_server(prepared:&mut desktop_host::windows_process::PreparedHandles)->std::io::Result<String>{
    use std::os::windows::io::{AsHandle,AsRawHandle,FromRawHandle,OwnedHandle};
    use windows_sys::Win32::{Foundation::{DuplicateHandle,GetHandleInformation,FILETIME,HANDLE_FLAG_INHERIT},System::Threading::{GetCurrentProcess,GetProcessId,GetProcessTimes,WaitForSingleObject}};
    const RIGHTS:u32=0x00101000;
    let current=unsafe{GetCurrentProcess()};let mut raw=0;
    if unsafe{DuplicateHandle(current,current,current,&mut raw,RIGHTS,0,0)}==0{return Err(std::io::Error::last_os_error());}
    let owned=unsafe{OwnedHandle::from_raw_handle(raw as _)};
    let index=prepared.push_duplicate(owned.as_handle())?;
    let import=prepared.get(index).ok_or_else(||std::io::Error::other("server process import absent"))?;
    let h=import.as_raw_handle() as isize;let mut flags=0;
    if unsafe{GetHandleInformation(h,&mut flags)}==0{return Err(std::io::Error::last_os_error());}
    if flags&HANDLE_FLAG_INHERIT!=0{return Err(std::io::Error::other("server process import unexpectedly inheritable"));}
    let pid=unsafe{GetProcessId(h)};if pid==0{return Err(std::io::Error::last_os_error());}
    let mut times:[FILETIME;4]=unsafe{std::mem::zeroed()};
    if unsafe{GetProcessTimes(h,times.as_mut_ptr(),times.as_mut_ptr().add(1),times.as_mut_ptr().add(2),times.as_mut_ptr().add(3))}==0{return Err(std::io::Error::last_os_error());}
    if unsafe{WaitForSingleObject(h,0)}!=258{return Err(std::io::Error::other("original service process is not live"));}
    let generation=((times[0].dwHighDateTime as u64)<<32)|times[0].dwLowDateTime as u64;
    Ok(format!("{}:{pid}:{generation}",import.as_raw_handle() as usize))
}
pub use base::sys::windows::inherited_handles::native_snapshot;
use std::{io,os::windows::io::{AsRawHandle,BorrowedHandle,OwnedHandle,FromRawHandle}};
use windows_sys::Win32::{Foundation::DuplicateHandle,System::Threading::GetCurrentProcess};
#[derive(Clone,Copy)]
pub enum Role {Kernel,Initrd,Bios,Disk,Composite,CompositeRoot,Metadata,ConsoleIn,ConsoleOut,Log,Failure,Control,Ramdump,DtbIn,DtbOut}
impl Role {fn name(self)->&'static str {match self {Self::Kernel=>"kernel",Self::Initrd=>"initrd",Self::Bios=>"bios",Self::Disk=>"disk",Self::Composite=>"composite",Self::CompositeRoot=>"composite-root",Self::Metadata=>"metadata",Self::ConsoleIn=>"console-in",Self::ConsoleOut=>"console-out",Self::Log=>"log",Self::Failure=>"failure",Self::Control=>"control",Self::Ramdump=>"ramdump",Self::DtbIn=>"dtb-in",Self::DtbOut=>"dtb-out"}}}
/// Query the exact dedicated duplicate passed to the launch callback. This
/// function does not turn inheritance on; spawn's gate owns that short window.
pub fn record(handle:BorrowedHandle<'_>,role:Role)->io::Result<String>{
 let raw=handle.as_raw_handle();let state=native_snapshot::snapshot(raw,false)?;
 let kind=match state.kind {1=>"file",3=>"pipe",_=>return Err(io::ErrorKind::InvalidInput.into())};
 let id=state.id.iter().map(|b|format!("{b:02x}")).collect::<String>();
 Ok(serde_json::json!({"handle":(raw as usize).to_string(),"role":role.name(),"kind":kind,"access":state.access,"mode":state.mode,"volume":format!("{:x}",state.volume),"file_id":id,"pipe_flags":state.pipe_flags,"pipe_state":state.pipe_state}).to_string())
}
/// Restore the existing file-log append contract by removing WRITE_DATA from
/// a dedicated duplicate, never by reopening a pathname or requesting rights
/// absent on the original object. Pipes use their separate write-data contract.
pub fn append_only_duplicate(original:BorrowedHandle<'_>)->io::Result<OwnedHandle>{
 let before=native_snapshot::snapshot(original.as_raw_handle(),false)?;
 const WRITE_DATA:u32=2;const APPEND_DATA:u32=4;
 if before.kind!=1||before.access&APPEND_DATA==0 {return Err(io::ErrorKind::PermissionDenied.into());}
 let desired=before.access&!WRITE_DATA;let mut raw=0;
 if unsafe{DuplicateHandle(GetCurrentProcess(),original.as_raw_handle() as _,GetCurrentProcess(),&mut raw,desired,0,0)}==0 {return Err(io::Error::last_os_error());}
 let copy=unsafe{OwnedHandle::from_raw_handle(raw as _)};
 let after=native_snapshot::snapshot(copy.as_raw_handle(),false)?;
 if after.kind!=1||after.volume!=before.volume||after.id!=before.id||after.mode!=before.mode||after.access&!before.access!=0||after.access&WRITE_DATA!=0||after.access&APPEND_DATA==0 {return Err(io::Error::new(io::ErrorKind::InvalidData,"append duplicate identity/access mismatch"));}
 Ok(copy)
}

/// Every reference belongs to this exact launch. All copies remain non-inheritable
/// until the reviewed spawn gate consumes this plan; no raw value is adopted.
#[derive(Default)]
pub struct LaunchFiles {
    prepared: desktop_host::windows_process::PreparedHandles,
    records: Vec<String>,
    poisoned: bool,
}
impl LaunchFiles {
    pub fn new()->Self {Self::default()}
    pub fn preserve(&mut self, file:&std::fs::File, role:Role)->io::Result<String> {
        use std::os::windows::io::AsHandle;
        if self.poisoned {return Err(io::Error::other("failed launch plan cannot be reused"));}
        self.poisoned=true;
        let index=self.prepared.push_duplicate(file.as_handle())?;
        let handle=self.prepared.get(index).ok_or_else(||io::Error::other("prepared handle missing"))?;
        let record=record(handle,role)?;
        let path=format!("handle:{}",handle.as_raw_handle() as usize);
        self.records.push(record);self.poisoned=false;Ok(path)
    }
    /// File serial sinks preserve append-only writes; pipes retain WRITE_DATA.
    /// The original object is borrowed and never reopened or marked inheritable.
    pub fn preserve_output(&mut self, file:&std::fs::File, role:Role)->io::Result<String> {
        use std::os::windows::io::AsHandle;
        if !matches!(role,Role::ConsoleOut|Role::Log|Role::Ramdump) {return Err(io::ErrorKind::InvalidInput.into());}
        match native_snapshot::snapshot(file.as_raw_handle(),false)?.kind {
            1 => {
                let append=std::fs::File::from(append_only_duplicate(file.as_handle())?);
                self.preserve(&append,role)
            },
            3 => self.preserve(file,role),
            _ => Err(io::ErrorKind::InvalidInput.into()),
        }
    }
    pub fn into_parts(self)->io::Result<(desktop_host::windows_process::PreparedHandles,Vec<String>)> {
        if self.poisoned {return Err(io::Error::other("failed launch plan cannot be consumed"));}
        Ok((self.prepared,self.records))
    }
}

/// Preserve the exact generated composite object while reducing write access.
/// No pathname is resolved and no permission absent on the original is requested.
pub fn read_only_duplicate(file:&std::fs::File)->io::Result<std::fs::File> {
    use std::os::windows::io::AsHandle;
    use windows_sys::Win32::Foundation::{GetHandleInformation,HANDLE_FLAG_INHERIT};
    const REQUIRED:u32=0x0001|0x0080|0x00100000; // READ_DATA, READ_ATTRIBUTES, SYNCHRONIZE
    let before=native_snapshot::snapshot(file.as_raw_handle(),false)?;
    if before.kind!=1 || before.access&REQUIRED!=REQUIRED {return Err(io::ErrorKind::PermissionDenied.into());}
    let mut raw=0;
    if unsafe{DuplicateHandle(GetCurrentProcess(),file.as_handle().as_raw_handle() as _,GetCurrentProcess(),&mut raw,REQUIRED,0,0)}==0 {return Err(io::Error::last_os_error());}
    let copy=unsafe{OwnedHandle::from_raw_handle(raw as _)};
    let after=native_snapshot::snapshot(copy.as_raw_handle(),false)?;
    let mut flags=0;
    if unsafe{GetHandleInformation(copy.as_raw_handle() as _,&mut flags)}==0 {return Err(io::Error::last_os_error());}
    if after.kind!=1 || after.volume!=before.volume || after.id!=before.id || after.mode!=before.mode || after.access&!REQUIRED!=0 || after.access&REQUIRED!=REQUIRED || flags&HANDLE_FLAG_INHERIT!=0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,"read duplicate identity/access mismatch"));
    }
    Ok(std::fs::File::from(copy))
}
