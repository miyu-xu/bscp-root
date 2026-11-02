// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::os::windows::fs::OpenOptionsExt;

use base::info;
use base::read_overlapped_blocking;
use cros_async::Executor;
use winapi::um::winbase::FILE_FLAG_NO_BUFFERING;
use winapi::um::winbase::FILE_FLAG_OVERLAPPED;
use winapi::um::winnt::FILE_SHARE_READ;

use crate::DiskFileParams;
use crate::Error;
use crate::Result;
use crate::SingleFileDisk;

impl SingleFileDisk {
    pub fn new(disk: File, ex: &Executor) -> Result<Self> {
        ex.async_overlapped_from(disk)
            .map_err(Error::CreateSingleFileDisk)
            .map(|inner| SingleFileDisk { inner })
    }
}

pub fn open_raw_disk_image(params: &DiskFileParams) -> Result<File> {
    let mut options = File::options();
    options.read(true).write(!params.is_read_only);
    if params.lock {
        if params.is_read_only {
            // Shared read-only file access.
            options.share_mode(FILE_SHARE_READ);
        } else {
            // Exclusive file access.
            options.share_mode(0);
        }
    }

    let mut flags = 0;
    if params.is_direct {
        info!("Opening disk file with no buffering");
        flags |= FILE_FLAG_NO_BUFFERING;
    }
    if params.is_overlapped {
        info!("Opening disk file for overlapped IO");
        flags |= FILE_FLAG_OVERLAPPED;
    }
    if flags != 0 {
        options.custom_flags(flags);
    }

    use base::sys::windows::inherited_handles::{self, Role};
    let inherited=inherited_handles::is_registered_reference(&params.path)
        .map_err(|e| Error::OpenFile(params.path.display().to_string(), e.into()))?;
    if inherited && params.lock {
        return Err(Error::OpenFile(params.path.display().to_string(),std::io::Error::new(std::io::ErrorKind::Unsupported,"cannot impose a new sharing lock on an inherited file").into()));
    }
    if inherited_handles::registered_role(&params.path)
        .map_err(|e| Error::OpenFile(params.path.display().to_string(),e.into()))? == Some(Role::CompositeRoot) {
        if params.depth != 0 || params.is_overlapped || params.is_direct {
            return Err(Error::OpenFile(params.path.display().to_string(),std::io::Error::new(
                std::io::ErrorKind::Unsupported,"composite-root requires depth-zero synchronous buffered metadata").into()));
        }
        // Only the official composite metadata is read-only. Child partition
        // write rights are checked separately from each component's declaration.
        let metadata=inherited_handles::open_path(&params.path,&options,&[Role::CompositeRoot],1,Some(false),Some(false))
            .map_err(|e| Error::OpenFile(params.path.display().to_string(),e.into()))?;
        if crate::detect_image_type(&metadata,false)? != crate::ImageType::CompositeDisk {
            return Err(Error::OpenFile(params.path.display().to_string(),std::io::Error::new(
                std::io::ErrorKind::InvalidData,"composite-root must contain a supported composite image").into()));
        }
        return Ok(metadata);
    }
    let roles=if params.depth==0 { &[Role::Disk][..] } else { &[Role::Composite,Role::Metadata][..] };
    let raw_image=inherited_handles::open_path(&params.path,&options,roles,if params.is_read_only{1}else{3},Some(params.is_overlapped),Some(params.is_direct))
        .map_err(|e| Error::OpenFile(params.path.display().to_string(),e.into()))?;

    Ok(raw_image)
}

/// On Windows, if the file is sparse, we set the option. On Linux this is not needed.
pub fn apply_raw_disk_file_options(raw_image: &File, is_sparse_file: bool) -> Result<()> {
    if is_sparse_file {
        base::set_sparse_file(raw_image).map_err(Error::SetSparseFailure)?;
    }
    Ok(())
}

pub fn read_from_disk(
    mut file: &File,
    offset: u64,
    buf: &mut [u8],
    overlapped_mode: bool,
) -> Result<()> {
    file.seek(SeekFrom::Start(offset))
        .map_err(Error::SeekingFile)?;
    if overlapped_mode {
        read_overlapped_blocking(file, offset, buf)
            .map(|_| ())
            .map_err(Error::ReadingHeader)
    } else {
        file.read_exact(buf).map_err(Error::ReadingHeader)
    }
}

#[cfg(test)]
mod inherited_disk_tests {
    use super::*;
    use std::os::windows::io::AsRawHandle;
    use std::io::Write;
    use std::ffi::c_void;
    #[link(name="kernel32")]
    extern "system" {
        fn GetCurrentProcess()->*mut c_void;
        fn DuplicateHandle(a:*mut c_void,b:*mut c_void,c:*mut c_void,d:*mut *mut c_void,e:u32,f:i32,g:u32)->i32;
    }
    fn open_case(role:&str,contents:&[u8],disk_writable:bool,cap_writable:bool,depth:u32)->bool {
        use base::sys::windows::inherited_handles::{install,native_snapshot::snapshot};
        let mut temp=tempfile::NamedTempFile::new().unwrap();
        temp.write_all(contents).unwrap();temp.flush().unwrap();
        let original=File::options().read(true).write(cap_writable).open(temp.path()).unwrap();
        let process=unsafe {GetCurrentProcess()};let mut imported=std::ptr::null_mut();
        assert_ne!(unsafe {DuplicateHandle(process,original.as_raw_handle(),process,&mut imported,0,1,2)},0);
        let m=snapshot(imported,true).unwrap();
        let id=m.id.iter().map(|b|format!("{b:02x}")).collect::<String>();
        let text=format!(r#"{{"handle":"{}","role":"{}","kind":"file","access":{},"mode":{},"volume":"{:x}","file_id":"{}","pipe_flags":0,"pipe_state":0}}"#,imported as usize,role,m.access,m.mode,m.volume,id);
        let guard=unsafe {install(&[text])}.unwrap();
        let result=open_raw_disk_image(&DiskFileParams{path:format!("handle:{}",imported as usize).into(),is_read_only:!disk_writable,is_sparse_file:false,is_overlapped:false,is_direct:false,lock:false,depth});
        let passed=result.is_ok();drop(result);drop(guard);passed
    }
    #[test]
    fn inherited_root_metadata_never_grants_plain_disk_write() {
        assert!(open_case("disk",b"plain raw disk",false,false,0));
        assert!(!open_case("disk",b"plain raw disk",true,false,0));
        assert!(open_case("disk",b"plain raw disk",true,true,0));
        assert!(!open_case("composite-root",b"plain raw disk",true,false,0));
        assert!(!open_case("composite-root",b"composite_disk\x1d",true,false,1));
        assert!(!open_case("composite",b"plain raw disk",true,false,1));
        assert!(open_case("composite",b"plain raw disk",true,true,1));
        // Without the official composite decoder feature this role is unsupported.
        assert_eq!(open_case("composite-root",b"composite_disk\x1d",true,false,0),cfg!(feature="composite-disk"));
    }
}
