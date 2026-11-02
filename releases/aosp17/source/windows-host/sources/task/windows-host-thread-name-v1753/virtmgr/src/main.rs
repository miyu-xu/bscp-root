// Copyright 2022, The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Android Virtualization Manager

mod aidl;
mod atom;
mod composite;
#[cfg(windows)]
mod windows_launch_files;
#[cfg(windows)]
mod windows_control_endpoint;
#[cfg(windows)]
mod windows_vm_process;
mod crosvm;
mod debug_config;
mod dt_overlay;
mod host_services;
mod payload;
mod secretkeeper;
#[cfg(target_os="android")]
mod selinux;
#[cfg(not(target_os="android"))]
#[cfg_attr(windows, path = "desktop_authority_windows.rs")]
mod desktop_authority;
#[cfg(not(target_os="android"))]
mod host_internal_service;
#[cfg(windows)]
mod windows_partition_path;
mod virtualmachine;
#[cfg(unix)]
mod process_limits;
#[cfg(not(target_os="android"))]
mod host_lifecycle_trace;

use crate::virtualmachine::VirtualizationService;
use anyhow::{bail, Result};
use binder::BinderFeatures;
#[cfg(target_os = "android")]
use binder::ProcessState;
use clap::Parser;
use log::{error, info, LevelFilter};
#[cfg(unix)]
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
#[cfg(unix)]
use nix::unistd::{setpgid, write, Pid, Uid};
#[cfg(unix)]
use rpcbinder::{FileDescriptorTransportMode, RpcServer};
#[cfg(unix)]
use rustutils::inherited_fd::take_fd_ownership;
#[cfg(unix)]
use binder::platform_fd::{AsFd, RawFd};
#[cfg(unix)]
use std::os::unix::raw::{pid_t, uid_t};
// Widths only for the legacy metadata API. These are never Windows identities.
#[cfg(windows)] #[allow(non_camel_case_types)] type pid_t = i32;
#[cfg(windows)] #[allow(non_camel_case_types)] type uid_t = u32;
#[cfg(unix)]
use std::sync::LazyLock;

#[cfg(early)]
const LOG_TAG: &str = "early_virtmgr";
#[cfg(not(early))]
const LOG_TAG: &str = "virtmgr";

#[cfg(target_os="android")]
static PID_PARENT: LazyLock<Pid> = LazyLock::new(Pid::parent);
#[cfg(target_os="android")]
static UID_CURRENT: LazyLock<Uid> = LazyLock::new(Uid::current);

#[cfg(target_os="android")]
fn get_calling_pid() -> binder::Result<pid_t> {
    // The caller is the parent of this process.
    Ok(PID_PARENT.as_raw())
}
#[cfg(all(unix, not(target_os="android")))]
fn get_calling_pid() -> binder::Result<pid_t> { desktop_authority::current().and_then(|a|a.request_identity()).map(|p|p.pid).map_err(|e|binder::Status::new_exception_str(binder::ExceptionCode::SECURITY,Some(e.to_string()))) }

#[cfg(target_os="android")]
fn get_calling_uid() -> binder::Result<uid_t> {
    // The caller and this process share the same UID.
    Ok(UID_CURRENT.as_raw())
}
#[cfg(all(unix, not(target_os="android")))]
fn get_calling_uid() -> binder::Result<uid_t> { desktop_authority::current().and_then(|a|a.request_identity()).map(|p|p.uid).map_err(|e|binder::Status::new_exception_str(binder::ExceptionCode::SECURITY,Some(e.to_string()))) }

#[cfg(windows)]
fn get_calling_pid() -> binder::Result<pid_t> {
    desktop_host::request_identity::current_pid()
        .and_then(|pid| i32::try_from(pid).map_err(|_| std::io::Error::new(
            std::io::ErrorKind::InvalidData, "request PID exceeds metadata range")))
        .map_err(|e| binder::Status::new_exception_str(binder::ExceptionCode::SECURITY,
            Some(e.to_string())))
}
#[cfg(windows)]
fn get_calling_uid() -> binder::Result<uid_t> {
    // Windows authorization uses the real request SID/token, never a synthetic Unix UID.
    Err(binder::Status::new_exception_str(binder::ExceptionCode::UNSUPPORTED_OPERATION,
        Some("Windows request identity has no Unix UID")))
}

