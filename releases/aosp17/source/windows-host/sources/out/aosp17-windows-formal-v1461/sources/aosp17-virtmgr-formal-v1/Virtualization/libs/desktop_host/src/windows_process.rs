//! Explicit inherited-handle launch primitive. Not wired into production yet.
//! Every inheriting process launch in the eventual host must use the same gate.
//! This module cannot constrain an unrelated library that calls CreateProcess
//! with unrestricted inheritance. That audit remains an integration gate.
#![cfg(windows)]
use std::{ffi::{OsStr,OsString}, fs::{File,OpenOptions}, io, mem::size_of,
    os::windows::{ffi::OsStrExt,fs::OpenOptionsExt,io::{AsRawHandle,BorrowedHandle,FromRawHandle,OwnedHandle,RawHandle}},
    path::{Path,Component,Prefix}, ptr, sync::Mutex, time::Duration};
use windows_sys::Win32::{Foundation::{DuplicateHandle,GetHandleInformation,SetHandleInformation,DUPLICATE_SAME_ACCESS,HANDLE,HANDLE_FLAG_INHERIT,WAIT_OBJECT_0,WAIT_TIMEOUT,FILETIME},
    Globalization::{CompareStringOrdinal,CSTR_EQUAL,CSTR_LESS_THAN,CSTR_GREATER_THAN},
    Storage::FileSystem::{GetFileInformationByHandle,BY_HANDLE_FILE_INFORMATION,FILE_SHARE_READ,FILE_SHARE_WRITE,FILE_READ_ATTRIBUTES,FILE_FLAG_OPEN_REPARSE_POINT,FILE_FLAG_BACKUP_SEMANTICS,FILE_ATTRIBUTE_REPARSE_POINT},
    System::Threading::{CreateProcessW,GetCurrentProcess,GetExitCodeProcess,GetProcessTimes,
    InitializeProcThreadAttributeList,UpdateProcThreadAttribute,DeleteProcThreadAttributeList,
    WaitForSingleObject,EXTENDED_STARTUPINFO_PRESENT,CREATE_UNICODE_ENVIRONMENT,
    PROC_THREAD_ATTRIBUTE_HANDLE_LIST,PROCESS_INFORMATION,STARTUPINFOEXW,STARTF_USESTDHANDLES}};

