//! Mac guest-vsock bridge. The wire remains Binder RPC; crosvm owns the bridge.
//! Same-user desktop transport, not a sandbox or protected-VM boundary.
use std::{io, path::PathBuf, os::{fd::OwnedFd, unix::{net::UnixListener,fs::{MetadataExt,PermissionsExt,FileTypeExt}}}};
use rpcbinder::RpcServer;
use binder::SpIBinder;
use std::os::fd::{AsRawFd,FromRawFd};
use std::sync::atomic::{AtomicBool,Ordering};
type AuditSignal=unsafe extern "C" fn(*const u32,i32)->i32;
fn audit_signal()->io::Result<AuditSignal>{
    // The installed libproc exports this generation-bound API but the SDK's
    // link stub does not. Resolve exactly this symbol, with no PID fallback.
    static API:std::sync::OnceLock<Result<AuditSignal,String>>=std::sync::OnceLock::new();
    API.get_or_init(||unsafe{
        let h=libc::dlopen(b"/usr/lib/libproc.dylib\0".as_ptr().cast(),libc::RTLD_NOW|libc::RTLD_LOCAL);
        if h.is_null(){return Err("libproc unavailable".into());}
        let f=libc::dlsym(h,b"proc_signal_with_audittoken\0".as_ptr().cast());
        if f.is_null(){libc::dlclose(h);return Err("generation-bound proc_signal_with_audittoken unavailable".into());}
        // Keep this library reference for the process lifetime of the function.
        Ok(std::mem::transmute::<*mut libc::c_void,AuditSignal>(f))
    }).as_ref().copied().map_err(|e|denied(e))
}

