// Windows raw host-to-guest listener ownership. No detached accept threads.
use std::collections::HashSet;
use std::io;
use std::sync::{Arc, Mutex};
use base::{AsRawDescriptor, Event, EventExt};
use base::named_pipes::{self, BlockingMode, FramingMode, OverlappedWrapper, PipeConnection};
use cros_async::{EventAsync, Executor};
use super::binder_bridge::{pipe_path, process::ServerProcess};

fn failure(message: &str) -> io::Error { io::Error::other(message) }
struct State { active: bool, next: u32, ports: HashSet<u32>, queued: Vec<PendingAccept> }
pub struct HostControl { cid: u64, owner: Arc<ServerProcess>, event: Event, state: Mutex<State> }
impl HostControl {
    pub fn new(cid: u64, owner: Arc<ServerProcess>) -> io::Result<Arc<Self>> {
        Ok(Arc::new(Self { cid, owner, event: Event::new()?, state: Mutex::new(State {
            active: false, next: 49152, ports: HashSet::new(), queued: Vec::new() }) }))
    }
    pub fn event(&self) -> io::Result<Event> { Ok(self.event.try_clone()?) }
    pub fn activate(self: &Arc<Self>) -> io::Result<ActiveControl> {
        let mut state = self.state.lock().map_err(|_| failure("poisoned host control"))?;
        if state.active { return Err(failure("host control already active")); }
        self.owner.check_live()?;
        state.active = true;
        Ok(ActiveControl(self.clone()))
    }
    /// Success means a listener exists, not that the guest has accepted its REQUEST.
    pub fn connect(self: &Arc<Self>, port: u32) -> io::Result<()> {
        if port < 1024 || port == u32::MAX { return Err(failure("invalid guest service port")); }
        self.owner.check_live()?;
        let mut state = self.state.lock().map_err(|_| failure("poisoned host control"))?;
        if !state.active || state.ports.contains(&port) { return Err(failure("inactive or reserved guest port")); }
        let host_port = state.next;
        state.next = host_port.checked_add(1).filter(|v| *v < u32::MAX)
            .ok_or_else(|| failure("host port space exhausted; no reuse"))?;
        let pipe = named_pipes::create_server_pipe(&pipe_path(self.cid, port)?,
            &FramingMode::Byte, &BlockingMode::Wait, 0, 128 * 1024, true)?;
        let overlapped = Box::new(OverlappedWrapper::new(true)?);
        let mut pending = PendingAccept { pipe: Some(pipe), overlapped, pending: false,
            lease: None };
        // An error here has not submitted a pending operation; the pipe is still owned.
        // SAFETY: PendingAccept owns the pinned operation and drains it in Drop.
        pending.pending = unsafe { pending.pipe.as_ref().unwrap()
            .start_accept_overlapped(&mut pending.overlapped)? };
        state.ports.insert(port);
        pending.lease = Some(PortLease { control: self.clone(), guest_port: port, host_port });
        state.queued.push(pending);
        // If signalling fails, remove and drain outside the mutex (PortLease also locks it).
        if let Err(error) = self.event.signal() {
            let pending = state.queued.pop();
            drop(state); drop(pending); return Err(error.into());
        }
        Ok(())
    }
    pub fn take_pending(&self) -> io::Result<Vec<PendingAccept>> {
        let mut state = self.state.lock().map_err(|_| failure("poisoned host control"))?;
        // connect pushes and signals under this same lock. No lost notification window.
        self.event.reset()?;
        Ok(std::mem::take(&mut state.queued))
    }
}
pub struct ActiveControl(Arc<HostControl>);
impl Drop for ActiveControl {
    fn drop(&mut self) {
        let queued = { let mut state = self.0.state.lock().unwrap_or_else(|p| p.into_inner());
            state.active = false; std::mem::take(&mut state.queued) };
        drop(queued);
    }
}
pub struct PortLease { control: Arc<HostControl>, pub guest_port: u32, pub host_port: u32 }
impl PortLease { pub fn check_live(&self) -> io::Result<()> { self.control.owner.check_live() } }
impl Drop for PortLease {
    fn drop(&mut self) { self.control.state.lock().unwrap_or_else(|p| p.into_inner()).ports.remove(&self.guest_port); }
}
pub struct PendingAccept { pipe: Option<PipeConnection>, overlapped: Box<OverlappedWrapper>, pending: bool, lease: Option<PortLease> }
impl PendingAccept {
    pub async fn accept(mut self, ex: &Executor) -> io::Result<(PipeConnection, PortLease)> {
        if self.pending {
            let event = EventAsync::new_without_reset(self.overlapped.get_h_event_ref().unwrap().try_clone()?, ex)
                .map_err(|e| failure(&format!("accept event: {e}")))?;
            event.next_val().await.map_err(|e| failure(&format!("accept wait: {e}")))?;
            let result = self.pipe.as_mut().unwrap().get_overlapped_result(&mut self.overlapped);
            self.pending = false;
            result?;
        }
        let lease = self.lease.as_ref().unwrap();
        lease.control.owner.verify_client(self.pipe.as_ref().unwrap().as_raw_descriptor())?;
        Ok((self.pipe.take().unwrap(), self.lease.take().unwrap()))
    }
}
impl Drop for PendingAccept {
    fn drop(&mut self) { if self.pending { if let Some(pipe) = self.pipe.as_mut() {
        let _ = pipe.cancel_overlapped(&mut self.overlapped);
        let _ = pipe.get_overlapped_result(&mut self.overlapped);
    } } }
}


