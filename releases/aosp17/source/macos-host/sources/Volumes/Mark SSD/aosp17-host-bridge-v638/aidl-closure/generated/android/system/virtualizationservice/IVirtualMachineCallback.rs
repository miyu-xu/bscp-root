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
  IVirtualMachineCallback["android.system.virtualizationservice.IVirtualMachineCallback"] {
    native: BnVirtualMachineCallback(on_transact),
    proxy: BpVirtualMachineCallback {
    },
    async: IVirtualMachineCallbackAsync(try_into_local_async),
  }
}
pub trait IVirtualMachineCallback: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice.IVirtualMachineCallback" }
  fn r#onPayloadStarted(&self, _arg_cid: i32) -> binder::Result<()>;
  fn r#onPayloadReady(&self, _arg_cid: i32) -> binder::Result<()>;
  fn r#onPayloadFinished(&self, _arg_cid: i32, _arg_exitCode: i32) -> binder::Result<()>;
  fn r#onError(&self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()>;
  fn r#onDied(&self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::Result<()>;
  fn r#onGuestAgentRegistered(&self, _arg_cid: i32, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()>;
  fn getDefaultImpl() -> IVirtualMachineCallbackDefaultRef where Self: Sized {
    DEFAULT_IMPL.lock().unwrap().clone()
  }
  fn setDefaultImpl(d: IVirtualMachineCallbackDefaultRef) -> IVirtualMachineCallbackDefaultRef where Self: Sized {
    std::mem::replace(&mut *DEFAULT_IMPL.lock().unwrap(), d)
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVirtualMachineCallbackAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVirtualMachineCallbackAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice.IVirtualMachineCallback" }
  fn r#onPayloadStarted<'a>(&'a self, _arg_cid: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#onPayloadReady<'a>(&'a self, _arg_cid: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#onPayloadFinished<'a>(&'a self, _arg_cid: i32, _arg_exitCode: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#onError<'a>(&'a self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &'a str) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#onDied<'a>(&'a self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#onGuestAgentRegistered<'a>(&'a self, _arg_cid: i32, _arg_guestAgent: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IVirtualMachineCallbackAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice.IVirtualMachineCallback" }
  async fn r#onPayloadStarted(&self, _arg_cid: i32) -> binder::Result<()>;
  async fn r#onPayloadReady(&self, _arg_cid: i32) -> binder::Result<()>;
  async fn r#onPayloadFinished(&self, _arg_cid: i32, _arg_exitCode: i32) -> binder::Result<()>;
  async fn r#onError(&self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()>;
  async fn r#onDied(&self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::Result<()>;
  async fn r#onGuestAgentRegistered(&self, _arg_cid: i32, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()>;
}
impl BnVirtualMachineCallback {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVirtualMachineCallback>
  where
    T: IVirtualMachineCallbackAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IVirtualMachineCallback for Wrapper<T, R>
    where
      T: IVirtualMachineCallbackAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#onPayloadStarted(&self, _arg_cid: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#onPayloadStarted(_arg_cid))
      }
      fn r#onPayloadReady(&self, _arg_cid: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#onPayloadReady(_arg_cid))
      }
      fn r#onPayloadFinished(&self, _arg_cid: i32, _arg_exitCode: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#onPayloadFinished(_arg_cid, _arg_exitCode))
      }
      fn r#onError(&self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#onError(_arg_cid, _arg_errorCode, _arg_message))
      }
      fn r#onDied(&self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#onDied(_arg_cid, _arg_reason))
      }
      fn r#onGuestAgentRegistered(&self, _arg_cid: i32, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#onGuestAgentRegistered(_arg_cid, _arg_guestAgent))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVirtualMachineCallbackAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVirtualMachineCallbackAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVirtualMachineCallback>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVirtualMachineCallbackAsync<P> for Wrapper {
      fn r#onPayloadStarted<'a>(&'a self, _arg_cid: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#onPayloadStarted(_arg_cid))
      }
      fn r#onPayloadReady<'a>(&'a self, _arg_cid: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#onPayloadReady(_arg_cid))
      }
      fn r#onPayloadFinished<'a>(&'a self, _arg_cid: i32, _arg_exitCode: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#onPayloadFinished(_arg_cid, _arg_exitCode))
      }
      fn r#onError<'a>(&'a self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &'a str) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#onError(_arg_cid, _arg_errorCode, _arg_message))
      }
      fn r#onDied<'a>(&'a self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#onDied(_arg_cid, _arg_reason))
      }
      fn r#onGuestAgentRegistered<'a>(&'a self, _arg_cid: i32, _arg_guestAgent: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#onGuestAgentRegistered(_arg_cid, _arg_guestAgent))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVirtualMachineCallbackAsync<P>>))
    } else {
      None
    }
  }
}
pub trait IVirtualMachineCallbackDefault: Send + Sync {
  fn r#onPayloadStarted(&self, _arg_cid: i32) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#onPayloadReady(&self, _arg_cid: i32) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#onPayloadFinished(&self, _arg_cid: i32, _arg_exitCode: i32) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#onError(&self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#onDied(&self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#onGuestAgentRegistered(&self, _arg_cid: i32, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
}
pub mod transactions {
  pub const r#onPayloadStarted: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#onPayloadReady: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#onPayloadFinished: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
  pub const r#onError: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 3;
  pub const r#onDied: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 4;
  pub const r#onGuestAgentRegistered: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 5;
}
pub type IVirtualMachineCallbackDefaultRef = Option<std::sync::Arc<dyn IVirtualMachineCallbackDefault>>;
static DEFAULT_IMPL: std::sync::Mutex<IVirtualMachineCallbackDefaultRef> = std::sync::Mutex::new(None);
impl BpVirtualMachineCallback {
  fn build_parcel_onPayloadStarted(&self, _arg_cid: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_cid)?;
    Ok(aidl_data)
  }
  fn read_response_onPayloadStarted(&self, _arg_cid: i32, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineCallback>::getDefaultImpl() {
        return _aidl_default_impl.r#onPayloadStarted(_arg_cid);
      }
    }
    let _aidl_reply = _aidl_reply?;
    Ok(())
  }
  fn build_parcel_onPayloadReady(&self, _arg_cid: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_cid)?;
    Ok(aidl_data)
  }
  fn read_response_onPayloadReady(&self, _arg_cid: i32, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineCallback>::getDefaultImpl() {
        return _aidl_default_impl.r#onPayloadReady(_arg_cid);
      }
    }
    let _aidl_reply = _aidl_reply?;
    Ok(())
  }
  fn build_parcel_onPayloadFinished(&self, _arg_cid: i32, _arg_exitCode: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_cid)?;
    aidl_data.write(&_arg_exitCode)?;
    Ok(aidl_data)
  }
  fn read_response_onPayloadFinished(&self, _arg_cid: i32, _arg_exitCode: i32, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineCallback>::getDefaultImpl() {
        return _aidl_default_impl.r#onPayloadFinished(_arg_cid, _arg_exitCode);
      }
    }
    let _aidl_reply = _aidl_reply?;
    Ok(())
  }
  fn build_parcel_onError(&self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_cid)?;
    aidl_data.write(&_arg_errorCode)?;
    aidl_data.write(_arg_message)?;
    Ok(aidl_data)
  }
  fn read_response_onError(&self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineCallback>::getDefaultImpl() {
        return _aidl_default_impl.r#onError(_arg_cid, _arg_errorCode, _arg_message);
      }
    }
    let _aidl_reply = _aidl_reply?;
    Ok(())
  }
  fn build_parcel_onDied(&self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_cid)?;
    aidl_data.write(&_arg_reason)?;
    Ok(aidl_data)
  }
  fn read_response_onDied(&self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineCallback>::getDefaultImpl() {
        return _aidl_default_impl.r#onDied(_arg_cid, _arg_reason);
      }
    }
    let _aidl_reply = _aidl_reply?;
    Ok(())
  }
  fn build_parcel_onGuestAgentRegistered(&self, _arg_cid: i32, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_cid)?;
    aidl_data.write(_arg_guestAgent)?;
    Ok(aidl_data)
  }
  fn read_response_onGuestAgentRegistered(&self, _arg_cid: i32, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineCallback>::getDefaultImpl() {
        return _aidl_default_impl.r#onGuestAgentRegistered(_arg_cid, _arg_guestAgent);
      }
    }
    let _aidl_reply = _aidl_reply?;
    Ok(())
  }
}
impl IVirtualMachineCallback for BpVirtualMachineCallback {
  fn r#onPayloadStarted(&self, _arg_cid: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_onPayloadStarted(_arg_cid)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#onPayloadStarted, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_onPayloadStarted(_arg_cid, _aidl_reply)
  }
  fn r#onPayloadReady(&self, _arg_cid: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_onPayloadReady(_arg_cid)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#onPayloadReady, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_onPayloadReady(_arg_cid, _aidl_reply)
  }
  fn r#onPayloadFinished(&self, _arg_cid: i32, _arg_exitCode: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_onPayloadFinished(_arg_cid, _arg_exitCode)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#onPayloadFinished, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_onPayloadFinished(_arg_cid, _arg_exitCode, _aidl_reply)
  }
  fn r#onError(&self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_onError(_arg_cid, _arg_errorCode, _arg_message)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#onError, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_onError(_arg_cid, _arg_errorCode, _arg_message, _aidl_reply)
  }
  fn r#onDied(&self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_onDied(_arg_cid, _arg_reason)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#onDied, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_onDied(_arg_cid, _arg_reason, _aidl_reply)
  }
  fn r#onGuestAgentRegistered(&self, _arg_cid: i32, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_onGuestAgentRegistered(_arg_cid, _arg_guestAgent)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#onGuestAgentRegistered, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_onGuestAgentRegistered(_arg_cid, _arg_guestAgent, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVirtualMachineCallbackAsync<P> for BpVirtualMachineCallback {
  fn r#onPayloadStarted<'a>(&'a self, _arg_cid: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_onPayloadStarted(_arg_cid) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#onPayloadStarted, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_onPayloadStarted(_arg_cid, _aidl_reply)
      }
    )
  }
  fn r#onPayloadReady<'a>(&'a self, _arg_cid: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_onPayloadReady(_arg_cid) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#onPayloadReady, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_onPayloadReady(_arg_cid, _aidl_reply)
      }
    )
  }
  fn r#onPayloadFinished<'a>(&'a self, _arg_cid: i32, _arg_exitCode: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_onPayloadFinished(_arg_cid, _arg_exitCode) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#onPayloadFinished, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_onPayloadFinished(_arg_cid, _arg_exitCode, _aidl_reply)
      }
    )
  }
  fn r#onError<'a>(&'a self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &'a str) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_onError(_arg_cid, _arg_errorCode, _arg_message) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#onError, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_onError(_arg_cid, _arg_errorCode, _arg_message, _aidl_reply)
      }
    )
  }
  fn r#onDied<'a>(&'a self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_onDied(_arg_cid, _arg_reason) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#onDied, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_onDied(_arg_cid, _arg_reason, _aidl_reply)
      }
    )
  }
  fn r#onGuestAgentRegistered<'a>(&'a self, _arg_cid: i32, _arg_guestAgent: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_onGuestAgentRegistered(_arg_cid, _arg_guestAgent) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#onGuestAgentRegistered, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_onGuestAgentRegistered(_arg_cid, _arg_guestAgent, _aidl_reply)
      }
    )
  }
}
impl IVirtualMachineCallback for binder::binder_impl::Binder<BnVirtualMachineCallback> {
  fn r#onPayloadStarted(&self, _arg_cid: i32) -> binder::Result<()> { self.0.r#onPayloadStarted(_arg_cid) }
  fn r#onPayloadReady(&self, _arg_cid: i32) -> binder::Result<()> { self.0.r#onPayloadReady(_arg_cid) }
  fn r#onPayloadFinished(&self, _arg_cid: i32, _arg_exitCode: i32) -> binder::Result<()> { self.0.r#onPayloadFinished(_arg_cid, _arg_exitCode) }
  fn r#onError(&self, _arg_cid: i32, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()> { self.0.r#onError(_arg_cid, _arg_errorCode, _arg_message) }
  fn r#onDied(&self, _arg_cid: i32, _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason) -> binder::Result<()> { self.0.r#onDied(_arg_cid, _arg_reason) }
  fn r#onGuestAgentRegistered(&self, _arg_cid: i32, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()> { self.0.r#onGuestAgentRegistered(_arg_cid, _arg_guestAgent) }
}
fn on_transact(_aidl_service: &dyn IVirtualMachineCallback, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> std::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#onPayloadStarted => {
      let _arg_cid: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#onPayloadStarted(_arg_cid);
      Ok(())
    }
    transactions::r#onPayloadReady => {
      let _arg_cid: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#onPayloadReady(_arg_cid);
      Ok(())
    }
    transactions::r#onPayloadFinished => {
      let _arg_cid: i32 = _aidl_data.read()?;
      let _arg_exitCode: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#onPayloadFinished(_arg_cid, _arg_exitCode);
      Ok(())
    }
    transactions::r#onError => {
      let _arg_cid: i32 = _aidl_data.read()?;
      let _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode = _aidl_data.read()?;
      let _arg_message: String = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#onError(_arg_cid, _arg_errorCode, &_arg_message);
      Ok(())
    }
    transactions::r#onDied => {
      let _arg_cid: i32 = _aidl_data.read()?;
      let _arg_reason: crate::mangled::_7_android_6_system_20_virtualizationcommon_11_DeathReason = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#onDied(_arg_cid, _arg_reason);
      Ok(())
    }
    transactions::r#onGuestAgentRegistered => {
      let _arg_cid: i32 = _aidl_data.read()?;
      let _arg_guestAgent: binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#onGuestAgentRegistered(_arg_cid, &_arg_guestAgent);
      Ok(())
    }
    _ => Err(binder::StatusCode::UNKNOWN_TRANSACTION)
  }
}
pub(crate) mod mangled {
 pub use super::r#IVirtualMachineCallback as _7_android_6_system_21_virtualizationservice_23_IVirtualMachineCallback;
}