extern "C" {
    static mach_task_self_: u32;
    fn task_name_for_pid(task:u32,pid:i32,name:*mut u32)->i32;
    fn task_info(task:u32,flavor:i32,info:*mut u32,count:*mut u32)->i32;
    fn mach_port_deallocate(task:u32,name:u32)->i32;
    fn BSCP_RpcCallingAuditToken(token:*mut u32,count:usize)->i32;
}
fn denied(s:&str)->io::Error {io::Error::new(io::ErrorKind::PermissionDenied,s)}
fn read_token(port:u32)->io::Result<[u32;8]>{
    let mut token=[0;8];let mut count=8;
    let kr=unsafe{task_info(port,15,token.as_mut_ptr(),&mut count)};
    if kr!=0||count!=8{return Err(denied(&format!("task audit token unavailable: {kr}/{count}")));}
    Ok(token)
}
#[derive(Debug)]
pub struct GuestPeer {port:u32,token:[u32;8],exit_queue:Option<OwnedFd>,exit_seen:AtomicBool,term_attempted:AtomicBool}
impl GuestPeer {
    /// Called for the newly spawned child before any wait/reap thread is started.
    pub fn bind(pid:u32)->io::Result<Self>{
        let mut port=0;
        let kr=unsafe{task_name_for_pid(mach_task_self_,pid.try_into().map_err(|_|denied("pid overflow"))?,&mut port)};
        if kr!=0||port==0{return Err(denied(&format!("child task-name port unavailable: {kr}")));}
        let mut owner=Self{port,token:[0;8],exit_queue:None,exit_seen:AtomicBool::new(false),term_attempted:AtomicBool::new(false)};
        owner.token=read_token(port)?;
        let t=owner.token;
        if t[5]!=pid||t[1]!=unsafe{libc::getuid()}||t[3]!=t[1]||t[2]!=unsafe{libc::getgid()}||t[4]!=t[2]||t[1]==0{return Err(denied("child task identity mismatch"));}
        let raw=unsafe{libc::kqueue()};if raw<0{return Err(io::Error::last_os_error());}
        let kq=unsafe{OwnedFd::from_raw_fd(raw)};
        let mut event:libc::kevent=unsafe{std::mem::zeroed()};event.ident=pid as _;
        event.filter=libc::EVFILT_PROC;event.flags=libc::EV_ADD|libc::EV_ONESHOT;event.fflags=libc::NOTE_EXIT;
        if unsafe{libc::kevent(kq.as_raw_fd(),&event,1,std::ptr::null_mut(),0,std::ptr::null())}<0{return Err(io::Error::last_os_error());}
        if read_token(port)?!=owner.token{return Err(denied("child changed while binding exit notification"));}
        owner.exit_queue=Some(kq);
        Ok(owner)
    }
    pub fn exited(&self)->io::Result<bool>{
        if self.exit_seen.load(Ordering::Acquire){return Ok(true);}
        let kq=self.exit_queue.as_ref().ok_or_else(||denied("exit queue absent"))?;
        let mut event:libc::kevent=unsafe{std::mem::zeroed()};let timeout=libc::timespec{tv_sec:0,tv_nsec:0};
        let n=unsafe{libc::kevent(kq.as_raw_fd(),std::ptr::null(),0,&mut event,1,&timeout)};
        if n<0{return Err(io::Error::last_os_error());}if n==0{return Ok(false);}
        if event.flags&libc::EV_ERROR!=0||event.filter!=libc::EVFILT_PROC||event.ident!=self.token[5] as usize||event.fflags&libc::NOTE_EXIT==0{return Err(denied("unexpected original-task exit event"));}
        self.exit_seen.store(true,Ordering::Release);Ok(true)
    }
    fn terminate_with(&self,signal:impl FnOnce(&[u32;8])->i32)->io::Result<bool>{
        if self.exited()?{return Ok(false);}
        if read_token(self.port)?!=self.token{return Err(denied("original task identity changed before TERM"));}
        if self.term_attempted.swap(true,Ordering::AcqRel){return Err(denied("TERM was already attempted; only observation is allowed"));}
        let rc=signal(&self.token);
        log::warn!("original crosvm audit-token TERM attempted once, pid={}, result={rc}",self.token[5]);
        if rc!=0{return Err(io::Error::from_raw_os_error(rc));}Ok(true)
    }
    pub fn terminate_once(&self)->io::Result<bool>{let signal=audit_signal()?;self.terminate_with(|token|unsafe{signal(token.as_ptr(),libc::SIGTERM)})}
    fn matches(&self,actual:[u32;8])->io::Result<()> {
        if actual!=self.token||read_token(self.port)?!=self.token{return Err(denied("guest RPC is not from the original live crosvm task"));} Ok(())
    }
    pub fn check_call(&self)->io::Result<()> {
        let mut actual=[0;8];let rc=unsafe{BSCP_RpcCallingAuditToken(actual.as_mut_ptr(),8)};
        if rc!=0{return Err(denied("authenticated guest RPC context absent"));}
        self.matches(actual)
    }
}
impl Drop for GuestPeer {fn drop(&mut self){let rc=unsafe{mach_port_deallocate(mach_task_self_,self.port)};if rc!=0{log::error!("task-name port deallocation failed: {rc}");}}}