// Field order is intentional: Drop first drains the exact operation, then the pipe
// clone closes before the last port lease may be released.
struct PendingWrite {
    pipe: PipeConnection,
    overlapped: Box<OverlappedWrapper>,
    data: Vec<u8>,
    pending: bool,
    shutdown: Event,
    _raw_lease: Option<Arc<PortLease>>,
}
impl Drop for PendingWrite {
    fn drop(&mut self) {
        if self.pending {
            let _ = self.pipe.cancel_overlapped(&mut self.overlapped);
            let _ = self.pipe.get_overlapped_result(&mut self.overlapped);
        }
    }
}

/// One owned write; both normal completion and future cancellation preserve all
/// buffers and the original OVERLAPPED until the real OS operation has completed.
pub async fn write_owned(
    pipe: PipeConnection, shutdown: Event, data: Vec<u8>,
    lease: Option<Arc<PortLease>>, ex: &Executor,
) -> io::Result<()> {
    use futures::{FutureExt, pin_mut, select_biased};
    let mut write = PendingWrite { pipe, overlapped: Box::new(OverlappedWrapper::new(true)?),
        data, pending: false, shutdown, _raw_lease: lease };
    let cancelled = EventAsync::new_without_reset(write.shutdown.try_clone()?, ex)
        .map_err(|error| failure(&format!("write cancellation event: {error}")))?;
    if let Some(result) = cancelled.next_val().now_or_never() {
        result.map_err(|error| failure(&format!("write cancellation wait: {error}")))?;
        return Err(io::Error::new(io::ErrorKind::Interrupted, "connection retired"));
    }
    // SAFETY: write owns the heap allocation, pipe, and pinned OVERLAPPED through drain.
    unsafe { write.pipe.write_overlapped(&write.data, &mut write.overlapped) }?;
    write.pending = true;
    let event = write.overlapped.get_h_event_ref()
        .ok_or_else(|| failure("write event absent"))?.try_clone()?;
    let ready = EventAsync::new_without_reset(event, ex)
        .map_err(|error| failure(&format!("write event: {error}")))?;
    select_biased! {
        result = cancelled.next_val().fuse() => {
            result.map_err(|error| failure(&format!("write cancellation wait: {error}")))?;
            return Err(io::Error::new(io::ErrorKind::Interrupted, "connection retired"));
        },
        result = ready.next_val().fuse() => {
            result.map_err(|error| failure(&format!("write wait: {error}")))?;
        },
    }
    let completed = write.pipe.get_overlapped_result(&mut write.overlapped);
    write.pending = false;
    if completed? as usize != write.data.len() {
        return Err(io::Error::new(io::ErrorKind::WriteZero, "partial pipe write"));
    }
    Ok(())
}
