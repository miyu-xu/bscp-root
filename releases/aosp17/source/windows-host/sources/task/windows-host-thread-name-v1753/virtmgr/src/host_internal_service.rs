// Local, unprivileged desktop implementation of A17's VM context service.
// No Android authorization or unavailable HAL service is simulated.
use crate::{aidl, desktop_authority};
use android_system_virtualizationservice_internal::aidl::android::system::virtualizationservice_internal::IVirtualizationServiceInternal::{BnVirtualizationServiceInternal, IVirtualizationServiceInternal, VmContext::VmContext};
use android_system_virtualizationcommon::aidl::android::system::virtualizationcommon::{Atom::Atom,Certificate::Certificate};
use android_system_virtualizationservice_internal::aidl::android::system::virtualizationservice_internal::IBoundDevice::IBoundDevice;
use binder::{BinderFeatures,ExceptionCode,Interface,ParcelFileDescriptor,SpIBinder,Status,Strong};
use std::{collections::{HashMap},sync::{Arc,Mutex,LazyLock},path::PathBuf};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt,MetadataExt};
#[derive(Default)]
struct State { contexts: HashMap<i32,(PathBuf,std::fs::File)>, registered:HashMap<i32,binder::Weak<dyn aidl::IVirtualMachine>>, owned:HashMap<i32,Strong<dyn aidl::IVirtualMachine>>, claims:HashMap<[u8;64],std::fs::File>,
 // Drop field last: pins outlive the service-owned VM, claims and lease fields.
 #[cfg(windows)] _directories:HashMap<i32,crate::desktop_lease::ExistingPrivateDirectoryGuard>
}
#[derive(Default,Clone)]
struct Service { state:Arc<Mutex<State>> }
impl Interface for Service {}
fn err(message:impl ToString)->Status {Status::new_exception_str(ExceptionCode::ILLEGAL_STATE,Some(message.to_string()))}
fn unsupported<T>() -> binder::Result<T> {Err(Status::new_exception_str(ExceptionCode::UNSUPPORTED_OPERATION,Some("Android-only service unavailable on desktop")))}
fn authorize()->binder::Result<()> {desktop_authority::current().and_then(|a|a.check("android.permission.MANAGE_VIRTUAL_MACHINE")).map_err(|e|Status::new_exception_str(ExceptionCode::SECURITY,Some(e.to_string())))}
fn local_service()-> &'static Service {
 static S:LazyLock<Service>=LazyLock::new(Service::default); &S
}
impl Service {
 fn release_owned_registration(&self,cid:i32)->binder::Result<()> {
  let removed={
   let mut s=self.state.lock().map_err(err)?;
   if s.registered.remove(&cid).is_none(){return Err(err("CID not registered"));}
   s.owned.remove(&cid)
  };
  // Local Binder destruction can re-enter VM shutdown. Never run it with State locked.
  drop(removed);Ok(())
 }
 fn retain_for_start(&self,cid:i32)->binder::Result<()> {
  let mut s=self.state.lock().map_err(err)?;
  if s.owned.contains_key(&cid){return Err(err("VM already handed to service owner"));}
  let vm=s.registered.get(&cid).ok_or_else(||err("CID not registered"))?.upgrade().map_err(err)?;
  s.owned.insert(cid,vm);Ok(())
 }
}
pub(crate) fn unregister_owned_vm(cid:i32)->binder::Result<()> {local_service().release_owned_registration(cid)}
pub fn retain_for_start(cid:i32)->binder::Result<()> { local_service().retain_for_start(cid) }
pub fn global_service()->Strong<dyn IVirtualizationServiceInternal> {
 static S:LazyLock<Strong<dyn IVirtualizationServiceInternal>>=LazyLock::new(||BnVirtualizationServiceInternal::new_binder(local_service().clone(),BinderFeatures::default())); S.clone()
}
impl IVirtualizationServiceInternal for Service {
 fn removeMemlockRlimit(&self)->binder::Result<()> {
  authorize()?;
  #[cfg(unix)] {crate::process_limits::remove_memlock_limit().map_err(err)}
  // Windows has no Unix memlock rlimit. Do not report a limit change or bypass authorization.
  #[cfg(windows)] {unsupported()}
 }
 fn allocateVmContext(&self)->binder::Result<VmContext> {
  authorize()?;
  let base=std::env::var("VIRTMGR_GUEST_CID").unwrap_or_else(|_|"2048".into()).parse::<i32>().map_err(err)?;
  if base<3{return Err(err("invalid guest CID"));}
  let root=std::env::var_os("VIRTMGR_TEMP_ROOT").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
  if !root.is_absolute(){return Err(err("temporary root must be absolute"));}
  #[cfg(unix)] {
  let meta=std::fs::symlink_metadata(&root).map_err(err)?;
  let uid=desktop_authority::current().map_err(err)?.identity().uid;
  if !meta.is_dir() || meta.uid()!=uid || meta.mode()&0o022!=0 {return Err(err("temporary root must be a private owned directory"));}
  }
  #[cfg(windows)]
  let private_parent=crate::desktop_lease::ExistingPrivateDirectoryGuard::open(&root).map_err(err)?;
  let mut state=self.state.lock().map_err(err)?;let cid=base.checked_add(state.contexts.len().try_into().map_err(err)?).ok_or_else(||err("CID exhausted"))?;
  let path=root.join(format!("virtmgr-{}-{cid}",std::process::id()));
  let path_string=path.to_str().ok_or_else(||err("non UTF-8 instance path"))?.to_owned();
  let lease=crate::desktop_lease::acquire(&format!("cid-{cid}")).map_err(err)?;
  #[cfg(unix)]
  std::fs::DirBuilder::new().mode(0o700).create(&path).map_err(err)?;
  #[cfg(windows)] {
   let directory=private_parent.create_private_child(&format!("virtmgr-{}-{cid}",std::process::id())).map_err(err)?;
   state._directories.insert(cid,directory);
  }
  state.contexts.insert(cid,(path,lease));
  Ok(VmContext{cid,tempDir:path_string})
 }
 fn registerVirtualMachine(&self,cid:i32,vm:&Strong<dyn aidl::IVirtualMachine>)->binder::Result<()> {
  authorize()?;let mut s=self.state.lock().map_err(err)?;
  if !s.contexts.contains_key(&cid)||s.registered.contains_key(&cid){return Err(err("unknown or already registered CID"));}
  // Creation is weak: abandoning an unstarted object must not leave a service-owned VM.
  // start() promotes it before spawning, and monitor_vm_exit releases it after exit.
  s.registered.insert(cid,Strong::downgrade(vm));Ok(())
 }
 fn unregisterVirtualMachine(&self,cid:i32)->binder::Result<()> {authorize()?;self.release_owned_registration(cid)}

 fn forwardAtom(&self,_atom:&Atom,_requesterUid:i32,_vmIdentifier:&str)->binder::Result<()> {unsupported()}
 fn debugListVms(&self)->binder::Result<Vec<aidl::VirtualMachineDebugInfo>> {
  desktop_authority::current().and_then(|a|a.check("android.permission.DUMP")).map_err(|e|Status::new_exception_str(ExceptionCode::SECURITY,Some(e.to_string())))?;
  // Clone real live objects under State, call them only after releasing its lock.
  let vms={let state=self.state.lock().map_err(err)?;state.registered.values().filter_map(|vm|vm.upgrade().ok()).collect::<Vec<_>>()};
  vms.iter().map(|vm|vm.getDebugInfo()).collect()
 }
 fn requestAttestation(&self,_csr:&[u8],_requesterUid:i32,_testMode:bool)->binder::Result<Vec<Certificate>> {unsupported()}
 fn enableTestAttestation(&self)->binder::Result<()> {unsupported()}
 fn isRemoteAttestationSupported(&self)->binder::Result<bool> {Ok(false)}
 fn getAssignableDevices(&self)->binder::Result<Vec<aidl::AssignableDevice>> {unsupported()}
 fn bindDevicesToVfioDriver(&self,_devices:&[String])->binder::Result<Vec<Strong<dyn IBoundDevice>>> {unsupported()}
 fn getDtboFile(&self)->binder::Result<ParcelFileDescriptor> {unsupported()}
 fn allocateInstanceId(&self)->binder::Result<[u8;64]> {
  authorize()?;let mut id=[0;64];
  #[cfg(unix)] {use std::io::Read;std::fs::File::open("/dev/urandom").and_then(|mut f|f.read_exact(&mut id)).map_err(err)?;}
  #[cfg(windows)] {crate::windows_entropy::fill(&mut id).map_err(err)?;}
  Ok(id)
 }
 fn claimVmInstance(&self,id:&[u8;64])->binder::Result<()> {authorize()?;let mut s=self.state.lock().map_err(err)?;if s.claims.contains_key(id){return Err(err("instance already claimed"));}let lease=crate::desktop_lease::acquire(&format!("instance-{}",hex::encode(id))).map_err(err)?;s.claims.insert(*id,lease);Ok(())}
 fn removeVmInstance(&self,id:&[u8;64])->binder::Result<()> {authorize()?;if self.state.lock().map_err(err)?.claims.remove(id).is_none(){return Err(err("instance not claimed"));}Ok(())}
 fn setDisplayService(&self,_ibinder:&SpIBinder)->binder::Result<()> {unsupported()}
 fn clearDisplayService(&self)->binder::Result<()> {unsupported()}
 fn waitDisplayService(&self)->binder::Result<SpIBinder> {unsupported()}
 fn createTapInterface(&self,_ifaceNameSuffix:&str)->binder::Result<ParcelFileDescriptor> {unsupported()}
 fn deleteTapInterface(&self,_tapFd:&ParcelFileDescriptor)->binder::Result<()> {unsupported()}
 fn claimSecretkeeperEntry(&self,_id:&[u8;64])->binder::Result<()> {unsupported()}
 fn startOrStopAdbd(&self,_cid:i32,_start:bool)->binder::Result<()> {unsupported()}
}
#[cfg(all(test,unix))]mod tests {
 use super::*;
 use std::os::{fd::AsFd,unix::net::UnixStream};
 #[test]fn actual_service_allocation_claim_and_failure_contract(){
  let (a,_b)=UnixStream::pair().unwrap();crate::desktop_authority::initialize(a.as_fd(),unsafe{libc::getpid()}).unwrap();
  let root=std::env::temp_dir().join(format!("formal-context-test-{}",std::process::id()));std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
  std::env::set_var("VIRTMGR_TEMP_ROOT",&root);std::env::set_var("VIRTMGR_GUEST_CID","52951");
  let first=Service::default();let second=Service::default();
  let context=first.allocateVmContext().unwrap();assert_eq!(context.cid,52951);assert!(second.allocateVmContext().is_err());
  assert!(std::path::Path::new(&context.tempDir).is_dir());
  let id=first.allocateInstanceId().unwrap();first.claimVmInstance(&id).unwrap();assert!(first.claimVmInstance(&id).is_err());assert!(second.claimVmInstance(&id).is_err());
  first.removeVmInstance(&id).unwrap();assert!(first.removeVmInstance(&id).is_err());second.claimVmInstance(&id).unwrap();second.removeVmInstance(&id).unwrap();
  assert!(first.unregisterVirtualMachine(context.cid).is_err());assert!(first.debugListVms().unwrap().is_empty());assert!(first.enableTestAttestation().is_err());assert!(!first.isRemoteAttestationSupported().unwrap());
  std::env::set_var("VIRTMGR_TEMP_ROOT","relative-untrusted");let count=first.state.lock().unwrap().contexts.len();assert!(first.allocateVmContext().is_err());assert_eq!(first.state.lock().unwrap().contexts.len(),count);
  std::env::remove_var("VIRTMGR_TEMP_ROOT");std::env::remove_var("VIRTMGR_GUEST_CID");
 }
}