static SPAWN_GATE: Mutex<()> = Mutex::new(());
fn raw(handle:&OwnedHandle)->HANDLE {handle.as_raw_handle() as HANDLE}
fn wide(s:&OsStr)->io::Result<Vec<u16>> {
    let mut v:Vec<_>=s.encode_wide().collect();
    if v.contains(&0) {return Err(io::ErrorKind::InvalidInput.into());}
    v.push(0); Ok(v)
}
// Microsoft's command-line quoting algorithm for the CRT argv convention.
pub fn quote(arg:&OsStr)->io::Result<Vec<u16>> {
    let units:Vec<_>=arg.encode_wide().collect();
    if units.contains(&0) {return Err(io::ErrorKind::InvalidInput.into());}
    let mut out=vec![b'"' as u16];let mut slashes=0;
    for c in units {
        if c==b'\\' as u16 {slashes+=1;continue;}
        if c==b'"' as u16 {out.extend(std::iter::repeat(b'\\' as u16).take(slashes*2+1));}
        else {out.extend(std::iter::repeat(b'\\' as u16).take(slashes));}
        slashes=0;out.push(c);
    }
    out.extend(std::iter::repeat(b'\\' as u16).take(slashes*2));out.push(b'"' as u16);Ok(out)
}
fn duplicate(handle:BorrowedHandle<'_>)->io::Result<OwnedHandle> {
    let mut copy=0;
    if unsafe{DuplicateHandle(GetCurrentProcess(),handle.as_raw_handle() as HANDLE,GetCurrentProcess(),&mut copy,0,0,DUPLICATE_SAME_ACCESS)}==0 {return Err(io::Error::last_os_error());}
    Ok(unsafe{OwnedHandle::from_raw_handle(copy as RawHandle)})
}
fn file_id(file:&File)->io::Result<(u32,u32,u32)> {
    let mut info:BY_HANDLE_FILE_INFORMATION=unsafe{std::mem::zeroed()};
    if unsafe{GetFileInformationByHandle(file.as_raw_handle() as HANDLE,&mut info)}==0 {return Err(io::Error::last_os_error());}
    Ok((info.dwVolumeSerialNumber,info.nFileIndexHigh,info.nFileIndexLow))
}
fn ordinal(a:&OsStr,b:&OsStr)->io::Result<std::cmp::Ordering>{
    let a:Vec<_>=a.encode_wide().collect();let b:Vec<_>=b.encode_wide().collect();
    let al=i32::try_from(a.len()).map_err(|_|io::ErrorKind::InvalidInput)?;
    let bl=i32::try_from(b.len()).map_err(|_|io::ErrorKind::InvalidInput)?;
    match unsafe{CompareStringOrdinal(a.as_ptr(),al,b.as_ptr(),bl,1)} as u32{
        CSTR_LESS_THAN=>Ok(std::cmp::Ordering::Less),CSTR_EQUAL=>Ok(std::cmp::Ordering::Equal),CSTR_GREATER_THAN=>Ok(std::cmp::Ordering::Greater),_=>Err(io::Error::last_os_error()),
    }
}
// Hold every directory from the volume root to the image's parent without
// FILE_SHARE_DELETE. Reject reparse points; keeping only the final image handle
// does not protect an ancestor rename before CreateProcess resolves the path.
fn lock_ancestors(executable:&Path)->io::Result<Vec<File>>{
    let mut components=executable.components();
    match components.next(){Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_))=>{},_=>return Err(io::Error::new(io::ErrorKind::InvalidInput,"absolute local disk path required"))}
    if components.next()!=Some(Component::RootDir) {return Err(io::ErrorKind::InvalidInput.into());}
    if components.any(|c|!matches!(c,Component::Normal(_))) {return Err(io::ErrorKind::InvalidInput.into());}
    let parent=executable.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let mut paths:Vec<_>=parent.ancestors().collect();paths.reverse();
    let mut guards=Vec::new();
    for path in paths{
        let file=OpenOptions::new().access_mode(FILE_READ_ATTRIBUTES).share_mode(FILE_SHARE_READ|FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT|FILE_FLAG_BACKUP_SEMANTICS).open(path)?;
        use std::os::windows::fs::MetadataExt;
        let metadata=file.metadata()?;
        if !metadata.is_dir()||metadata.file_attributes()&FILE_ATTRIBUTE_REPARSE_POINT!=0 {return Err(io::Error::new(io::ErrorKind::InvalidData,"executable ancestor is not a non-reparse directory"));}
        guards.push(file);
    }
    Ok(guards)
}
struct Attributes(Vec<usize>,bool);
impl Attributes {
    fn new(handles:&[HANDLE])->io::Result<Self> {
        let mut size=0;
        unsafe{InitializeProcThreadAttributeList(ptr::null_mut(),1,0,&mut size);}
        if size==0 {return Err(io::Error::last_os_error());}
        let mut this=Self(vec![0;(size+size_of::<usize>()-1)/size_of::<usize>()],false);
        if unsafe{InitializeProcThreadAttributeList(this.0.as_mut_ptr().cast(),1,0,&mut size)}==0 {return Err(io::Error::last_os_error());}
        this.1=true;
        if unsafe{UpdateProcThreadAttribute(this.0.as_mut_ptr().cast(),0,PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,handles.as_ptr().cast(),std::mem::size_of_val(handles),ptr::null_mut(),ptr::null_mut())}==0 {return Err(io::Error::last_os_error());}
        Ok(this)
    }
}
impl Drop for Attributes {fn drop(&mut self){if self.1 {unsafe{DeleteProcThreadAttributeList(self.0.as_mut_ptr().cast())}}}}

