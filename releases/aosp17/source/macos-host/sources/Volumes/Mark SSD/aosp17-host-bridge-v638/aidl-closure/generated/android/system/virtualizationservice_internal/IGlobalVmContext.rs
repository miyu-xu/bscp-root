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
  IGlobalVmContext["android.system.virtualizationservice_internal.IGlobalVmContext"] {
    native: BnGlobalVmContext(on_transact),
    proxy: BpGlobalVmContext {
    },
    async: IGlobalVmContextAsync(try_into_local_async),
  }
}
pub trait IGlobalVmContext: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IGlobalVmContext" }
  fn r#getCid(&self) -> binder::Result<i32>;
  fn r#getTemporaryDirectory(&self) -> binder::Result<String>;
  fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()>;
  fn getDefaultImpl() -> IGlobalVmContextDefaultRef where Self: Sized {
    DEFAULT_IMPL.lock().unwrap().clone()
  }
  fn setDefaultImpl(d: IGlobalVmContextDefaultRef) -> IGlobalVmContextDefaultRef where Self: Sized {
    std::mem::replace(&mut *DEFAULT_IMPL.lock().unwrap(), d)
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IGlobalVmContextAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IGlobalVmContextAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IGlobalVmContext" }
  fn r#getCid<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>>;
  fn r#getTemporaryDirectory<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>>;
  fn r#setHostConsoleName<'a>(&'a self, _arg_pathname: &'a str) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IGlobalVmContextAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IGlobalVmContext" }
  async fn r#getCid(&self) -> binder::Result<i32>;
  async fn r#getTemporaryDirectory(&self) -> binder::Result<String>;
  async fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()>;
}
impl BnGlobalVmContext {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IGlobalVmContext>
  where
    T: IGlobalVmContextAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IGlobalVmContext for Wrapper<T, R>
    where
      T: IGlobalVmContextAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#getCid(&self) -> binder::Result<i32> {
        self._rt.block_on(self._inner.r#getCid())
      }
      fn r#getTemporaryDirectory(&self) -> binder::Result<String> {
        self._rt.block_on(self._inner.r#getTemporaryDirectory())
      }
      fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#setHostConsoleName(_arg_pathname))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IGlobalVmContextAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IGlobalVmContextAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnGlobalVmContext>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IGlobalVmContextAsync<P> for Wrapper {
      fn r#getCid<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getCid())
      }
      fn r#getTemporaryDirectory<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getTemporaryDirectory())
      }
      fn r#setHostConsoleName<'a>(&'a self, _arg_pathname: &'a str) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#setHostConsoleName(_arg_pathname))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IGlobalVmContextAsync<P>>))
    } else {
      None
    }
  }
}
pub trait IGlobalVmContextDefault: Send + Sync {
  fn r#getCid(&self) -> binder::Result<i32> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getTemporaryDirectory(&self) -> binder::Result<String> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
}
pub mod transactions {
  pub const r#getCid: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#getTemporaryDirectory: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#setHostConsoleName: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
}
pub type IGlobalVmContextDefaultRef = Option<std::sync::Arc<dyn IGlobalVmContextDefault>>;
static DEFAULT_IMPL: std::sync::Mutex<IGlobalVmContextDefaultRef> = std::sync::Mutex::new(None);
impl BpGlobalVmContext {
  fn build_parcel_getCid(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getCid(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<i32> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IGlobalVmContext>::getDefaultImpl() {
        return _aidl_default_impl.r#getCid();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: i32 = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getTemporaryDirectory(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getTemporaryDirectory(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<String> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IGlobalVmContext>::getDefaultImpl() {
        return _aidl_default_impl.r#getTemporaryDirectory();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: String = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_pathname)?;
    Ok(aidl_data)
  }
  fn read_response_setHostConsoleName(&self, _arg_pathname: &str, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IGlobalVmContext>::getDefaultImpl() {
        return _aidl_default_impl.r#setHostConsoleName(_arg_pathname);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IGlobalVmContext for BpGlobalVmContext {
  fn r#getCid(&self) -> binder::Result<i32> {
    let _aidl_data = self.build_parcel_getCid()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getCid, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getCid(_aidl_reply)
  }
  fn r#getTemporaryDirectory(&self) -> binder::Result<String> {
    let _aidl_data = self.build_parcel_getTemporaryDirectory()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getTemporaryDirectory, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getTemporaryDirectory(_aidl_reply)
  }
  fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_setHostConsoleName(_arg_pathname)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#setHostConsoleName, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_setHostConsoleName(_arg_pathname, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IGlobalVmContextAsync<P> for BpGlobalVmContext {
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
  fn r#getTemporaryDirectory<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
    let _aidl_data = match self.build_parcel_getTemporaryDirectory() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getTemporaryDirectory, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getTemporaryDirectory(_aidl_reply)
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
}
impl IGlobalVmContext for binder::binder_impl::Binder<BnGlobalVmContext> {
  fn r#getCid(&self) -> binder::Result<i32> { self.0.r#getCid() }
  fn r#getTemporaryDirectory(&self) -> binder::Result<String> { self.0.r#getTemporaryDirectory() }
  fn r#setHostConsoleName(&self, _arg_pathname: &str) -> binder::Result<()> { self.0.r#setHostConsoleName(_arg_pathname) }
}
fn on_transact(_aidl_service: &dyn IGlobalVmContext, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> std::result::Result<(), binder::StatusCode> {
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
    transactions::r#getTemporaryDirectory => {
      let _aidl_return = _aidl_service.r#getTemporaryDirectory();
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
    _ => Err(binder::StatusCode::UNKNOWN_TRANSACTION)
  }
}
pub(crate) mod mangled {
 pub use super::r#IGlobalVmContext as _7_android_6_system_30_virtualizationservice_internal_16_IGlobalVmContext;
}