#[cfg(all(test,unix))] mod ownership_contract_tests {
 use super::*;
 use std::sync::atomic::{AtomicUsize,AtomicBool,Ordering};
 use std::os::{fd::AsFd,unix::net::UnixStream};
 use android_system_virtualizationservice::aidl::android::system::virtualizationservice::IVirtualMachine::BnVirtualMachine;
 // Test-only Binder object: all business methods reject; no VM or success stub.
 struct ObservedVm { drops:Arc<AtomicUsize>, unlocked:Arc<AtomicBool>, state:std::sync::Weak<Mutex<State>> }
 impl Interface for ObservedVm {}
 impl Drop for ObservedVm { fn drop(&mut self) {
  self.drops.fetch_add(1,Ordering::SeqCst);
  self.unlocked.store(self.state.upgrade().unwrap().try_lock().is_ok(),Ordering::SeqCst);
 }}
 impl aidl::IVirtualMachine for ObservedVm {
  fn r#getCid<'a, >(&'a self) -> binder::Result<i32> { unsupported() }
  fn r#getState<'a, >(&'a self) -> binder::Result<android_system_virtualizationservice::aidl::android::system::virtualizationservice::VirtualMachineState::VirtualMachineState> { unsupported() }
  fn r#registerCallback<'a, 'l1, >(&'a self, _arg_callback: &'l1 binder::Strong<dyn android_system_virtualizationservice::aidl::android::system::virtualizationservice::IVirtualMachineCallback::IVirtualMachineCallback>) -> binder::Result<()> { unsupported() }
  fn r#start<'a, >(&'a self) -> binder::Result<()> { unsupported() }
  fn r#stop<'a, >(&'a self) -> binder::Result<()> { unsupported() }
  fn r#isMemoryBalloonEnabled<'a, >(&'a self) -> binder::Result<bool> { unsupported() }
  fn r#getActualMemoryBalloonBytes<'a, >(&'a self) -> binder::Result<i64> { unsupported() }
  fn r#setMemoryBalloon<'a, >(&'a self, _arg_num_bytes: i64) -> binder::Result<()> { unsupported() }
  fn r#connectVsock<'a, >(&'a self, _arg_port: i32) -> binder::Result<binder::ParcelFileDescriptor> { unsupported() }
  fn r#createAccessorBinder<'a, 'l1, >(&'a self, _arg_instance: &'l1 str, _arg_port: i32) -> binder::Result<binder::SpIBinder> { unsupported() }
  fn r#setHostConsoleName<'a, 'l1, >(&'a self, _arg_pathname: &'l1 str) -> binder::Result<()> { unsupported() }
  fn r#suspend<'a, >(&'a self) -> binder::Result<()> { unsupported() }
  fn r#resume<'a, >(&'a self) -> binder::Result<()> { unsupported() }
  fn r#getDebugInfo<'a, >(&'a self) -> binder::Result<android_system_virtualizationservice::aidl::android::system::virtualizationservice::VirtualMachineDebugInfo::VirtualMachineDebugInfo> { unsupported() }
  fn r#addMemoryToGuest<'a, 'l1, >(&'a self, _arg_fd: &'l1 binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::Result<i32> { unsupported() }
  fn r#removeMemoryFromGuest<'a, >(&'a self, _arg_memory_id: i32) -> binder::Result<()> { unsupported() }
  fn r#getGuestAgent<'a, >(&'a self) -> binder::Result<Option<binder::Strong<dyn android_system_virtualizationcommon::aidl::android::system::virtualizationcommon::IGuestAgent::IGuestAgent>>> { unsupported() }
  fn r#addDisplay<'a, 'l1, >(&'a self, _arg_config: &'l1 android_system_virtualizationservice::aidl::android::system::virtualizationservice::DisplayConfig::DisplayConfig) -> binder::Result<()> { unsupported() }
  fn r#removeDisplay<'a, >(&'a self, _arg_displayId: i32) -> binder::Result<()> { unsupported() }
  fn r#getDisplays<'a, >(&'a self) -> binder::Result<Vec<android_system_virtualizationservice::aidl::android::system::virtualizationservice::VirtualMachineDisplay::VirtualMachineDisplay>> { unsupported() }
  fn r#startHostVsockTcpBridge<'a, >(&'a self, _arg_hostPort: i32, _arg_guestPort: i32) -> binder::Result<()> { unsupported() }
 }
 #[test] fn actual_binder_weak_create_strong_start_and_unlocked_release() {
  // Bootstrap is intentionally process-global and one-shot. Test in its own
  // process rather than relaxing production authentication initialization.
  if std::env::var_os("BSCP_OWNER_CONTRACT_CHILD").is_none() {
   let status=std::process::Command::new(std::env::current_exe().unwrap())
    .args(["--exact","host_internal_service::ownership_contract_tests::actual_binder_weak_create_strong_start_and_unlocked_release","--nocapture"])
    .env("BSCP_OWNER_CONTRACT_CHILD","1").status().unwrap();
   assert!(status.success());return;
  }
  let (socket,_peer)=UnixStream::pair().unwrap();
  crate::desktop_authority::initialize(socket.as_fd(),unsafe{libc::getpid()}).unwrap();
  let service=Service::default();
  // Context files below are test-owned references, not VM disks or leases.
  service.state.lock().unwrap().contexts.insert(52953,(PathBuf::from("unused-test-context"),std::fs::File::open("/dev/null").unwrap()));
  for started in [false,true] {
   let drops=Arc::new(AtomicUsize::new(0));let unlocked=Arc::new(AtomicBool::new(false));
   let vm=BnVirtualMachine::new_binder(ObservedVm{drops:drops.clone(),unlocked:unlocked.clone(),state:Arc::downgrade(&service.state)},BinderFeatures::default());
   let weak=Strong::downgrade(&vm);
   service.registerVirtualMachine(52953,&vm).unwrap();
   assert!(service.registerVirtualMachine(52953,&vm).is_err());
   if started { service.retain_for_start(52953).unwrap();assert!(service.retain_for_start(52953).is_err()); }
   drop(vm);
   if started { assert_eq!(drops.load(Ordering::SeqCst),0);assert!(weak.upgrade().is_ok()); }
   else { assert_eq!(drops.load(Ordering::SeqCst),1);assert!(weak.upgrade().is_err());assert!(service.retain_for_start(52953).is_err()); }
   service.unregisterVirtualMachine(52953).unwrap();
   assert_eq!(drops.load(Ordering::SeqCst),1);assert!(unlocked.load(Ordering::SeqCst));assert!(weak.upgrade().is_err());
   assert!(service.unregisterVirtualMachine(52953).is_err());assert!(service.retain_for_start(52953).is_err());
  }
 }
}