/// A created child is returned even when the post-create identity query fails.
/// No Drop implementation sends a signal, and no operation reopens by PID.
/// Host process CPU accounting, not guest-vCPU time.
#[derive(Debug,Clone,Copy)]
pub struct ProcessCpuTimes { pub kernel:Duration, pub user:Duration }
fn cpu_duration(time:FILETIME)->Duration {
    let ticks=((time.dwHighDateTime as u64)<<32)|time.dwLowDateTime as u64;
    Duration::new(ticks/10_000_000,((ticks%10_000_000)*100) as u32)
}
pub struct Child {
    process:OwnedHandle,
    pub pid:u32,
    pub creation_time:Result<u64,i32>,
}
impl Child {
    pub fn wait(&self,budget:Duration)->io::Result<Option<u32>> {
        let ms=budget.as_millis().min((u32::MAX-1) as u128) as u32;
        match unsafe{WaitForSingleObject(raw(&self.process),ms)} {
            WAIT_TIMEOUT=>Ok(None),
            WAIT_OBJECT_0=>{let mut exit=0;if unsafe{GetExitCodeProcess(raw(&self.process),&mut exit)}==0 {Err(io::Error::last_os_error())}else{Ok(Some(exit))}},
            _=>Err(io::Error::last_os_error()),
        }
    }
    pub fn id(&self)->u32 {self.pid}
    /// Consume the original CreateProcess capability, never reopen by PID.
    pub fn into_owned_handle(self)->OwnedHandle {self.process}
    pub fn try_wait_status(&self)->io::Result<Option<std::process::ExitStatus>> {
        use std::os::windows::process::ExitStatusExt;
        Ok(self.wait(Duration::ZERO)?.map(std::process::ExitStatus::from_raw))
    }
    /// Observe the retained original process. No PID reopen or termination.
    pub fn wait_status(&self)->io::Result<std::process::ExitStatus> {
        use std::os::windows::process::ExitStatusExt;
        match unsafe{WaitForSingleObject(raw(&self.process),u32::MAX)} {
            WAIT_OBJECT_0=>{
                let mut exit=0;
                if unsafe{GetExitCodeProcess(raw(&self.process),&mut exit)}==0 {return Err(io::Error::last_os_error());}
                Ok(std::process::ExitStatus::from_raw(exit))
            },
            _=>Err(io::Error::last_os_error()),
        }
    }
    pub fn cpu_times(&self)->io::Result<ProcessCpuTimes> {
        let mut times:[FILETIME;4]=unsafe{std::mem::zeroed()};
        if unsafe{GetProcessTimes(raw(&self.process),times.as_mut_ptr(),times.as_mut_ptr().add(1),times.as_mut_ptr().add(2),times.as_mut_ptr().add(3))}==0 {return Err(io::Error::last_os_error());}
        Ok(ProcessCpuTimes{kernel:cpu_duration(times[2]),user:cpu_duration(times[3])})
    }
    pub fn handle(&self)->BorrowedHandle<'_> {use std::os::windows::io::AsHandle;self.process.as_handle()}
}

/// Dedicated, non-inheritable duplicates with stable values until consumed.
/// No raw-handle adoption is exposed: dropping this set cannot close a caller's
/// original handle. Borrowed values must not be closed, reused or published as
/// independently owned resources. Only spawn_prepared enables inheritance,
/// under the same global spawn gate as spawn, and closes them before unlocking.
#[derive(Default)]
pub struct PreparedHandles { copies:Vec<OwnedHandle> }
impl PreparedHandles {
    pub fn new()->Self {Self::default()}
    pub fn push_duplicate(&mut self, original:BorrowedHandle<'_>)->io::Result<usize> {
        let copy=duplicate(original)?;
        let mut flags=0;
        if unsafe{GetHandleInformation(raw(&copy),&mut flags)}==0 {return Err(io::Error::last_os_error());}
        if flags&HANDLE_FLAG_INHERIT!=0 {return Err(io::Error::new(io::ErrorKind::InvalidData,"new duplicate is inheritable"));}
        let index=self.copies.len();self.copies.push(copy);Ok(index)
    }
    pub fn get(&self,index:usize)->Option<BorrowedHandle<'_>> {
        use std::os::windows::io::AsHandle;
        self.copies.get(index).map(|h|h.as_handle())
    }
}