#[cfg(unix)]
#[derive(Parser)]
struct Args {
    /// File descriptor inherited from the caller to run RpcBinder server on.
    /// This should be one end of a socketpair() compatible with RpcBinder's
    /// UDS bootstrap transport.
    #[clap(long)]
    #[cfg_attr(target_os="android",clap(required=true))]
    rpc_server_fd: Option<RawFd>,
    /// Private desktop instance socket. Each RPC session is authenticated independently.
    #[cfg(not(target_os="android"))]
    #[clap(long,conflicts_with="rpc_server_fd",required_unless_present="rpc_server_fd")]
    rpc_server_path: Option<std::path::PathBuf>,
    /// File descriptor inherited from the caller to signal RpcBinder server
    /// readiness. This should be one end of pipe() and the caller should be
    /// waiting for HUP on the other end.
    #[clap(long)]
    ready_fd: RawFd,
}

fn check_vm_support() -> Result<()> {
    if hypervisor_props::is_any_vm_supported()? {
        Ok(())
    } else {
        // This should never happen, it indicates a misconfigured device where the virt APEX
        // is present but VMs are not supported. If it does happen, fail fast to avoid wasting
        // resources trying.
        bail!("Device doesn't support protected or non-protected VMs")
    }
}

#[cfg(unix)]
fn main() {
    // SAFETY: This is very early in the process. Nobody has taken ownership of the inherited FDs
    // yet.
    unsafe { rustutils::inherited_fd::init_once() }
        .expect("Failed to take ownership of inherited FDs");
    #[cfg(not(target_os="android"))]
    host_lifecycle_trace::initialize().expect("Cannot initialize private lifecycle evidence");

    android_logger::init_once(
        android_logger::Config::default()
            .with_tag(LOG_TAG)
            .with_max_level(LevelFilter::Info)
            .with_log_buffer(android_logger::LogId::System),
    );

    let pid_zero = Pid::from_raw(0);
    setpgid(pid_zero, pid_zero).expect("Failed to become a process group leader");

    check_vm_support().unwrap();

    let args = Args::parse();

    let rpc_server_fd = args.rpc_server_fd.map(take_fd_ownership).transpose().expect("Failed to take ownership of rpc_server_fd");
    let ready_fd = take_fd_ownership(args.ready_fd).expect("Failed to take ownership of ready_fd");

    #[cfg(not(target_os="android"))]
    if args.rpc_server_path.is_some() {
        desktop_authority::initialize_listener().expect("Invalid desktop listener owner");
    } else {
        desktop_authority::initialize(rpc_server_fd.as_ref().expect("bootstrap fd required").as_fd(), Pid::parent().as_raw()).expect("Unauthenticated desktop bootstrap");
    }

    // Start thread pool for kernel Binder connection to VirtualizationServiceInternal.
    #[cfg(target_os = "android")]
    ProcessState::start_thread_pool();

    // Android privileged/protected-VM accounting requires an unlimited memlock.
    // Desktop unprotected KVM keeps inherited limits; it does not request --lock-guest-memory.
    #[cfg(target_os="android")]
    if cfg!(early) {
        // we can't access VirtualizationServiceInternal, so directly call rlimit
        #[cfg(target_os = "android")]
        {
        let pid = std::process::id() as i32;
        let lim = libc::rlimit { rlim_cur: libc::RLIM_INFINITY, rlim_max: libc::RLIM_INFINITY };

        // SAFETY: borrowing the new limit struct only. prlimit doesn't use lim outside its lifetime
        let ret = unsafe { libc::prlimit(pid, libc::RLIMIT_MEMLOCK, &lim, std::ptr::null_mut()) };
        if ret == -1 {
            panic!("rlimit error: {}", std::io::Error::last_os_error());
        } else if ret != 0 {
            panic!("Unexpected return value from prlimit(): {ret}");
        }
        }
        #[cfg(not(target_os = "android"))]
        process_limits::remove_memlock_limit().expect("Failed to remove memlock rlimit");
    } else {
        virtualmachine::global_service()
            .unwrap()
            .removeMemlockRlimit()
            .expect("Failed to remove memlock rlimit");
    }

    let service = VirtualizationService::init();
    let service = aidl::BnVirtualizationService::new_binder(service, BinderFeatures::default());
    #[cfg(target_os="android")]
    let server = RpcServer::new_unix_domain_bootstrap(service.as_binder(), rpc_server_fd.expect("bootstrap fd required")).expect("Failed to start RpcServer");
    #[cfg(not(target_os="android"))]
    let server = if let Some(path)=args.rpc_server_path.as_ref() {
        let endpoint=desktop_listener::bind(path).expect("Invalid private desktop socket");
        let server=RpcServer::new_bound_socket(service.as_binder(),endpoint).expect("Failed to bind desktop RpcServer");
        #[cfg(any(target_os="macos",target_os="linux"))]
        server.enable_desktop_authentication().expect("Failed to enable peer authentication");
        #[cfg(not(any(target_os="macos",target_os="linux")))]
        panic!("Authenticated persistent RPC is not yet supported on this platform");
        server
    } else {
        RpcServer::new_unix_domain_bootstrap(service.as_binder(),rpc_server_fd.expect("bootstrap fd required")).expect("Failed to start bootstrap RpcServer")
    };
    server.set_supported_file_descriptor_transport_modes(&[FileDescriptorTransportMode::Unix]);

    let server = std::sync::Arc::new(server);

    // Give a weak reference of the server to the service so that it can shut down the server.
    use binder::binder_impl::Binder;
    let binder: Binder<aidl::BnVirtualizationService> =
        service.as_binder().clone().try_into().unwrap();
    let service_impl: &VirtualizationService = binder.downcast_binder().unwrap();
    service_impl.set_server(std::sync::Arc::downgrade(&server));

    info!("Started VirtualizationService RpcServer. Ready to accept connections");

    // Write the exact readiness token only after the authenticated server is configured.
    if let Err(e) = write(ready_fd.as_fd(), "o".as_bytes()) {
        error!(
            "Failed to signal to the VM client that virtmgr is ready. Is the client dead?: {:#}",
            e
        );
        std::process::exit(1);
    }
    drop(ready_fd);

    #[cfg(target_os = "android")]
    {
    let sa_flags = SaFlags::empty();
    let sa_mask = SigSet::empty();
    let sa = SigAction::new(SigHandler::SigIgn, sa_flags, sa_mask);
    // Ignore SIGTERM in virtmgr and crosvm to allow the VM to shutdown gracefully.
    // As part of the shutdown sequence, processes receive SIGTERM, which leads to
    // Virtualizationservice requesting a shutdown of the VM. However, virtmgr and crosvm also
    // receive SIGTERM which then leads to those processes dying.
    // SAFETY: we don't do anything in the handler
    unsafe { sigaction(Signal::SIGTERM, &sa) }.expect("failed to set sigaction for SIGTERM");
    }

    server.join();
    info!("Shutting down VirtualizationService RpcServer");

    // Do all the standard cleanup we can, mainly to make sure we join logging threads.
    service_impl.get_vms().iter().for_each(|vm| {
        if let Err(e) = vm.kill() {
            error!("VM (cid: {}) did not die when I tried to kill it: {:#}", vm.cid, e);
        }
    });

    info!("All VMs halted.");
}