#[cfg(all(test,unix))] mod rpc_concurrency_tests {
 use super::*;
 use std::os::{fd::AsFd,unix::net::UnixStream};
 use std::sync::{Barrier,atomic::{AtomicBool,Ordering}};
 #[test] fn actual_rpc_concurrent_claim_and_shutdown() {
  const NAME:&str="host_internal_service::rpc_concurrency_tests::actual_rpc_concurrent_claim_and_shutdown";
  if std::env::var_os("BSCP_RPC_CONTRACT_CHILD").is_none() {
   let status=std::process::Command::new(std::env::current_exe().unwrap())
    .args(["--exact",NAME,"--nocapture"]).env("BSCP_RPC_CONTRACT_CHILD","1").status().unwrap();
   assert!(status.success());return;
  }
  // Only this test subprocess exits on a stalled transport; no signal or VM.
  let finished=Arc::new(AtomicBool::new(false));let watch=finished.clone();
  std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_secs(30));if !watch.load(Ordering::SeqCst){eprintln!("RPC_CONTRACT_DEADLINE");std::process::exit(124);}});
  let (server_fd,client_fd)=UnixStream::pair().unwrap();
  crate::desktop_authority::initialize(server_fd.as_fd(),unsafe{libc::getpid()}).unwrap();
  let implementation=Service::default();
  let root=BnVirtualizationServiceInternal::new_binder(implementation.clone(),BinderFeatures::default());
  let server=rpcbinder::RpcServer::new_unix_domain_bootstrap(root.as_binder(),server_fd.into()).unwrap();
  server.set_supported_file_descriptor_transport_modes(&[rpcbinder::FileDescriptorTransportMode::Unix]);server.set_max_threads(8);server.start();
  let mut clients=Vec::new();let mut sessions=Vec::new();
  for _ in 0..8 {
   let session=rpcbinder::RpcSession::new();session.set_file_descriptor_transport_mode(rpcbinder::FileDescriptorTransportMode::Unix);
   let client:Strong<dyn IVirtualizationServiceInternal>=session.setup_unix_domain_bootstrap_client(client_fd.as_fd()).unwrap();
   clients.push(client);sessions.push(session);
  }
  let id=clients[0].allocateInstanceId().unwrap();let control=clients[0].clone();
  let barrier=Arc::new(Barrier::new(9));let mut threads=Vec::new();
  for client in clients {let b=barrier.clone();threads.push(std::thread::spawn(move||{b.wait();client.claimVmInstance(&id).is_ok()}));}
  barrier.wait();let winners=threads.into_iter().map(|t|t.join().unwrap() as usize).sum::<usize>();
  assert_eq!(winners,1);assert_eq!(implementation.state.lock().unwrap().claims.len(),1);
  control.removeVmInstance(&id).unwrap();assert!(control.removeVmInstance(&id).is_err());
  assert!(implementation.state.lock().unwrap().claims.is_empty());
  assert!(control.enableTestAttestation().is_err());
  server.shutdown().unwrap();assert!(control.allocateInstanceId().is_err());
  drop(control);drop(sessions);drop(server);drop(client_fd);
  finished.store(true,Ordering::SeqCst);
  println!("REAL_RPC sessions=8 claim_winners=1 release=confirmed shutdown_error=confirmed no_vm=true");
 }
}


