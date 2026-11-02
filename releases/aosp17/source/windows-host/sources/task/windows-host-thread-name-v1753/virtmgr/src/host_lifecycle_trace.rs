//! Host-only observation of authenticated lifecycle events. Never a readiness source by itself.
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
const LIMIT: u64 = 8 * 1024 * 1024;
static TRACE: OnceLock<Option<Mutex<Trace>>> = OnceLock::new();
struct Trace { file: File, bytes: u64, failed: bool }
impl Trace {
    #[cfg(unix)]
    fn create(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).create_new(true).mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path)?;
        if !file.metadata()?.is_file() { return Err(io::Error::other("trace is not a regular file")); }
        Ok(Self {file, bytes: 0, failed: false})
    }
    #[cfg(windows)]
    fn create(path: &Path) -> io::Result<Self> {
        let parent=path.parent().ok_or_else(||io::Error::other("trace parent missing"))?;
        let name=path.file_name().ok_or_else(||io::Error::other("trace filename missing"))?;
        let guard=crate::desktop_lease::ExistingPrivateDirectoryGuard::open(parent)?;
        let file=guard.create_trace(name)?;
        Ok(Self {file,bytes:0,failed:false})
    }
    fn append(&mut self, line: &str) -> io::Result<()> {
        if self.failed { return Err(io::Error::other("trace failure is sticky")); }
        if !line.ends_with('\n') || line[..line.len()-1].contains('\n') || self.bytes + line.len() as u64 > LIMIT {
            self.failed = true; return Err(io::Error::other("invalid or oversized lifecycle trace"));
        }
        match self.file.write_all(line.as_bytes()).and_then(|_| self.file.flush()) {
            Ok(()) => { self.bytes += line.len() as u64; Ok(()) },
            Err(e) => { self.failed = true; Err(e) }
        }
    }
}
pub fn initialize() -> io::Result<()> {
    let trace = std::env::var_os("VIRTMGR_TRACE_FILE").map(|p| Trace::create(Path::new(&p)).map(Mutex::new)).transpose()?;
    TRACE.set(trace).map_err(|_| io::Error::other("lifecycle trace initialized twice"))
}
pub fn record(line: String) {
    if let Some(Some(trace)) = TRACE.get() {
        let result = trace.lock().map_err(|_| io::Error::other("trace mutex poisoned")).and_then(|mut t| t.append(&line));
        if let Err(e) = result { log::error!("lifecycle evidence unavailable: {e}"); }
    }
}
#[cfg(all(test,unix))] mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn path() -> std::path::PathBuf { std::env::temp_dir().join(format!("a17-trace-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::SeqCst))) }
    #[test] fn preserve_existing_and_symlink() {
        let p=path(); let mut t=Trace::create(&p).unwrap(); t.append("notifyPayloadReady cid=2048\n").unwrap();
        assert!(Trace::create(&p).is_err()); let link=path(); std::os::unix::fs::symlink(&p,&link).unwrap(); assert!(Trace::create(&link).is_err());
        assert_eq!(std::fs::read_to_string(&p).unwrap(),"notifyPayloadReady cid=2048\n"); std::fs::remove_file(link).unwrap();std::fs::remove_file(p).unwrap();
    }
    #[test] fn bounded_sticky_error() {
        let p=path();let mut t=Trace::create(&p).unwrap();t.bytes=LIMIT;assert!(t.append("x\n").is_err());t.bytes=0;assert!(t.append("x\n").is_err());assert_eq!(std::fs::metadata(&p).unwrap().len(),0);std::fs::remove_file(p).unwrap();
    }
    #[test] fn no_partial_event_or_missing_parent() {
        assert!(Trace::create(&path().join("missing/trace")).is_err());let p=path();let mut t=Trace::create(&p).unwrap();assert!(t.append("ready\nforged\n").is_err());assert!(t.append("later\n").is_err());std::fs::remove_file(p).unwrap();
    }
    #[test] fn concurrent_complete_records() {
        let p=path();let t=Arc::new(Mutex::new(Trace::create(&p).unwrap()));let hs:Vec<_>=(0..8).map(|n|{let t=t.clone();std::thread::spawn(move||for i in 0..16 {t.lock().unwrap().append(&format!("cid={n} seq={i}\n")).unwrap();})}).collect();for h in hs{h.join().unwrap();}let s=std::fs::read_to_string(&p).unwrap();assert_eq!(s.lines().count(),128);assert_eq!(s.lines().collect::<std::collections::HashSet<_>>().len(),128);std::fs::remove_file(p).unwrap();
    }
}

#[cfg(all(test,windows))] mod windows_tests {
 use super::*;
 use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
 use std::os::windows::io::AsRawHandle;
 static NEXT:AtomicUsize=AtomicUsize::new(0);
 fn path()->std::path::PathBuf {std::env::temp_dir().join(format!("bscp-trace-component-v1560-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::SeqCst)))}
 // Test append on an ordinary unprivileged fixture only. This deliberately
 // bypasses create and is NOT proof of private production trace admission.
 fn ordinary()->(std::path::PathBuf,Trace){let p=path();let f=OpenOptions::new().write(true).read(true).create_new(true).open(&p).unwrap();(p,Trace{file:f,bytes:0,failed:false})}
 #[test]fn missing_or_relative_parent_is_rejected(){assert!(Trace::create(Path::new("relative/trace")).is_err());assert!(Trace::create(&path().join("absent/trace")).is_err());}
 #[test]fn complete_records_and_original_handle_noninherit(){
  let(p,t)=ordinary();let mut flags=0;assert_ne!(unsafe{windows_sys::Win32::Foundation::GetHandleInformation(t.file.as_raw_handle() as _,&mut flags)},0);assert_eq!(flags&1,0);
  let t=Arc::new(Mutex::new(t));let workers:Vec<_>=(0..4).map(|id|{let t=t.clone();std::thread::spawn(move||for n in 0..16{t.lock().unwrap().append(&format!("id={id} n={n}\n")).unwrap();})}).collect();for w in workers{w.join().unwrap();}
  let s=std::fs::read_to_string(p).unwrap();assert_eq!(s.lines().count(),64);assert_eq!(s.lines().collect::<std::collections::HashSet<_>>().len(),64);
 }
 #[test]fn invalid_and_limit_failures_are_sticky(){let(_,mut t)=ordinary();assert!(t.append("x\nforged\n").is_err());assert!(t.append("valid\n").is_err());let(_,mut t)=ordinary();t.bytes=LIMIT;assert!(t.append("x\n").is_err());t.bytes=0;assert!(t.append("x\n").is_err());}
 #[test]fn actual_write_error_is_sticky(){let(p,t)=ordinary();drop(t);let file=File::open(p).unwrap();let mut t=Trace{file,bytes:0,failed:false};assert!(t.append("x\n").is_err());assert!(t.failed);assert!(t.append("x\n").is_err());}
}
