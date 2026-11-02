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
  ICEStoreKEK["android.system.virtualizationcommon.ICEStoreKEK"] {
    native: BnCEStoreKEK(on_transact),
    proxy: BpCEStoreKEK {
    },
    async: ICEStoreKEKAsync(try_into_local_async),
  }
}
pub trait ICEStoreKEK: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationcommon.ICEStoreKEK" }
  fn r#getKEK(&self) -> binder::Result<Option<Vec<u8>>>;
  fn r#onKEKCreated(&self, _arg_kek: &[u8]) -> binder::Result<()>;
  fn getDefaultImpl() -> ICEStoreKEKDefaultRef where Self: Sized {
    DEFAULT_IMPL.lock().unwrap().clone()
  }
  fn setDefaultImpl(d: ICEStoreKEKDefaultRef) -> ICEStoreKEKDefaultRef where Self: Sized {
    std::mem::replace(&mut *DEFAULT_IMPL.lock().unwrap(), d)
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn ICEStoreKEKAsyncServer + Send + Sync)> {
    None
  }
}
pub trait ICEStoreKEKAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationcommon.ICEStoreKEK" }
  fn r#getKEK<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<Vec<u8>>>>;
  fn r#onKEKCreated<'a>(&'a self, _arg_kek: &'a [u8]) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait ICEStoreKEKAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationcommon.ICEStoreKEK" }
  async fn r#getKEK(&self) -> binder::Result<Option<Vec<u8>>>;
  async fn r#onKEKCreated(&self, _arg_kek: &[u8]) -> binder::Result<()>;
}
impl BnCEStoreKEK {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn ICEStoreKEK>
  where
    T: ICEStoreKEKAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> ICEStoreKEK for Wrapper<T, R>
    where
      T: ICEStoreKEKAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#getKEK(&self) -> binder::Result<Option<Vec<u8>>> {
        self._rt.block_on(self._inner.r#getKEK())
      }
      fn r#onKEKCreated(&self, _arg_kek: &[u8]) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#onKEKCreated(_arg_kek))
      }
      fn try_as_async_server(&self) -> Option<&(dyn ICEStoreKEKAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn ICEStoreKEKAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnCEStoreKEK>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> ICEStoreKEKAsync<P> for Wrapper {
      fn r#getKEK<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<Vec<u8>>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getKEK())
      }
      fn r#onKEKCreated<'a>(&'a self, _arg_kek: &'a [u8]) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#onKEKCreated(_arg_kek))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn ICEStoreKEKAsync<P>>))
    } else {
      None
    }
  }
}
pub trait ICEStoreKEKDefault: Send + Sync {
  fn r#getKEK(&self) -> binder::Result<Option<Vec<u8>>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#onKEKCreated(&self, _arg_kek: &[u8]) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
}
pub mod transactions {
  pub const r#getKEK: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#onKEKCreated: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
}
pub type ICEStoreKEKDefaultRef = Option<std::sync::Arc<dyn ICEStoreKEKDefault>>;
static DEFAULT_IMPL: std::sync::Mutex<ICEStoreKEKDefaultRef> = std::sync::Mutex::new(None);
impl BpCEStoreKEK {
  fn build_parcel_getKEK(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getKEK(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Option<Vec<u8>>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as ICEStoreKEK>::getDefaultImpl() {
        return _aidl_default_impl.r#getKEK();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Option<Vec<u8>> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_onKEKCreated(&self, _arg_kek: &[u8]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_kek)?;
    Ok(aidl_data)
  }
  fn read_response_onKEKCreated(&self, _arg_kek: &[u8], _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as ICEStoreKEK>::getDefaultImpl() {
        return _aidl_default_impl.r#onKEKCreated(_arg_kek);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl ICEStoreKEK for BpCEStoreKEK {
  fn r#getKEK(&self) -> binder::Result<Option<Vec<u8>>> {
    let _aidl_data = self.build_parcel_getKEK()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getKEK, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getKEK(_aidl_reply)
  }
  fn r#onKEKCreated(&self, _arg_kek: &[u8]) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_onKEKCreated(_arg_kek)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#onKEKCreated, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_onKEKCreated(_arg_kek, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> ICEStoreKEKAsync<P> for BpCEStoreKEK {
  fn r#getKEK<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<Vec<u8>>>> {
    let _aidl_data = match self.build_parcel_getKEK() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getKEK, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getKEK(_aidl_reply)
      }
    )
  }
  fn r#onKEKCreated<'a>(&'a self, _arg_kek: &'a [u8]) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_onKEKCreated(_arg_kek) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#onKEKCreated, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_onKEKCreated(_arg_kek, _aidl_reply)
      }
    )
  }
}
impl ICEStoreKEK for binder::binder_impl::Binder<BnCEStoreKEK> {
  fn r#getKEK(&self) -> binder::Result<Option<Vec<u8>>> { self.0.r#getKEK() }
  fn r#onKEKCreated(&self, _arg_kek: &[u8]) -> binder::Result<()> { self.0.r#onKEKCreated(_arg_kek) }
}
fn on_transact(_aidl_service: &dyn ICEStoreKEK, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> std::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#getKEK => {
      let _aidl_return = _aidl_service.r#getKEK();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#onKEKCreated => {
      let _arg_kek: Vec<u8> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#onKEKCreated(&_arg_kek);
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
 pub use super::r#ICEStoreKEK as _7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK;
}