#[cfg(target_os = "macos")]
mod platform_pipe;

mod vsock_transport;

#[cfg(not(target_os = "android"))]
mod desktop_apex;

#[cfg(not(target_os="android"))]
#[cfg_attr(windows, path = "desktop_lease_windows.rs")]
mod desktop_lease;

#[cfg(not(target_os = "android"))]
mod bridge;
#[cfg(windows)]
mod windows_vsock_io;
#[cfg(windows)]
mod windows_entropy;
#[cfg(windows)]
mod windows_pipe;

#[cfg(all(unix,not(target_os="android")))]
mod desktop_listener;

#[cfg(target_os="macos")]
mod guest_rpc_macos;


#[cfg(unix)]
type HostRpcServer = RpcServer;
#[cfg(windows)]
type HostRpcServer = windows_ndr_server::WindowsNdrServer;
#[cfg(windows)]
mod windows_ndr_server;
#[cfg(windows)]
mod windows_ready;

#[cfg(windows)]
#[derive(Parser)]
struct WindowsArgs {
    /// Local RPC endpoint. Its name and READY byte are not peer credentials.
    #[clap(long)]
    ndr_endpoint: String,
    /// Dedicated inherited byte-pipe writer, passed by the explicit HANDLE list.
    #[clap(long)]
    ready_handle: usize,
}