struct Endpoint {path:PathBuf,dev:u64,ino:u64,cid:u32,uid:u32}
impl Endpoint {
    fn evidence(&self,phase:&str) {
        crate::host_lifecycle_trace::record(format!("guestRpcEndpoint {}\n",serde_json::json!({
            "phase":phase,"cid":self.cid,"pid":std::process::id(),"dev":self.dev,
            "ino":self.ino,"uid":self.uid
        })));
    }
    fn bind(cid:u32)->io::Result<(Self,OwnedFd)>{
        if cid<3{return Err(denied("invalid guest CID"));}
        // This is the existing r14/crosvm Mac host-vsock path contract.
        let path=PathBuf::from(format!("/tmp/binder_rpc_vsock_{cid}_{cid}.sock"));
        // bind itself is exclusive: never unlink or replace an existing endpoint.
        let listener=UnixListener::bind(&path)?;
        let m=std::fs::symlink_metadata(&path)?;
        if !m.file_type().is_socket()||m.uid()!=unsafe{libc::getuid()}{return Err(denied("bound socket identity mismatch"));}
        let owner=Self{path,dev:m.dev(),ino:m.ino(),cid,uid:m.uid()};
        std::fs::set_permissions(&owner.path,std::fs::Permissions::from_mode(0o600))?;
        owner.evidence("bound");
        Ok((owner,listener.into()))
    }
}
impl Drop for Endpoint {fn drop(&mut self){
    match std::fs::symlink_metadata(&self.path){
        Ok(m) if m.file_type().is_socket()&&m.uid()==unsafe{libc::getuid()}&&m.dev()==self.dev&&m.ino()==self.ino=>{
            match std::fs::remove_file(&self.path) {
                Ok(())=>self.evidence("retired"),
                Err(e)=>log::error!("owned guest RPC socket cleanup failed: {e}"),
            }
        },
        Err(e) if e.kind()==io::ErrorKind::NotFound=>{},
        _=>log::error!("guest RPC socket identity changed; retaining path"),
    }
}}
pub struct GuestRpcServer {server:RpcServer,_endpoint:Endpoint}
impl GuestRpcServer {
    pub fn new(service:SpIBinder,cid:u32)->anyhow::Result<Self>{
        let (endpoint,fd)=Endpoint::bind(cid)?;
        let server=RpcServer::new_bound_socket(service,fd)?;
        server.enable_desktop_authentication()?;
        Ok(Self{server,_endpoint:endpoint})
    }
    pub fn start(&self){self.server.start();}
    pub fn shutdown(&self)->anyhow::Result<()>{self.server.shutdown().map_err(Into::into)}
}
// Fields drop in declaration order: server shuts down before endpoint is removed.

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn endpoint_retirement_actor() {
        let role=match std::env::var("BSCP_ENDPOINT_ACTOR") {Ok(r)=>r,Err(_)=>return};
        if role=="guest" {std::thread::sleep(std::time::Duration::from_secs(10));return;}
        assert_eq!(role,"service");
        crate::host_lifecycle_trace::initialize().unwrap();
        let cid:u32=std::env::var("BSCP_ENDPOINT_TEST_CID").unwrap().parse().unwrap();
        assert!(cid>100000);
        let (endpoint,fd)=Endpoint::bind(cid).unwrap();
        let mut child=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","guest_rpc_macos::tests::endpoint_retirement_actor","--nocapture","--test-threads=1"])
            .env("BSCP_ENDPOINT_ACTOR","guest").spawn().unwrap();
        let ready=std::env::var("BSCP_ENDPOINT_TEST_READY").unwrap();
        let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(ready).unwrap();
        use std::io::Write;
        writeln!(f,"{}",serde_json::json!({"service":std::process::id(),"guest":child.id(),"cid":cid})).unwrap();f.sync_all().unwrap();drop(f);
        child.wait().unwrap();drop(fd);drop(endpoint);
        // Independent cooperative self-exit, not a hard guarantee. No helper
        // sends any signal; the production audit owner is exercised externally.
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    #[test]fn actual_task_generation_and_missing_dispatch(){
        let p=GuestPeer::bind(std::process::id()).unwrap();p.matches(p.token).unwrap();
        let mut changed=p.token;changed[7]^=1;assert!(p.matches(changed).is_err());
        assert!(p.check_call().is_err());
    }
    #[test]fn exclusive_endpoint_and_owned_cleanup(){
        let cid=100000+std::process::id();let (o,fd)=Endpoint::bind(cid).unwrap();let path=o.path.clone();
        assert_eq!(std::fs::symlink_metadata(&path).unwrap().mode()&0o777,0o600);
        assert!(Endpoint::bind(cid).is_err());drop(fd);drop(o);assert!(!path.exists());
    }
    #[test]fn replaced_endpoint_is_not_removed(){
        let cid=200000+std::process::id();let (o,fd)=Endpoint::bind(cid).unwrap();let path=o.path.clone();
        std::fs::remove_file(&path).unwrap();let f=std::fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
        drop(fd);drop(o);assert!(path.is_file());drop(f);std::fs::remove_file(path).unwrap();
    }
}