/// `arguments` sees only this launch's dedicated duplicates. The caller builds
/// the typed consumer records using those numbers; originals are never marked
/// inheritable. Standard handles are separate duplicates, not consumer records.
/// Parent reader ownership remains with the caller on every return path.
pub fn spawn(
    executable:&Path, expected_executable:&File, inherited:&[BorrowedHandle<'_>],
    standard:[BorrowedHandle<'_>;3], arguments:impl FnOnce(&[RawHandle])->io::Result<Vec<OsString>>,
    environment:&[(OsString,OsString)],
)->io::Result<Child> {
    let mut prepared=PreparedHandles::new();
    for handle in inherited {prepared.push_duplicate(*handle)?;}
    spawn_prepared(executable,expected_executable,prepared,standard,arguments,environment)
}

/// Consume exactly the prepared objects used to encode composite references.
/// No second duplication changes their numeric values. Every return closes the
/// parent's dedicated copies; originals and parent readers remain caller-owned.
pub fn spawn_prepared(
    executable:&Path, expected_executable:&File, prepared:PreparedHandles,
    standard:[BorrowedHandle<'_>;3], arguments:impl FnOnce(&[RawHandle])->io::Result<Vec<OsString>>,
    environment:&[(OsString,OsString)],
)->io::Result<Child> {
    if !executable.is_absolute() {return Err(io::ErrorKind::InvalidInput.into());}
    // Deny write/delete while resolving and creating the executable. A process
    // retains the image after launch; this guard is not a pathname-only proof.
    let _ancestors=lock_ancestors(executable)?;
    let image=OpenOptions::new().read(true).share_mode(FILE_SHARE_READ).custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(executable)?;
    use std::os::windows::fs::MetadataExt;
    let metadata=image.metadata()?;
    if !metadata.is_file()||metadata.file_attributes()&FILE_ATTRIBUTE_REPARSE_POINT!=0 {return Err(io::Error::new(io::ErrorKind::InvalidData,"executable is not a regular non-reparse file"));}
    if file_id(&image)?!=file_id(expected_executable)? {return Err(io::Error::new(io::ErrorKind::InvalidData,"executable identity changed"));}
    let mut copies=prepared.copies;
    let args=arguments(&copies.iter().map(|h|h.as_raw_handle()).collect::<Vec<_>>())?;
    let standard_start=copies.len();
    for handle in standard {copies.push(duplicate(handle)?);}
    let handles:Vec<_>=copies.iter().map(raw).collect();
    let app=wide(executable.as_os_str())?;
    let mut command=quote(executable.as_os_str())?;
    for arg in args {command.push(b' ' as u16);command.extend(quote(&arg)?);}
    command.push(0);
    let mut sorted=environment.to_vec();let mut sort_error=None;
    sorted.sort_by(|(a,_),(b,_)|match ordinal(a,b){Ok(order)=>order,Err(e)=>{sort_error=Some(e);std::cmp::Ordering::Equal}});
    if let Some(error)=sort_error{return Err(error);}
    let mut block=Vec::new();let mut previous:Option<OsString>=None;
    for (key,value) in sorted {
        if key.is_empty() || key.encode_wide().any(|c|c==b'=' as u16) {return Err(io::ErrorKind::InvalidInput.into());}
        if let Some(prior)=&previous {if ordinal(prior,&key)?.is_eq(){return Err(io::ErrorKind::InvalidInput.into());}}
        previous=Some(key.clone());
        let mut entry=key;entry.push("=");entry.push(value);block.extend(wide(&entry)?);
    }
    block.push(0);if block.len()==1 {block.push(0);}
    let mut attributes=Attributes::new(&handles)?;
    let mut startup:STARTUPINFOEXW=unsafe{std::mem::zeroed()};
    startup.StartupInfo.cb=size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags=STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput=handles[standard_start];startup.StartupInfo.hStdOutput=handles[standard_start+1];startup.StartupInfo.hStdError=handles[standard_start+2];
    startup.lpAttributeList=attributes.0.as_mut_ptr().cast();
    let mut info:PROCESS_INFORMATION=unsafe{std::mem::zeroed()};
    let gate=SPAWN_GATE.lock().map_err(|_|io::Error::new(io::ErrorKind::Other,"spawn gate poisoned"))?;
    // On ALL exits copies must drop before the gate. Keep this closure's cleanup
    // explicit; failure while enabling inheritance cannot leave a visible copy.
    let created=(|| {
        for handle in &handles {
            let mut flags=0;
            if unsafe{GetHandleInformation(*handle,&mut flags)}==0 {return Err(io::Error::last_os_error());}
            if flags&HANDLE_FLAG_INHERIT!=0 {return Err(io::Error::new(io::ErrorKind::InvalidData,"prepared duplicate already inheritable"));}
            if unsafe{SetHandleInformation(*handle,HANDLE_FLAG_INHERIT,HANDLE_FLAG_INHERIT)}==0 {return Err(io::Error::last_os_error());}
        }
        if unsafe{CreateProcessW(app.as_ptr(),command.as_mut_ptr(),ptr::null(),ptr::null(),1,EXTENDED_STARTUPINFO_PRESENT|CREATE_UNICODE_ENVIRONMENT,block.as_ptr().cast(),ptr::null(),&startup.StartupInfo,&mut info)}==0 {return Err(io::Error::last_os_error());}
        // Immediately adopt original handles. No fallible post-create return.
        let process=unsafe{OwnedHandle::from_raw_handle(info.hProcess as RawHandle)};
        let thread=unsafe{OwnedHandle::from_raw_handle(info.hThread as RawHandle)};drop(thread);
        let mut times:[FILETIME;4]=unsafe{std::mem::zeroed()};
        let creation_time=if unsafe{GetProcessTimes(raw(&process),times.as_mut_ptr(),times.as_mut_ptr().add(1),times.as_mut_ptr().add(2),times.as_mut_ptr().add(3))}==0 {
            Err(io::Error::last_os_error().raw_os_error().unwrap_or(-1))
        }else{Ok(((times[0].dwHighDateTime as u64)<<32)|times[0].dwLowDateTime as u64)};
        Ok(Child{process,pid:info.dwProcessId,creation_time})
    })();
    drop(copies);drop(gate);created
}
