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
  IBoundDevice["android.system.virtualizationservice_internal.IBoundDevice"] {
    native: BnBoundDevice(on_transact),
    proxy: BpBoundDevice {
    },
    async: IBoundDeviceAsync(try_into_local_async),
  }
}
pub trait IBoundDevice: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IBoundDevice" }
  fn r#getSysfsPath(&self) -> binder::Result<String>;
  fn r#getDtboLabel(&self) -> binder::Result<String>;
  fn getDefaultImpl() -> IBoundDeviceDefaultRef where Self: Sized {
    DEFAULT_IMPL.lock().unwrap().clone()
  }
  fn setDefaultImpl(d: IBoundDeviceDefaultRef) -> IBoundDeviceDefaultRef where Self: Sized {
    std::mem::replace(&mut *DEFAULT_IMPL.lock().unwrap(), d)
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IBoundDeviceAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IBoundDeviceAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IBoundDevice" }
  fn r#getSysfsPath<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>>;
  fn r#getDtboLabel<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>>;
}
#[::async_trait::async_trait]
pub trait IBoundDeviceAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IBoundDevice" }
  async fn r#getSysfsPath(&self) -> binder::Result<String>;
  async fn r#getDtboLabel(&self) -> binder::Result<String>;
}
impl BnBoundDevice {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IBoundDevice>
  where
    T: IBoundDeviceAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IBoundDevice for Wrapper<T, R>
    where
      T: IBoundDeviceAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#getSysfsPath(&self) -> binder::Result<String> {
        self._rt.block_on(self._inner.r#getSysfsPath())
      }
      fn r#getDtboLabel(&self) -> binder::Result<String> {
        self._rt.block_on(self._inner.r#getDtboLabel())
      }
      fn try_as_async_server(&self) -> Option<&(dyn IBoundDeviceAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IBoundDeviceAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnBoundDevice>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IBoundDeviceAsync<P> for Wrapper {
      fn r#getSysfsPath<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getSysfsPath())
      }
      fn r#getDtboLabel<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getDtboLabel())
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IBoundDeviceAsync<P>>))
    } else {
      None
    }
  }
}
pub trait IBoundDeviceDefault: Send + Sync {
  fn r#getSysfsPath(&self) -> binder::Result<String> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getDtboLabel(&self) -> binder::Result<String> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
}
pub mod transactions {
  pub const r#getSysfsPath: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#getDtboLabel: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
}
pub type IBoundDeviceDefaultRef = Option<std::sync::Arc<dyn IBoundDeviceDefault>>;
static DEFAULT_IMPL: std::sync::Mutex<IBoundDeviceDefaultRef> = std::sync::Mutex::new(None);
impl BpBoundDevice {
  fn build_parcel_getSysfsPath(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getSysfsPath(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<String> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IBoundDevice>::getDefaultImpl() {
        return _aidl_default_impl.r#getSysfsPath();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: String = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getDtboLabel(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getDtboLabel(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<String> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IBoundDevice>::getDefaultImpl() {
        return _aidl_default_impl.r#getDtboLabel();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: String = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
}
impl IBoundDevice for BpBoundDevice {
  fn r#getSysfsPath(&self) -> binder::Result<String> {
    let _aidl_data = self.build_parcel_getSysfsPath()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getSysfsPath, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getSysfsPath(_aidl_reply)
  }
  fn r#getDtboLabel(&self) -> binder::Result<String> {
    let _aidl_data = self.build_parcel_getDtboLabel()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getDtboLabel, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getDtboLabel(_aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IBoundDeviceAsync<P> for BpBoundDevice {
  fn r#getSysfsPath<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
    let _aidl_data = match self.build_parcel_getSysfsPath() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getSysfsPath, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getSysfsPath(_aidl_reply)
      }
    )
  }
  fn r#getDtboLabel<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
    let _aidl_data = match self.build_parcel_getDtboLabel() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getDtboLabel, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getDtboLabel(_aidl_reply)
      }
    )
  }
}
impl IBoundDevice for binder::binder_impl::Binder<BnBoundDevice> {
  fn r#getSysfsPath(&self) -> binder::Result<String> { self.0.r#getSysfsPath() }
  fn r#getDtboLabel(&self) -> binder::Result<String> { self.0.r#getDtboLabel() }
}
fn on_transact(_aidl_service: &dyn IBoundDevice, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> std::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#getSysfsPath => {
      let _aidl_return = _aidl_service.r#getSysfsPath();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getDtboLabel => {
      let _aidl_return = _aidl_service.r#getDtboLabel();
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
 pub use super::r#IBoundDevice as _7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice;
}