#[cfg(all(test,target_os="linux"))]
mod authenticated_listener_tests {
 use super::*;
 use std::os::unix::fs::PermissionsExt;
 use std::process::{Command,Stdio};
 const NAME:&str="host_internal_service::authenticated_listener_tests::actual_listener_authority_and_independent_clients";
 #[test]
 fn actual_listener_authority_and_independent_clients(){
  let role=std::env::var("BSCP_AUTH_TEST_ROLE").unwrap_or_default();
  if role.is_empty(){
   let output=Command::new(std::env::current_exe().unwrap()).args(["--exact",NAME,"--nocapture"]).env("BSCP_AUTH_TEST_ROLE","server").output().unwrap();
   print!("{}",String::from_utf8_lossy(&output.stdout));eprint!("{}",String::from_utf8_lossy(&output.stderr));assert!(output.status.success());return;
  }
  std::thread::spawn(||{std::thread::sleep(std::time::Duration::from_secs(40));eprintln!("AUTH_TEST_DEADLINE");std::process::exit(124);});
  if role=="client" || role=="denied" {
   let session=rpcbinder::RpcSession::new();session.set_file_descriptor_transport_mode(rpcbinder::FileDescriptorTransportMode::Unix);
   let socket=std::env::var("BSCP_AUTH_TEST_SOCKET").unwrap();let service:Strong<dyn IVirtualizationServiceInternal>=session.setup_preconnected_client(move||{use std::os::fd::IntoRawFd;std::os::unix::net::UnixStream::connect(&socket).ok().map(|s|s.into_raw_fd())}).unwrap();
   if role=="denied"{assert!(service.allocateInstanceId().is_err());println!("RUST_NATIVE_MISSING_CONTEXT_DENIED");return;}
   for _ in 0..16 {let id=service.allocateInstanceId().unwrap();assert_eq!(id.len(),64);}
   assert!(service.enableTestAttestation().is_err());
   println!("RUST_AUTH_CLIENT pid={} requests=16 actual_service=PASS",std::process::id());return;
  }
  assert_eq!(role,"server");
  crate::desktop_authority::initialize_listener().unwrap();
  assert!(crate::desktop_authority::current().unwrap().request_identity().is_err());
  let implementation=Service::default();assert!(implementation.allocateInstanceId().is_err());
  let base=std::path::PathBuf::from(std::env::var("BSCP_AUTH_TEST_ROOT").unwrap()).join(format!("server-{}",std::process::id()));
  std::fs::create_dir(&base).unwrap();std::fs::set_permissions(&base,std::fs::Permissions::from_mode(0o700)).unwrap();
  let root=BnVirtualizationServiceInternal::new_binder(implementation,BinderFeatures::default());
  let path=base.join("rpc.sock");
  let endpoint=crate::desktop_listener::bind(&path).unwrap();
  let server=rpcbinder::RpcServer::new_bound_socket(root.as_binder(),endpoint).unwrap();
  server.enable_desktop_authentication().unwrap();
  server.set_supported_file_descriptor_transport_modes(&[rpcbinder::FileDescriptorTransportMode::Unix]);server.set_max_threads(4);server.start();
  let mut clients=Vec::new();
  for _ in 0..2{
   clients.push(Command::new(std::env::current_exe().unwrap()).args(["--exact",NAME,"--nocapture"]).env("BSCP_AUTH_TEST_ROLE","client").env("BSCP_AUTH_TEST_SOCKET",&path).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap());
  }
  for child in clients{let output=child.wait_with_output().unwrap();print!("{}",String::from_utf8_lossy(&output.stdout));assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));}
  assert!(crate::desktop_authority::current().unwrap().request_identity().is_err());
  server.shutdown().unwrap();drop(server);
  let denied=base.join("denied.sock");let endpoint=crate::desktop_listener::bind(&denied).unwrap();
  let server=rpcbinder::RpcServer::new_bound_socket(root.as_binder(),endpoint).unwrap();
  server.set_supported_file_descriptor_transport_modes(&[rpcbinder::FileDescriptorTransportMode::Unix]);server.start();
  let output=Command::new(std::env::current_exe().unwrap()).args(["--exact",NAME,"--nocapture"]).env("BSCP_AUTH_TEST_ROLE","denied").env("BSCP_AUTH_TEST_SOCKET",&denied).output().unwrap();
  print!("{}",String::from_utf8_lossy(&output.stdout));assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));server.shutdown().unwrap();
  println!("RUST_PRODUCTION_AUTHORITY real_rpc=PASS caller_context_missing=DENIED client_exit_and_pipe_eof=PASS no_vm=true");
 }
}
