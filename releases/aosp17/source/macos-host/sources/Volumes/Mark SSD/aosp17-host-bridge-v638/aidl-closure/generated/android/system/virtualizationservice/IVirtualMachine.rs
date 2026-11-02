/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#[allow(unused_imports)] use binder::binder_impl::IBinderInternal;
use binder::declare_binder_interface;
declare_binder_interface! {
  IVirtualMachine["android.system.virtualizationservice.IVirtualMachine"] {
    native: BnVirtualMachine(on_transact),
    proxy: BpVirtualMachine {
    },
    async: IVirtualMachineAsync(try_into_local_async),
  }
}
pub trait IVirtualMachine: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice.IVirtualMachine" }
  fn r#getCid(&self) -> binder::Result<i32>;
  fn r#getState(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState>;
  fn r#registerCallback(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::Result<()>;
  fn r#start(&self) -> binder::Result<()>;
  fn r#stop(&self) -> binder::Result<()>;
  fn r#isMemoryBalloonEnabled(&self) -> binder::Result<bool>;
  fn r#getActualMemoryBalloonBytes(&self) -> binder::Result<i64>;
  fn r#setMemoryBalloon(&self, _arg_num_bytes: i64) -> binder::Result<()>;
  fn r#connectVsock(&self, _arg_port: i32) -> binder::Result<binder::ParcelFileDescriptor>;
  fn r#createAccessorBinder(&self, _arg_instance: &str, _arg_port: i32) -> binder::Result<binder::SpIBinder>;
  fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()>;
  fn r#suspend(&self) -> binder::Result<()>;
  fn r#resume(&self) -> binder::Result<()>;
  fn r#getDebugInfo(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>;
  fn r#addMemoryToGuest(&self, _arg_fd: &binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::Result<i32>;
  fn r#removeMemoryFromGuest(&self, _arg_memory_id: i32) -> binder::Result<()>;
  fn r#getGuestAgent(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>>;
  fn r#addDisplay(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::Result<()>;
  fn r#removeDisplay(&self, _arg_displayId: i32) -> binder::Result<()>;
  fn r#getDisplays(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>>;
  fn getDefaultImpl() -> IVirtualMachineDefaultRef where Self: Sized {
    DEFAULT_IMPL.lock().unwrap().clone()
  }
  fn setDefaultImpl(d: IVirtualMachineDefaultRef) -> IVirtualMachineDefaultRef where Self: Sized {
    std::mem::replace(&mut *DEFAULT_IMPL.lock().unwrap(), d)
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVirtualMachineAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVirtualMachineAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice.IVirtualMachine" }
  fn r#getCid<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>>;
  fn r#getState<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState>>;
  fn r#registerCallback<'a>(&'a self, _arg_callback: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#start<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#stop<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#isMemoryBalloonEnabled<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>>;
  fn r#getActualMemoryBalloonBytes<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i64>>;
  fn r#setMemoryBalloon<'a>(&'a self, _arg_num_bytes: i64) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#connectVsock<'a>(&'a self, _arg_port: i32) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>>;
  fn r#createAccessorBinder<'a>(&'a self, _arg_instance: &'a str, _arg_port: i32) -> binder::BoxFuture<'a, binder::Result<binder::SpIBinder>>;
  fn r#setHostConsoleName<'a>(&'a self, _arg_pathname: &'a str) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#suspend<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#resume<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#getDebugInfo<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>;
  fn r#addMemoryToGuest<'a>(&'a self, _arg_fd: &'a binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::BoxFuture<'a, binder::Result<i32>>;
  fn r#removeMemoryFromGuest<'a>(&'a self, _arg_memory_id: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#getGuestAgent<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>>>;
  fn r#addDisplay<'a>(&'a self, _arg_config: &'a crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#removeDisplay<'a>(&'a self, _arg_displayId: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#getDisplays<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>>>;
}
#[::async_trait::async_trait]
pub trait IVirtualMachineAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice.IVirtualMachine" }
  async fn r#getCid(&self) -> binder::Result<i32>;
  async fn r#getState(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState>;
  async fn r#registerCallback(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::Result<()>;
  async fn r#start(&self) -> binder::Result<()>;
  async fn r#stop(&self) -> binder::Result<()>;
  async fn r#isMemoryBalloonEnabled(&self) -> binder::Result<bool>;
  async fn r#getActualMemoryBalloonBytes(&self) -> binder::Result<i64>;
  async fn r#setMemoryBalloon(&self, _arg_num_bytes: i64) -> binder::Result<()>;
  async fn r#connectVsock(&self, _arg_port: i32) -> binder::Result<binder::ParcelFileDescriptor>;
  async fn r#createAccessorBinder(&self, _arg_instance: &str, _arg_port: i32) -> binder::Result<binder::SpIBinder>;
  async fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()>;
  async fn r#suspend(&self) -> binder::Result<()>;
  async fn r#resume(&self) -> binder::Result<()>;
  async fn r#getDebugInfo(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>;
  async fn r#addMemoryToGuest(&self, _arg_fd: &binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::Result<i32>;
  async fn r#removeMemoryFromGuest(&self, _arg_memory_id: i32) -> binder::Result<()>;
  async fn r#getGuestAgent(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>>;
  async fn r#addDisplay(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::Result<()>;
  async fn r#removeDisplay(&self, _arg_displayId: i32) -> binder::Result<()>;
  async fn r#getDisplays(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>>;
}
impl BnVirtualMachine {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVirtualMachine>
  where
    T: IVirtualMachineAsyncServer + binder::Interface + Send + Sync + 'static,
    R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
  {
    struct Wrapper<T, R> {
      _inner: T,
      _rt: R,
    }
    impl<T, R> binder::Interface for Wrapper<T, R> where T: binder::Interface, R: Send + Sync + 'static {
      fn as_binder(&self) -> binder::SpIBinder { self._inner.as_binder() }
      fn dump(&self, _writer: &mut dyn std::io::Write, _args: &[&std::ffi::CStr]) -> std::result::Result<(), binder::StatusCode> { self._inner.dump(_writer, _args) }
    }
    impl<T, R> IVirtualMachine for Wrapper<T, R>
    where
      T: IVirtualMachineAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#getCid(&self) -> binder::Result<i32> {
        self._rt.block_on(self._inner.r#getCid())
      }
      fn r#getState(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState> {
        self._rt.block_on(self._inner.r#getState())
      }
      fn r#registerCallback(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#registerCallback(_arg_callback))
      }
      fn r#start(&self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#start())
      }
      fn r#stop(&self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#stop())
      }
      fn r#isMemoryBalloonEnabled(&self) -> binder::Result<bool> {
        self._rt.block_on(self._inner.r#isMemoryBalloonEnabled())
      }
      fn r#getActualMemoryBalloonBytes(&self) -> binder::Result<i64> {
        self._rt.block_on(self._inner.r#getActualMemoryBalloonBytes())
      }
      fn r#setMemoryBalloon(&self, _arg_num_bytes: i64) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#setMemoryBalloon(_arg_num_bytes))
      }
      fn r#connectVsock(&self, _arg_port: i32) -> binder::Result<binder::ParcelFileDescriptor> {
        self._rt.block_on(self._inner.r#connectVsock(_arg_port))
      }
      fn r#createAccessorBinder(&self, _arg_instance: &str, _arg_port: i32) -> binder::Result<binder::SpIBinder> {
        self._rt.block_on(self._inner.r#createAccessorBinder(_arg_instance, _arg_port))
      }
      fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#setHostConsoleName(_arg_pathname))
      }
      fn r#suspend(&self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#suspend())
      }
      fn r#resume(&self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#resume())
      }
      fn r#getDebugInfo(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo> {
        self._rt.block_on(self._inner.r#getDebugInfo())
      }
      fn r#addMemoryToGuest(&self, _arg_fd: &binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::Result<i32> {
        self._rt.block_on(self._inner.r#addMemoryToGuest(_arg_fd, _arg_offset, _arg_rangeStart, _arg_rangeEnd, _arg_cacheable))
      }
      fn r#removeMemoryFromGuest(&self, _arg_memory_id: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#removeMemoryFromGuest(_arg_memory_id))
      }
      fn r#getGuestAgent(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>> {
        self._rt.block_on(self._inner.r#getGuestAgent())
      }
      fn r#addDisplay(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#addDisplay(_arg_config))
      }
      fn r#removeDisplay(&self, _arg_displayId: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#removeDisplay(_arg_displayId))
      }
      fn r#getDisplays(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>> {
        self._rt.block_on(self._inner.r#getDisplays())
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVirtualMachineAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVirtualMachineAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVirtualMachine>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVirtualMachineAsync<P> for Wrapper {
      fn r#getCid<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getCid())
      }
      fn r#getState<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getState())
      }
      fn r#registerCallback<'a>(&'a self, _arg_callback: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#registerCallback(_arg_callback))
      }
      fn r#start<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#start())
      }
      fn r#stop<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#stop())
      }
      fn r#isMemoryBalloonEnabled<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#isMemoryBalloonEnabled())
      }
      fn r#getActualMemoryBalloonBytes<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i64>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getActualMemoryBalloonBytes())
      }
      fn r#setMemoryBalloon<'a>(&'a self, _arg_num_bytes: i64) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#setMemoryBalloon(_arg_num_bytes))
      }
      fn r#connectVsock<'a>(&'a self, _arg_port: i32) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#connectVsock(_arg_port))
      }
      fn r#createAccessorBinder<'a>(&'a self, _arg_instance: &'a str, _arg_port: i32) -> binder::BoxFuture<'a, binder::Result<binder::SpIBinder>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#createAccessorBinder(_arg_instance, _arg_port))
      }
      fn r#setHostConsoleName<'a>(&'a self, _arg_pathname: &'a str) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#setHostConsoleName(_arg_pathname))
      }
      fn r#suspend<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#suspend())
      }
      fn r#resume<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#resume())
      }
      fn r#getDebugInfo<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getDebugInfo())
      }
      fn r#addMemoryToGuest<'a>(&'a self, _arg_fd: &'a binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::BoxFuture<'a, binder::Result<i32>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#addMemoryToGuest(_arg_fd, _arg_offset, _arg_rangeStart, _arg_rangeEnd, _arg_cacheable))
      }
      fn r#removeMemoryFromGuest<'a>(&'a self, _arg_memory_id: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#removeMemoryFromGuest(_arg_memory_id))
      }
      fn r#getGuestAgent<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getGuestAgent())
      }
      fn r#addDisplay<'a>(&'a self, _arg_config: &'a crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#addDisplay(_arg_config))
      }
      fn r#removeDisplay<'a>(&'a self, _arg_displayId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#removeDisplay(_arg_displayId))
      }
      fn r#getDisplays<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getDisplays())
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVirtualMachineAsync<P>>))
    } else {
      None
    }
  }
}
pub trait IVirtualMachineDefault: Send + Sync {
  fn r#getCid(&self) -> binder::Result<i32> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getState(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#registerCallback(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#start(&self) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#stop(&self) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#isMemoryBalloonEnabled(&self) -> binder::Result<bool> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getActualMemoryBalloonBytes(&self) -> binder::Result<i64> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#setMemoryBalloon(&self, _arg_num_bytes: i64) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#connectVsock(&self, _arg_port: i32) -> binder::Result<binder::ParcelFileDescriptor> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#createAccessorBinder(&self, _arg_instance: &str, _arg_port: i32) -> binder::Result<binder::SpIBinder> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#suspend(&self) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#resume(&self) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getDebugInfo(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#addMemoryToGuest(&self, _arg_fd: &binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::Result<i32> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#removeMemoryFromGuest(&self, _arg_memory_id: i32) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getGuestAgent(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#addDisplay(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#removeDisplay(&self, _arg_displayId: i32) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getDisplays(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
}
pub mod transactions {
  pub const r#getCid: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#getState: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#registerCallback: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
  pub const r#start: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 3;
  pub const r#stop: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 4;
  pub const r#isMemoryBalloonEnabled: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 5;
  pub const r#getActualMemoryBalloonBytes: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 6;
  pub const r#setMemoryBalloon: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 7;
  pub const r#connectVsock: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 8;
  pub const r#createAccessorBinder: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 9;
  pub const r#setHostConsoleName: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 10;
  pub const r#suspend: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 11;
  pub const r#resume: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 12;
  pub const r#getDebugInfo: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 13;
  pub const r#addMemoryToGuest: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 14;
  pub const r#removeMemoryFromGuest: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 15;
  pub const r#getGuestAgent: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 16;
  pub const r#addDisplay: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 17;
  pub const r#removeDisplay: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 18;
  pub const r#getDisplays: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 19;
}
pub type IVirtualMachineDefaultRef = Option<std::sync::Arc<dyn IVirtualMachineDefault>>;
static DEFAULT_IMPL: std::sync::Mutex<IVirtualMachineDefaultRef> = std::sync::Mutex::new(None);
pub const r#ERROR_UNEXPECTED: i32 = -1;
impl BpVirtualMachine {
  fn build_parcel_getCid(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getCid(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<i32> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#getCid();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: i32 = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getState(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getState(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#getState();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_registerCallback(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_callback)?;
    Ok(aidl_data)
  }
  fn read_response_registerCallback(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#registerCallback(_arg_callback);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_start(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_start(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#start();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_stop(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_stop(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#stop();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_isMemoryBalloonEnabled(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_isMemoryBalloonEnabled(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<bool> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#isMemoryBalloonEnabled();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: bool = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getActualMemoryBalloonBytes(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getActualMemoryBalloonBytes(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<i64> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#getActualMemoryBalloonBytes();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: i64 = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_setMemoryBalloon(&self, _arg_num_bytes: i64) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_num_bytes)?;
    Ok(aidl_data)
  }
  fn read_response_setMemoryBalloon(&self, _arg_num_bytes: i64, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#setMemoryBalloon(_arg_num_bytes);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_connectVsock(&self, _arg_port: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_port)?;
    Ok(aidl_data)
  }
  fn read_response_connectVsock(&self, _arg_port: i32, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::ParcelFileDescriptor> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#connectVsock(_arg_port);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::ParcelFileDescriptor = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_createAccessorBinder(&self, _arg_instance: &str, _arg_port: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_instance)?;
    aidl_data.write(&_arg_port)?;
    Ok(aidl_data)
  }
  fn read_response_createAccessorBinder(&self, _arg_instance: &str, _arg_port: i32, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::SpIBinder> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#createAccessorBinder(_arg_instance, _arg_port);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::SpIBinder = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_pathname)?;
    Ok(aidl_data)
  }
  fn read_response_setHostConsoleName(&self, _arg_pathname: &str, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#setHostConsoleName(_arg_pathname);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_suspend(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_suspend(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#suspend();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_resume(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_resume(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#resume();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_getDebugInfo(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getDebugInfo(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#getDebugInfo();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_addMemoryToGuest(&self, _arg_fd: &binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_fd)?;
    aidl_data.write(&_arg_offset)?;
    aidl_data.write(&_arg_rangeStart)?;
    aidl_data.write(&_arg_rangeEnd)?;
    aidl_data.write(&_arg_cacheable)?;
    Ok(aidl_data)
  }
  fn read_response_addMemoryToGuest(&self, _arg_fd: &binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<i32> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#addMemoryToGuest(_arg_fd, _arg_offset, _arg_rangeStart, _arg_rangeEnd, _arg_cacheable);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: i32 = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_removeMemoryFromGuest(&self, _arg_memory_id: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_memory_id)?;
    Ok(aidl_data)
  }
  fn read_response_removeMemoryFromGuest(&self, _arg_memory_id: i32, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#removeMemoryFromGuest(_arg_memory_id);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_getGuestAgent(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getGuestAgent(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#getGuestAgent();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_addDisplay(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_config)?;
    Ok(aidl_data)
  }
  fn read_response_addDisplay(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#addDisplay(_arg_config);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_removeDisplay(&self, _arg_displayId: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_displayId)?;
    Ok(aidl_data)
  }
  fn read_response_removeDisplay(&self, _arg_displayId: i32, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#removeDisplay(_arg_displayId);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_getDisplays(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getDisplays(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachine>::getDefaultImpl() {
        return _aidl_default_impl.r#getDisplays();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
}
impl IVirtualMachine for BpVirtualMachine {
  fn r#getCid(&self) -> binder::Result<i32> {
    let _aidl_data = self.build_parcel_getCid()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getCid, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getCid(_aidl_reply)
  }
  fn r#getState(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState> {
    let _aidl_data = self.build_parcel_getState()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getState, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getState(_aidl_reply)
  }
  fn r#registerCallback(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_registerCallback(_arg_callback)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#registerCallback, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_registerCallback(_arg_callback, _aidl_reply)
  }
  fn r#start(&self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_start()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#start, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_start(_aidl_reply)
  }
  fn r#stop(&self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_stop()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#stop, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_stop(_aidl_reply)
  }
  fn r#isMemoryBalloonEnabled(&self) -> binder::Result<bool> {
    let _aidl_data = self.build_parcel_isMemoryBalloonEnabled()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#isMemoryBalloonEnabled, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_isMemoryBalloonEnabled(_aidl_reply)
  }
  fn r#getActualMemoryBalloonBytes(&self) -> binder::Result<i64> {
    let _aidl_data = self.build_parcel_getActualMemoryBalloonBytes()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getActualMemoryBalloonBytes, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getActualMemoryBalloonBytes(_aidl_reply)
  }
  fn r#setMemoryBalloon(&self, _arg_num_bytes: i64) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_setMemoryBalloon(_arg_num_bytes)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#setMemoryBalloon, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_setMemoryBalloon(_arg_num_bytes, _aidl_reply)
  }
  fn r#connectVsock(&self, _arg_port: i32) -> binder::Result<binder::ParcelFileDescriptor> {
    let _aidl_data = self.build_parcel_connectVsock(_arg_port)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#connectVsock, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_connectVsock(_arg_port, _aidl_reply)
  }
  fn r#createAccessorBinder(&self, _arg_instance: &str, _arg_port: i32) -> binder::Result<binder::SpIBinder> {
    let _aidl_data = self.build_parcel_createAccessorBinder(_arg_instance, _arg_port)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#createAccessorBinder, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_createAccessorBinder(_arg_instance, _arg_port, _aidl_reply)
  }
  fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_setHostConsoleName(_arg_pathname)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#setHostConsoleName, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_setHostConsoleName(_arg_pathname, _aidl_reply)
  }
  fn r#suspend(&self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_suspend()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#suspend, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_suspend(_aidl_reply)
  }
  fn r#resume(&self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_resume()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#resume, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_resume(_aidl_reply)
  }
  fn r#getDebugInfo(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo> {
    let _aidl_data = self.build_parcel_getDebugInfo()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getDebugInfo, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getDebugInfo(_aidl_reply)
  }
  fn r#addMemoryToGuest(&self, _arg_fd: &binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::Result<i32> {
    let _aidl_data = self.build_parcel_addMemoryToGuest(_arg_fd, _arg_offset, _arg_rangeStart, _arg_rangeEnd, _arg_cacheable)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#addMemoryToGuest, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_addMemoryToGuest(_arg_fd, _arg_offset, _arg_rangeStart, _arg_rangeEnd, _arg_cacheable, _aidl_reply)
  }
  fn r#removeMemoryFromGuest(&self, _arg_memory_id: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_removeMemoryFromGuest(_arg_memory_id)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#removeMemoryFromGuest, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_removeMemoryFromGuest(_arg_memory_id, _aidl_reply)
  }
  fn r#getGuestAgent(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>> {
    let _aidl_data = self.build_parcel_getGuestAgent()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getGuestAgent, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getGuestAgent(_aidl_reply)
  }
  fn r#addDisplay(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_addDisplay(_arg_config)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#addDisplay, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_addDisplay(_arg_config, _aidl_reply)
  }
  fn r#removeDisplay(&self, _arg_displayId: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_removeDisplay(_arg_displayId)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#removeDisplay, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_removeDisplay(_arg_displayId, _aidl_reply)
  }
  fn r#getDisplays(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>> {
    let _aidl_data = self.build_parcel_getDisplays()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getDisplays, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getDisplays(_aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVirtualMachineAsync<P> for BpVirtualMachine {
  fn r#getCid<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>> {
    let _aidl_data = match self.build_parcel_getCid() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getCid, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getCid(_aidl_reply)
      }
    )
  }
  fn r#getState<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState>> {
    let _aidl_data = match self.build_parcel_getState() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getState, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getState(_aidl_reply)
      }
    )
  }
  fn r#registerCallback<'a>(&'a self, _arg_callback: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_registerCallback(_arg_callback) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#registerCallback, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_registerCallback(_arg_callback, _aidl_reply)
      }
    )
  }
  fn r#start<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_start() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#start, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_start(_aidl_reply)
      }
    )
  }
  fn r#stop<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_stop() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#stop, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_stop(_aidl_reply)
      }
    )
  }
  fn r#isMemoryBalloonEnabled<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>> {
    let _aidl_data = match self.build_parcel_isMemoryBalloonEnabled() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#isMemoryBalloonEnabled, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_isMemoryBalloonEnabled(_aidl_reply)
      }
    )
  }
  fn r#getActualMemoryBalloonBytes<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i64>> {
    let _aidl_data = match self.build_parcel_getActualMemoryBalloonBytes() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getActualMemoryBalloonBytes, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getActualMemoryBalloonBytes(_aidl_reply)
      }
    )
  }
  fn r#setMemoryBalloon<'a>(&'a self, _arg_num_bytes: i64) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_setMemoryBalloon(_arg_num_bytes) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#setMemoryBalloon, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_setMemoryBalloon(_arg_num_bytes, _aidl_reply)
      }
    )
  }
  fn r#connectVsock<'a>(&'a self, _arg_port: i32) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>> {
    let _aidl_data = match self.build_parcel_connectVsock(_arg_port) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#connectVsock, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_connectVsock(_arg_port, _aidl_reply)
      }
    )
  }
  fn r#createAccessorBinder<'a>(&'a self, _arg_instance: &'a str, _arg_port: i32) -> binder::BoxFuture<'a, binder::Result<binder::SpIBinder>> {
    let _aidl_data = match self.build_parcel_createAccessorBinder(_arg_instance, _arg_port) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#createAccessorBinder, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_createAccessorBinder(_arg_instance, _arg_port, _aidl_reply)
      }
    )
  }
  fn r#setHostConsoleName<'a>(&'a self, _arg_pathname: &'a str) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_setHostConsoleName(_arg_pathname) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#setHostConsoleName, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_setHostConsoleName(_arg_pathname, _aidl_reply)
      }
    )
  }
  fn r#suspend<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_suspend() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#suspend, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_suspend(_aidl_reply)
      }
    )
  }
  fn r#resume<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_resume() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#resume, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_resume(_aidl_reply)
      }
    )
  }
  fn r#getDebugInfo<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> {
    let _aidl_data = match self.build_parcel_getDebugInfo() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getDebugInfo, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getDebugInfo(_aidl_reply)
      }
    )
  }
  fn r#addMemoryToGuest<'a>(&'a self, _arg_fd: &'a binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::BoxFuture<'a, binder::Result<i32>> {
    let _aidl_data = match self.build_parcel_addMemoryToGuest(_arg_fd, _arg_offset, _arg_rangeStart, _arg_rangeEnd, _arg_cacheable) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#addMemoryToGuest, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_addMemoryToGuest(_arg_fd, _arg_offset, _arg_rangeStart, _arg_rangeEnd, _arg_cacheable, _aidl_reply)
      }
    )
  }
  fn r#removeMemoryFromGuest<'a>(&'a self, _arg_memory_id: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_removeMemoryFromGuest(_arg_memory_id) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#removeMemoryFromGuest, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_removeMemoryFromGuest(_arg_memory_id, _aidl_reply)
      }
    )
  }
  fn r#getGuestAgent<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>>> {
    let _aidl_data = match self.build_parcel_getGuestAgent() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getGuestAgent, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getGuestAgent(_aidl_reply)
      }
    )
  }
  fn r#addDisplay<'a>(&'a self, _arg_config: &'a crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_addDisplay(_arg_config) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#addDisplay, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_addDisplay(_arg_config, _aidl_reply)
      }
    )
  }
  fn r#removeDisplay<'a>(&'a self, _arg_displayId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_removeDisplay(_arg_displayId) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#removeDisplay, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_removeDisplay(_arg_displayId, _aidl_reply)
      }
    )
  }
  fn r#getDisplays<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>>> {
    let _aidl_data = match self.build_parcel_getDisplays() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getDisplays, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getDisplays(_aidl_reply)
      }
    )
  }
}
impl IVirtualMachine for binder::binder_impl::Binder<BnVirtualMachine> {
  fn r#getCid(&self) -> binder::Result<i32> { self.0.r#getCid() }
  fn r#getState(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_19_VirtualMachineState> { self.0.r#getState() }
  fn r#registerCallback(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback>) -> binder::Result<()> { self.0.r#registerCallback(_arg_callback) }
  fn r#start(&self) -> binder::Result<()> { self.0.r#start() }
  fn r#stop(&self) -> binder::Result<()> { self.0.r#stop() }
  fn r#isMemoryBalloonEnabled(&self) -> binder::Result<bool> { self.0.r#isMemoryBalloonEnabled() }
  fn r#getActualMemoryBalloonBytes(&self) -> binder::Result<i64> { self.0.r#getActualMemoryBalloonBytes() }
  fn r#setMemoryBalloon(&self, _arg_num_bytes: i64) -> binder::Result<()> { self.0.r#setMemoryBalloon(_arg_num_bytes) }
  fn r#connectVsock(&self, _arg_port: i32) -> binder::Result<binder::ParcelFileDescriptor> { self.0.r#connectVsock(_arg_port) }
  fn r#createAccessorBinder(&self, _arg_instance: &str, _arg_port: i32) -> binder::Result<binder::SpIBinder> { self.0.r#createAccessorBinder(_arg_instance, _arg_port) }
  fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()> { self.0.r#setHostConsoleName(_arg_pathname) }
  fn r#suspend(&self) -> binder::Result<()> { self.0.r#suspend() }
  fn r#resume(&self) -> binder::Result<()> { self.0.r#resume() }
  fn r#getDebugInfo(&self) -> binder::Result<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo> { self.0.r#getDebugInfo() }
  fn r#addMemoryToGuest(&self, _arg_fd: &binder::ParcelFileDescriptor, _arg_offset: i64, _arg_rangeStart: i64, _arg_rangeEnd: i64, _arg_cacheable: bool) -> binder::Result<i32> { self.0.r#addMemoryToGuest(_arg_fd, _arg_offset, _arg_rangeStart, _arg_rangeEnd, _arg_cacheable) }
  fn r#removeMemoryFromGuest(&self, _arg_memory_id: i32) -> binder::Result<()> { self.0.r#removeMemoryFromGuest(_arg_memory_id) }
  fn r#getGuestAgent(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>>> { self.0.r#getGuestAgent() }
  fn r#addDisplay(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig) -> binder::Result<()> { self.0.r#addDisplay(_arg_config) }
  fn r#removeDisplay(&self, _arg_displayId: i32) -> binder::Result<()> { self.0.r#removeDisplay(_arg_displayId) }
  fn r#getDisplays(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_21_VirtualMachineDisplay>> { self.0.r#getDisplays() }
}
fn on_transact(_aidl_service: &dyn IVirtualMachine, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> std::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#getCid => {
      let _aidl_return = _aidl_service.r#getCid();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getState => {
      let _aidl_return = _aidl_service.r#getState();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#registerCallback => {
      let _arg_callback: binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#registerCallback(&_arg_callback);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#start => {
      let _aidl_return = _aidl_service.r#start();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#stop => {
      let _aidl_return = _aidl_service.r#stop();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#isMemoryBalloonEnabled => {
      let _aidl_return = _aidl_service.r#isMemoryBalloonEnabled();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getActualMemoryBalloonBytes => {
      let _aidl_return = _aidl_service.r#getActualMemoryBalloonBytes();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#setMemoryBalloon => {
      let _arg_num_bytes: i64 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#setMemoryBalloon(_arg_num_bytes);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#connectVsock => {
      let _arg_port: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#connectVsock(_arg_port);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#createAccessorBinder => {
      let _arg_instance: String = _aidl_data.read()?;
      let _arg_port: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#createAccessorBinder(&_arg_instance, _arg_port);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#setHostConsoleName => {
      let _arg_pathname: String = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#setHostConsoleName(&_arg_pathname);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#suspend => {
      let _aidl_return = _aidl_service.r#suspend();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#resume => {
      let _aidl_return = _aidl_service.r#resume();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getDebugInfo => {
      let _aidl_return = _aidl_service.r#getDebugInfo();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#addMemoryToGuest => {
      let _arg_fd: binder::ParcelFileDescriptor = _aidl_data.read()?;
      let _arg_offset: i64 = _aidl_data.read()?;
      let _arg_rangeStart: i64 = _aidl_data.read()?;
      let _arg_rangeEnd: i64 = _aidl_data.read()?;
      let _arg_cacheable: bool = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#addMemoryToGuest(&_arg_fd, _arg_offset, _arg_rangeStart, _arg_rangeEnd, _arg_cacheable);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#removeMemoryFromGuest => {
      let _arg_memory_id: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#removeMemoryFromGuest(_arg_memory_id);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getGuestAgent => {
      let _aidl_return = _aidl_service.r#getGuestAgent();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#addDisplay => {
      let _arg_config: crate::mangled::_7_android_6_system_21_virtualizationservice_13_DisplayConfig = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#addDisplay(&_arg_config);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#removeDisplay => {
      let _arg_displayId: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#removeDisplay(_arg_displayId);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getDisplays => {
      let _aidl_return = _aidl_service.r#getDisplays();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    _ => Err(binder::StatusCode::UNKNOWN_TRANSACTION)
  }
}
pub(crate) mod mangled {
 pub use super::r#IVirtualMachine as _7_android_6_system_21_virtualizationservice_15_IVirtualMachine;
}
