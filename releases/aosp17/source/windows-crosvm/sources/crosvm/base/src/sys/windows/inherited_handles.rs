use std::{
    collections::BTreeMap,
    ffi::c_void,
    fs::File,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
};
type Handle = *mut c_void;
#[path = "inherited_handles/native_snapshot.rs"]
pub mod native_snapshot;
use native_snapshot::snapshot;
pub use native_snapshot::Snapshot;
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> Handle;
    fn GetHandleInformation(h: Handle, flags: *mut u32) -> i32;
    fn DuplicateHandle(
        p: Handle,
        h: Handle,
        p2: Handle,
        out: *mut Handle,
        access: u32,
        inherit: i32,
        options: u32,
    ) -> i32;
}
fn bad(s: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, s)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Kernel,
    Initrd,
    Bios,
    Disk,
    Composite,
    CompositeRoot,
    Metadata,
    ConsoleIn,
    ConsoleOut,
    Log,
    Failure,
    Control,
    Ramdump,
    DtbIn,
    DtbOut,
}
impl Role {
    pub fn parse(s: &str) -> io::Result<Self> {
        Ok(match s {
            "kernel" => Self::Kernel,
            "initrd" => Self::Initrd,
            "bios" => Self::Bios,
            "disk" => Self::Disk,
            "composite" => Self::Composite,
            "composite-root" => Self::CompositeRoot,
            "metadata" => Self::Metadata,
            "console-in" => Self::ConsoleIn,
            "console-out" => Self::ConsoleOut,
            "log" => Self::Log,
            "failure" => Self::Failure,
            "control" => Self::Control,
            "ramdump" => Self::Ramdump,
            "dtb-in" => Self::DtbIn,
            "dtb-out" => Self::DtbOut,
            _ => return Err(bad("unknown role")),
        })
    }
    fn allows(self, kind: u32) -> bool {
        match self {
            Self::Failure | Self::Control => kind == 3,
            Self::ConsoleIn | Self::ConsoleOut | Self::Log => kind == 1 || kind == 3,
            _ => kind == 1,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Spec {
    pub value: usize,
    pub role: Role,
    pub expected: Snapshot,
}
fn number(s: &str) -> io::Result<usize> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(bad("invalid decimal handle"));
    }
    let n = s.parse::<usize>().map_err(|_| bad("handle overflow"))?;
    if n == 0 || n >= usize::MAX - 15 {
        return Err(bad("null/pseudo handle"));
    }
    Ok(n)
}
pub fn reference(s: &str) -> io::Result<Option<usize>> {
    if let Some(n) = s.strip_prefix("handle:") {
        Ok(Some(number(n)?))
    } else {
        Ok(None)
    }
}
struct Entry {
    owned: OwnedHandle,
    role: Role,
    metadata: Snapshot,
}
pub struct Registry {
    entries: BTreeMap<usize, Entry>,
}
impl Registry {
    // Caller transfers only dedicated inherited import handles, never stdio handles.
    // On success they are closed in this child after stable noninheritable copies exist.
    // Failed registration leaves the original import handles owned by the caller.
    /// # Safety
    /// Every raw value must be a dedicated startup import owned exclusively by
    /// this caller, kept open and unchanged for this call. It must not alias
    /// stdio or another owner. Success transfers and closes those import handles;
    /// failure transfers none. Startup callers must register before worker threads.
    pub unsafe fn register(specs: &[Spec]) -> io::Result<Self> {
        let mut entries = BTreeMap::new();
        for s in specs {
            number(&s.value.to_string())?;
            if entries.contains_key(&s.value) {
                return Err(bad("duplicate import"));
            }
            let h = s.value as Handle;
            let actual = snapshot(h, true)?;
            if actual != s.expected || !s.role.allows(actual.kind) {
                return Err(bad("identity/type/access/mode mismatch"));
            }
            if s.role == Role::Control
                && (actual.kind != 3
                    // GetNamedPipeInfo documents SERVER_END | TYPE_MESSAGE (5).
                    // Current Windows also returns REJECT_REMOTE_CLIENTS (8).
                    // Accept only these exact representations; the trusted
                    // producer still creates local-only pipes. The complete
                    // native snapshot must match the supplied original identity.
                    || !matches!(actual.pipe_flags, 5 | 13)
                    || actual.pipe_state & 3 != 2
                    || actual.mode & 0x30 != 0
                    || actual.access & 3 != 3)
            {
                return Err(bad(
                    "control requires message/wait/overlapped duplex server listener",
                ));
            }
            let owned = duplicate(h)?;
            if snapshot(owned.as_raw_handle(), false)? != actual {
                return Err(bad("identity changed while duplicating"));
            }
            entries.insert(
                s.value,
                Entry {
                    owned,
                    role: s.role,
                    metadata: actual,
                },
            );
        }
        // All validation succeeded; originals are dedicated startup capabilities.
        for s in specs {
            drop(unsafe { OwnedHandle::from_raw_handle(s.value as Handle) });
        }
        Ok(Self { entries })
    }
    pub fn open(
        &self,
        path: &str,
        role: Role,
        required_access: u32,
        overlapped: Option<bool>,
        direct: Option<bool>,
    ) -> io::Result<File> {
        let n = reference(path)?.ok_or_else(|| bad("not a handle reference"))?;
        let e = self.entries.get(&n).ok_or_else(|| bad("unlisted handle"))?;
        if e.role != role {
            return Err(bad("wrong purpose"));
        }
        if required_access == 0 || required_access & !7 != 0 {
            return Err(bad("invalid access requirement"));
        }
        if e.metadata.access & required_access != required_access {
            return Err(bad("missing read/write access"));
        }
        if let Some(v) = overlapped {
            if (e.metadata.mode & 0x30 == 0) != v {
                return Err(bad("overlapped mismatch"));
            }
        }
        if let Some(v) = direct {
            if (e.metadata.mode & 8 != 0) != v {
                return Err(bad("direct IO mismatch"));
            }
        }
        if snapshot(e.owned.as_raw_handle(), false)? != e.metadata {
            return Err(bad("native metadata changed after registration"));
        }
        // No create/truncate/append/share-mode transformation is supported here.
        Ok(File::from(duplicate_for_consumer(
            e.owned.as_raw_handle(), &e.metadata, required_access,
        )?))
    }
}
// Registry ownership retains the original capability. Each consumer receives only
// the requested data access plus already-granted read-only metadata/wait rights.
// In particular a read-only composite parent must not propagate a child's write
// capability merely because the parent supplied a stronger original handle.
fn duplicate_for_consumer(h: Handle, original: &Snapshot, required_access: u32) -> io::Result<OwnedHandle> {
    const READ_METADATA_AND_WAIT: u32 = 0x0008 | 0x0080 | 0x0002_0000 | 0x0010_0000;
    let access = required_access | (original.access & READ_METADATA_AND_WAIT);
    let mut out = std::ptr::null_mut();
    let process = unsafe { GetCurrentProcess() };
    // No DUPLICATE_SAME_ACCESS: that would silently retain excess write rights.
    if unsafe { DuplicateHandle(process, h, process, &mut out, access, 0, 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let owned = unsafe { OwnedHandle::from_raw_handle(out) };
    let mut flags = 0;
    if unsafe { GetHandleInformation(out, &mut flags) } == 0 || flags & 1 != 0 {
        return Err(bad("consumer duplicate inherited unexpectedly"));
    }
    let mut expected = original.clone();
    expected.access = access;
    if snapshot(owned.as_raw_handle(), false)? != expected {
        return Err(bad("consumer duplicate identity/access/mode mismatch"));
    }
    Ok(owned)
}
fn duplicate(h: Handle) -> io::Result<OwnedHandle> {
    let mut out = std::ptr::null_mut();
    let p = unsafe { GetCurrentProcess() };
    if unsafe { DuplicateHandle(p, h, p, &mut out, 0, 0, 2) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let owned = unsafe { OwnedHandle::from_raw_handle(out) };
    let mut flags = 0;
    if unsafe { GetHandleInformation(out, &mut flags) } == 0 || flags & 1 != 0 {
        return Err(bad("duplicate inherited unexpectedly"));
    }
    Ok(owned)
}

// The command-line metadata is serialized as one JSON argument per capability.
// JSON derives reject duplicate and unknown fields; raw OS values are queried
// again in this process, never trusted solely because they were serialized.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSpec {
    handle: String, role: String, kind: String, access: u32, mode: u32,
    volume: String, file_id: String, pipe_flags: u32, pipe_state: u32,
}
pub fn parse_spec(text: &str) -> io::Result<Spec> {
    let w: WireSpec = serde_json::from_str(text).map_err(|e|bad(&format!("invalid inherited-handle JSON: {e}")))?;
    let value=number(&w.handle)?;let role=Role::parse(&w.role)?;
    let kind=match w.kind.as_str(){"file"=>1,"pipe"=>3,_=>return Err(bad("unknown handle kind"))};
    if w.volume.is_empty()||w.volume.len()>16||!w.volume.bytes().all(|b|b.is_ascii_hexdigit())||w.file_id.len()!=32||!w.file_id.bytes().all(|b|b.is_ascii_hexdigit()){return Err(bad("invalid native file identity encoding"))}
    let volume=u64::from_str_radix(&w.volume,16).map_err(|_|bad("volume overflow"))?;let mut id=[0;16];for (i,byte) in id.iter_mut().enumerate(){*byte=u8::from_str_radix(&w.file_id[i*2..i*2+2],16).map_err(|_|bad("invalid file ID"))?;}
    if (kind==1&&(w.pipe_flags!=0||w.pipe_state!=0))||(kind==3&&(volume!=0||id!=[0;16])){return Err(bad("cross-kind metadata"))}
    Ok(Spec{value,role,expected:Snapshot{kind,access:w.access,mode:w.mode,volume,id,pipe_flags:w.pipe_flags,pipe_state:w.pipe_state}})
}
static ACTIVE: std::sync::Mutex<Option<std::sync::Arc<Registry>>> = std::sync::Mutex::new(None);
pub struct RegistrationGuard{active:bool}
impl Drop for RegistrationGuard{fn drop(&mut self){if self.active{ACTIVE.lock().unwrap_or_else(|p|p.into_inner()).take();}}}
/// Install only at the ordinary `run` startup, before log initialization/workers.
/// # Safety
/// Specs must describe exclusive dedicated startup imports from the trusted
/// parent's STARTUPINFOEX list. No thread may concurrently close/reuse them.
pub unsafe fn install(texts:&[String])->io::Result<RegistrationGuard>{
    let mut slot=ACTIVE.lock().map_err(|_|bad("registry poisoned"))?;if slot.is_some(){return Err(bad("registry already installed"))}if texts.is_empty(){return Ok(RegistrationGuard{active:false})}
    let specs=texts.iter().map(|s|parse_spec(s)).collect::<io::Result<Vec<_>>>()?;
    #[link(name="kernel32")] extern "system"{fn GetStdHandle(which:u32)->Handle;}
    for s in &specs{for n in [-10i32,-11,-12]{if s.value as Handle==unsafe{GetStdHandle(n as u32)}{return Err(bad("stdio requires a dedicated import duplicate"))}}}
    // SAFETY: upheld by the startup caller and the explicit dedicated import contract.
    *slot=Some(std::sync::Arc::new(unsafe{Registry::register(&specs)}?));Ok(RegistrationGuard{active:true})
}
// Reserved syntax is recognized before consulting ACTIVE. Never fall back to a
// filesystem open (including Windows ADS syntax) for an unregistered capability.
fn path_reference(path:&std::path::Path)->io::Result<Option<(&str,usize)>> {
    use std::os::windows::ffi::OsStrExt;
    if !path.as_os_str().encode_wide().take(7).eq("handle:".encode_utf16()) { return Ok(None); }
    let text=path.to_str().ok_or_else(||bad("invalid Unicode in handle reference"))?;
    let value=reference(text)?.ok_or_else(||bad("invalid handle reference"))?;
    Ok(Some((text,value)))
}
pub fn registered_role(path:&std::path::Path)->io::Result<Option<Role>> {
    let Some((_,value))=path_reference(path)? else { return Ok(None); };
    let active=ACTIVE.lock().map_err(|_|bad("registry poisoned"))?;
    let reg=active.as_ref().ok_or_else(||bad("handle registry not installed"))?;
    Ok(Some(reg.entries.get(&value).ok_or_else(||bad("unlisted handle"))?.role))
}
pub fn is_registered_reference(path:&std::path::Path)->io::Result<bool>{
    registered_role(path).map(|role|role.is_some())
}
pub fn open_if_handle(path:&std::path::Path,roles:&[Role],access:u32,overlapped:Option<bool>,direct:Option<bool>)->io::Result<Option<File>>{
    let Some((text,value))=path_reference(path)? else { return Ok(None); };
    let slot=ACTIVE.lock().map_err(|_|bad("registry poisoned"))?;
    let reg=slot.as_ref().ok_or_else(||bad("handle registry not installed"))?;
    let entry=reg.entries.get(&value).ok_or_else(||bad("unlisted handle"))?;if !roles.contains(&entry.role){return Err(bad("wrong consumer role"))}
    reg.open(text,entry.role,access,overlapped,direct).map(Some)
}
#[cfg(test)]mod json_tests{use super::*;
const VALID:&str=r#"{"handle":"123","role":"disk","kind":"file","access":3,"mode":32,"volume":"1234","file_id":"00000000000000000000000000000001","pipe_flags":0,"pipe_state":0}"#;
#[test]fn strict_json_contract(){assert!(parse_spec(VALID).is_ok());assert!(parse_spec(&VALID.replace("\"access\":3","\"access\":3,\"access\":1")).is_err());assert!(parse_spec(&VALID.replace("\"mode\":32","\"extra\":0,\"mode\":32")).is_err());assert!(parse_spec(&VALID.replace("\"handle\":\"123\"","\"handle\":123")).is_err());assert!(parse_spec(&VALID.replace("\"kind\":\"file\"","\"kind\":\"pipe\"")).is_err());}
}

/// Preserve ordinary paths; handle references require an explicit consumer contract.
pub fn open_path(path:&std::path::Path,options:&std::fs::OpenOptions,roles:&[Role],access:u32,overlapped:Option<bool>,direct:Option<bool>)->io::Result<File>{
 if let Some(file)=open_if_handle(path,roles,access,overlapped,direct)?{Ok(file)}else{options.open(path)}
}
/// Existing log/serial path semantics are append/create. Imported regular files
/// must already be append-only capabilities; duplication cannot retrofit flags.
pub fn open_output_path(path:&std::path::Path,roles:&[Role])->io::Result<File>{
 let Some((text,value))=path_reference(path)? else {
  return std::fs::OpenOptions::new().create(true).append(true).open(path);
 };
 let slot=ACTIVE.lock().map_err(|_|bad("registry poisoned"))?;
 let reg=slot.as_ref().ok_or_else(||bad("handle registry not installed"))?;
 let e=reg.entries.get(&value).ok_or_else(||bad("unlisted handle"))?;if !roles.contains(&e.role){return Err(bad("wrong output purpose"))}
 let access=if e.metadata.kind==1{if e.metadata.access&2!=0||e.metadata.access&4==0{return Err(bad("file output requires original append-only access"))}4}else{2};
 reg.open(text,e.role,access,Some(false),None)
}

#[cfg(test)]
mod reserved_path_tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn capability_reference_requires_registration() {
        assert!(ACTIVE.lock().unwrap().is_none());
        let path=Path::new("handle:123");
        // This first assertion reproduces the old defect without opening an ADS.
        assert!(is_registered_reference(path).is_err());
        assert!(open_if_handle(path,&[Role::Disk],1,None,None).is_err());
        assert!(open_output_path(path,&[Role::Log]).is_err());
        assert!(is_registered_reference(Path::new("handle:not-decimal")).is_err());
        let empty=unsafe { install(&[]) }.unwrap();
        assert!(is_registered_reference(path).is_err());
        assert!(open_if_handle(path,&[Role::Disk],1,None,None).is_err());
        assert!(open_output_path(path,&[Role::Log]).is_err());
        drop(empty);
        assert!(!is_registered_reference(Path::new("ordinary-file")).unwrap());
        assert!(open_if_handle(Path::new("ordinary-file"),&[Role::Disk],1,None,None).unwrap().is_none());
        let original=tempfile::tempfile().unwrap();
        let mut imported=std::ptr::null_mut();
        let process=unsafe { GetCurrentProcess() };
        assert_ne!(unsafe { DuplicateHandle(process,original.as_raw_handle(),process,&mut imported,0,1,2) },0);
        let metadata=snapshot(imported,true).unwrap();
        let id=metadata.id.iter().map(|b|format!("{b:02x}")).collect::<String>();
        let text=serde_json::json!({"handle":(imported as usize).to_string(),"role":"disk","kind":"file","access":metadata.access,"mode":metadata.mode,"volume":format!("{:x}",metadata.volume),"file_id":id,"pipe_flags":0,"pipe_state":0}).to_string();
        let registered=unsafe { install(&[text]) }.unwrap();
        let named=format!("handle:{}",imported as usize);
        assert!(is_registered_reference(Path::new(&named)).unwrap());
        let file=open_if_handle(Path::new(&named),&[Role::Disk],1,Some(false),Some(false)).unwrap().unwrap();
        assert_eq!(snapshot(file.as_raw_handle(),false).unwrap(),metadata);
        assert!(open_if_handle(Path::new(&named),&[Role::Kernel],1,None,None).is_err());
        assert!(is_registered_reference(Path::new("handle:1")).is_err());
        assert!(open_if_handle(Path::new("handle:1"),&[Role::Disk],1,None,None).is_err());
        drop(file);drop(registered);
        assert!(is_registered_reference(Path::new(&named)).is_err());
    }
}

/// Serial file-output purposes share the original append-file/write-pipe contract.
pub fn open_serial_output_path(path:&std::path::Path)->io::Result<File>{
 open_output_path(path,&[Role::ConsoleOut,Role::Log,Role::Failure,Role::Ramdump])
}

#[cfg(test)]
mod serial_output_tests {
 use super::*;
 use std::io::{Read,Write,Seek,SeekFrom};
 use std::path::Path;
 #[link(name="kernel32")] extern "system" {
  fn CreatePipe(read:*mut Handle,write:*mut Handle,attrs:*mut c_void,size:u32)->i32;
  fn CloseHandle(h:Handle)->i32;
 }
 fn import(h:Handle,role:&str,access:Option<u32>)->(String,Handle){
  let mut raw=std::ptr::null_mut();let p=unsafe{GetCurrentProcess()};
  assert_ne!(unsafe{DuplicateHandle(p,h,p,&mut raw,access.unwrap_or(0),1,if access.is_some(){0}else{2})},0);
  let m=snapshot(raw,true).unwrap();
  let text=serde_json::json!({"handle":(raw as usize).to_string(),"role":role,"kind":if m.kind==1{"file"}else{"pipe"},"access":m.access,"mode":m.mode,"volume":format!("{:x}",m.volume),"file_id":m.id.iter().map(|v|format!("{v:02x}")).collect::<String>(),"pipe_flags":m.pipe_flags,"pipe_state":m.pipe_state}).to_string();
  (text,raw)
 }
 #[test] fn actual_serial_output_caps(){
  assert!(ACTIVE.lock().unwrap().is_none());
  let mut original=tempfile::tempfile().unwrap();original.write_all(b"old").unwrap();
  // Narrow a dedicated original grant to append-only, READ_ATTRIBUTES and SYNCHRONIZE.
  let (text,raw)=import(original.as_raw_handle(),"ramdump",Some(4|0x80|0x100000));
  let guard=unsafe{install(&[text])}.unwrap();let reference=format!("handle:{}",raw as usize);
  let mut output=open_serial_output_path(Path::new(&reference)).unwrap();
  output.seek(SeekFrom::Start(0)).unwrap();output.write_all(b"new").unwrap();drop(output);drop(guard);
  original.seek(SeekFrom::Start(0)).unwrap();let mut contents=String::new();original.read_to_string(&mut contents).unwrap();assert_eq!(contents,"oldnew");
  let (text,raw)=import(original.as_raw_handle(),"ramdump",None);let guard=unsafe{install(&[text])}.unwrap();
  assert!(open_serial_output_path(Path::new(&format!("handle:{}",raw as usize))).is_err());drop(guard);
  let (text,raw)=import(original.as_raw_handle(),"disk",Some(4|0x80|0x100000));let guard=unsafe{install(&[text])}.unwrap();
  assert!(open_serial_output_path(Path::new(&format!("handle:{}",raw as usize))).is_err());drop(guard);
  let mut read=std::ptr::null_mut();let mut write=std::ptr::null_mut();assert_ne!(unsafe{CreatePipe(&mut read,&mut write,std::ptr::null_mut(),0)},0);
  let reader=unsafe{OwnedHandle::from_raw_handle(read)};let writer=unsafe{OwnedHandle::from_raw_handle(write)};
  let(text,raw)=import(writer.as_raw_handle(),"ramdump",None);assert!(unsafe{install(&[text])}.is_err());assert_ne!(unsafe{CloseHandle(raw)},0);
  let(text,raw)=import(writer.as_raw_handle(),"failure",None);let guard=unsafe{install(&[text])}.unwrap();
  let mut output=open_serial_output_path(Path::new(&format!("handle:{}",raw as usize))).unwrap();output.write_all(b"fail").unwrap();
  let mut reader=File::from(reader);let mut bytes=[0;4];reader.read_exact(&mut bytes).unwrap();assert_eq!(&bytes,b"fail");drop(output);drop(guard);
  println!("SERIAL_CAPS append-only ramdump accepted; write-data rejected; wrong-role rejected; ramdump pipe rejected; failure pipe write accepted");
 }
}