#[cfg(test)]mod rpc_tests {
    use super::*;
    use crate::aidl::{IEncryptedStoreKEK,BnEncryptedStoreKEK};
    use binder::{Interface,BinderFeatures};
    use std::{sync::{Arc,Mutex},io::{Read,Write},os::fd::IntoRawFd};
    // Test-only service; production authentication methods are unchanged.
    struct Probe(Arc<Mutex<Option<GuestPeer>>>);
    impl Interface for Probe {}
    impl IEncryptedStoreKEK for Probe {
        fn getKEK(&self)->binder::Result<Option<Vec<u8>>>{
            self.0.lock().unwrap().as_ref().unwrap().check_call().map_err(|e|binder::Status::new_exception_str(binder::ExceptionCode::SECURITY,Some(e.to_string())))?;
            Ok(Some(vec![1,2]))
        }
        fn onKEKCreated(&self,_:&[u8])->binder::Result<()>{Err(binder::StatusCode::INVALID_OPERATION.into())}
    }
    fn connect(cid:u32)->binder::Strong<dyn IEncryptedStoreKEK>{
        rpcbinder::RpcSession::new().setup_preconnected_client(move||std::os::unix::net::UnixStream::connect(format!("/tmp/binder_rpc_vsock_{cid}_{cid}.sock")).ok().map(|x|x.into_raw_fd())).unwrap()
    }
    #[test]fn rpc_child(){
        let Ok(cid)=std::env::var("GUEST_RPC_PROBE_CID")else{return;};
        std::thread::spawn(||{std::thread::sleep(std::time::Duration::from_secs(20));std::process::exit(90);});
        let mut ready=[0];std::io::stdin().read_exact(&mut ready).unwrap();assert_eq!(ready,[b'R']);
        let svc=connect(cid.parse().unwrap());assert_eq!(svc.getKEK().unwrap(),Some(vec![1,2]));drop(svc);
    }
    #[test]fn actual_child_rpc_and_foreign_caller_denied(){
        let cid=300000+std::process::id();let peer=Arc::new(Mutex::new(None));
        let service=BnEncryptedStoreKEK::new_binder(Probe(peer.clone()),BinderFeatures::default());
        let server=GuestRpcServer::new(service.as_binder(),cid).unwrap();
        let mut child=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","guest_rpc_macos::rpc_tests::rpc_child","--nocapture"])
            .env("GUEST_RPC_PROBE_CID",cid.to_string()).stdin(std::process::Stdio::piped()).spawn().unwrap();
        *peer.lock().unwrap()=Some(GuestPeer::bind(child.id()).unwrap());server.start();
        let foreign=connect(cid);assert!(foreign.getKEK().is_err());drop(foreign);
        child.stdin.take().unwrap().write_all(b"R").unwrap();
        let end=std::time::Instant::now()+std::time::Duration::from_secs(25);
        loop {if let Some(s)=child.try_wait().unwrap(){assert!(s.success());break;} assert!(std::time::Instant::now()<end,"test child exceeded cooperative deadline");std::thread::sleep(std::time::Duration::from_millis(20));}
        {let guard=peer.lock().unwrap();let p=guard.as_ref().unwrap();assert!(p.matches(p.token).is_err());assert!(p.exited().unwrap());}
        server.shutdown().unwrap();drop(server);
    }
}


#[cfg(test)]mod stop_tests {
    use super::*;
    #[test]fn installed_generation_api_resolves_without_signal(){audit_signal().unwrap();}
    #[test]fn attempted_signal_error_is_not_retried(){
        let owner=GuestPeer::bind(std::process::id()).unwrap();let count=std::cell::Cell::new(0);
        assert!(owner.terminate_with(|_|{count.set(count.get()+1);libc::EIO}).is_err());
        assert!(owner.terminate_with(|_|{count.set(count.get()+1);0}).is_err());assert_eq!(count.get(),1);
    }
    #[test]fn successful_attempt_is_not_repeated(){
        let owner=GuestPeer::bind(std::process::id()).unwrap();let count=std::cell::Cell::new(0);
        assert!(owner.terminate_with(|_|{count.set(count.get()+1);0}).unwrap());
        assert!(owner.terminate_with(|_|{count.set(count.get()+1);0}).is_err());assert_eq!(count.get(),1);
    }
}
