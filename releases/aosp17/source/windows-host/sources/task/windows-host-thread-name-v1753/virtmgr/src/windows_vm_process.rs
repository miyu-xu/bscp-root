//! Original process and parent-created control endpoint ownership for Windows.
//! No PID reopening, TerminateProcess, destructor kill, or alternate endpoint.
use std::{io,process::ExitStatus,sync::atomic::{AtomicBool,Ordering},time::Duration};
use desktop_host::windows_process::Child;
use crate::windows_control_endpoint::ControlEndpoint;
pub struct WindowsVmProcess {
    child: Child,
    control: ControlEndpoint,
    exit_requested: AtomicBool,
    listeners:std::sync::Mutex<std::collections::BTreeMap<u32,String>>,
}
impl WindowsVmProcess {
    /// Adopt immediately after successful spawn; this operation cannot fail.
    pub fn from_spawn(child:Child,control:ControlEndpoint)->Self {
        Self{child,control,exit_requested:AtomicBool::new(false),listeners:Default::default()}
    }
    pub fn connect_guest(&self,cid:u32,port:u32)->anyhow::Result<std::os::windows::io::OwnedHandle>{
        use crate::windows_control_endpoint::GuestListenerResult;
        let mut uncertain=self.listeners.lock().map_err(|_|anyhow::anyhow!("listener gate poisoned"))?;
        if let Some(reason)=uncertain.get(&port){anyhow::bail!("{reason}");}
        // One new connection, one setup request. Never retry an uncertain send.
        // A confirmed previous connection is not a permanently cached listener.
        uncertain.insert(port,"listener request outcome unknown".into());
        match self.control.request_guest_listener(port){
            GuestListenerResult::Unknown(e)=>{uncertain.insert(port,e.to_string());return Err(e);},
            GuestListenerResult::Rejected(e)=>{uncertain.remove(&port);return Err(e);},
            GuestListenerResult::Created=>{},
        }
        // Keep the setup and immediate connect serialized. If the connection
        // fails after confirmed creation, retain uncertainty/owned listener.
        let stream=crate::vsock_transport::connect_bound(cid,port,self.child.handle())?;
        uncertain.remove(&port);Ok(stream)
    }
    pub fn id(&self)->u32 {self.child.id()}
    pub fn original_handle(&self)->std::os::windows::io::BorrowedHandle<'_>{self.child.handle()}
    pub fn wait(&self)->io::Result<ExitStatus> {self.child.wait_status()}
    pub fn try_wait(&self)->io::Result<Option<ExitStatus>> {self.child.try_wait_status()}
    pub fn wait_timeout(&self,budget:Duration)->io::Result<Option<u32>> {self.child.wait(budget)}
    /// One attempt only, including connection/send/response uncertainty.
    /// A response is not exit evidence: callers must observe the original child.
    pub fn request_exit_once(&self)->anyhow::Result<()> {
        if self.exit_requested.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err() {
            anyhow::bail!("Exit already attempted; observation only, no replay");
        }
        self.control.request_exit()
    }
}