#[cfg(windows)]
fn main() {
    // --help and malformed arguments have no service/resource side effects.
    let args = WindowsArgs::parse();
    if let Err(error) = run_windows(args) {
        eprintln!("virtmgr Windows service failed: {error:#}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn run_windows(args: WindowsArgs) -> Result<()> {
    use binder::binder_impl::Binder;
    use std::{io::Write,sync::Arc,time::Duration};
    // Exactly one explicitly inherited writer is adopted; no native CRT scan.
    // The launcher owns the original reader and must separately authenticate
    // the actual RPC peer against its original launched process object.
    let mut ready = unsafe { windows_ready::take_inherited(args.ready_handle) }?;
    host_lifecycle_trace::initialize()?;
    desktop_authority::initialize_listener()?;
    check_vm_support()?;
    let service = aidl::BnVirtualizationService::new_binder(
        VirtualizationService::init(), BinderFeatures::default());
    let server = Arc::new(HostRpcServer::new_listener(service.as_binder(),&args.ndr_endpoint)?);
    let local: Binder<aidl::BnVirtualizationService> = service.as_binder().clone()
        .try_into().map_err(|_|anyhow::anyhow!("local service root conversion failed"))?;
    let implementation: &VirtualizationService = local.downcast_binder()
        .ok_or_else(||anyhow::anyhow!("local service implementation unavailable"))?;
    implementation.set_server(Arc::downgrade(&server));
    let startup = (|| -> Result<()> {
        server.start(16)?;
        let observation=server.wait_ready(Duration::from_secs(5))?;
        anyhow::ensure!(observation.ready(),"join-loop not ready: {observation:?}");
        // This is lifecycle status only, never an authenticated association.
        ready.write_all(b"o")?;
        Ok(())
    })();
    drop(ready); // Real EOF after the only local writer closes.
    if startup.is_err() { server.shutdown()?; }
    let completion=server.drive_until_joined();
    // A native observation error keeps the exact owner in its retention slot.
    // No guessed process termination or fabricated resource-reaped result.
    let mut vm_errors=Vec::new();
    for vm in implementation.get_vms() {
        if let Err(error)=vm.kill(){vm_errors.push(format!("cid {}: {error:#}",vm.cid));}
    }
    match completion {
        Ok(observation) => {
            server.release_joined()?;
            startup?;
            anyhow::ensure!(observation.loop_status==0,"NDR loop failed after resource join: {observation:?}");
            anyhow::ensure!(vm_errors.is_empty(),"VM stop failures: {}",vm_errors.join("; "));
            // Native thread join and process stop are not complete disk/bridge
            // reclamation evidence; external acceptance checks those separately.
            Ok(())
        }
        Err(error) => Err(anyhow::anyhow!("NDR owner retained: {error}; startup={startup:?}; VM stop errors={vm_errors:?}")),
    }
}

#[cfg(windows)]
mod guest_rpc_windows;
