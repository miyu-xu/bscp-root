// Shared read-only Windows native identity/access/mode query. No tests or startup mutation.
use std::{ffi::c_void, io};
type Handle = *mut c_void;
#[repr(C)]
struct IoStatus {
    status: usize,
    information: usize,
}
#[repr(C)]
#[derive(Default)]
struct FileId {
    volume: u64,
    id: [u8; 16],
}
#[repr(C)]
#[derive(Default)]
struct AttrTag {
    attributes: u32,
    tag: u32,
}
#[link(name = "kernel32")]
extern "system" {
    fn GetHandleInformation(h: Handle, flags: *mut u32) -> i32;
    fn GetFileType(h: Handle) -> u32;
    fn GetFileInformationByHandleEx(h: Handle, class: u32, out: *mut c_void, size: u32) -> i32;
    fn GetNamedPipeHandleStateW(
        h: Handle,
        state: *mut u32,
        instances: *mut u32,
        count: *mut u32,
        timeout: *mut u32,
        user: *mut u16,
        size: u32,
    ) -> i32;
    fn GetNamedPipeInfo(
        h: Handle,
        flags: *mut u32,
        out: *mut u32,
        input: *mut u32,
        max: *mut u32,
    ) -> i32;

}
#[link(name = "ntdll")]
extern "system" {
    fn NtQueryInformationFile(
        h: Handle,
        ios: *mut IoStatus,
        out: *mut c_void,
        len: u32,
        class: u32,
    ) -> i32;
}
fn bad(s: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, s)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub kind: u32,
    pub access: u32,
    pub mode: u32,
    pub volume: u64,
    pub id: [u8; 16],
    pub pipe_flags: u32,
    pub pipe_state: u32,
}
fn query_u32(h: Handle, class: u32) -> io::Result<u32> {
    let mut out = 0u32;
    let mut ios = IoStatus {
        status: 0,
        information: 0,
    };
    let st =
        unsafe { NtQueryInformationFile(h, &mut ios, &mut out as *mut _ as *mut c_void, 4, class) };
    if st != 0 || ios.status as i32 != 0 || ios.information != 4 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "native query {class} failed {st:#x} size {}",
                ios.information
            ),
        ));
    }
    Ok(out)
}
pub fn snapshot(h: Handle, require_inherit: bool) -> io::Result<Snapshot> {
    let mut flags = 0;
    if unsafe { GetHandleInformation(h, &mut flags) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if require_inherit && flags & 1 == 0 {
        return Err(bad("handle is not inherited"));
    }
    let kind = unsafe { GetFileType(h) };
    if kind != 1 && kind != 3 {
        return Err(bad("not disk/pipe"));
    }
    let mut s = Snapshot {
        kind,
        access: query_u32(h, 8)?,
        mode: query_u32(h, 16)?,
        volume: 0,
        id: [0; 16],
        pipe_flags: 0,
        pipe_state: 0,
    };
    if kind == 1 {
        let mut a = AttrTag::default();
        if unsafe {
            GetFileInformationByHandleEx(
                h,
                9,
                &mut a as *mut _ as *mut c_void,
                std::mem::size_of::<AttrTag>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if a.attributes & (0x10 | 0x400) != 0 {
            return Err(bad("directory/reparse file"));
        }
        let mut id = FileId::default();
        if unsafe {
            GetFileInformationByHandleEx(
                h,
                18,
                &mut id as *mut _ as *mut c_void,
                std::mem::size_of::<FileId>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        s.volume = id.volume;
        s.id = id.id;
    } else if unsafe {
        GetNamedPipeInfo(
            h,
            &mut s.pipe_flags,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if kind == 3
        && unsafe {
            GetNamedPipeHandleStateW(
                h,
                &mut s.pipe_state,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
        } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(s)
}
