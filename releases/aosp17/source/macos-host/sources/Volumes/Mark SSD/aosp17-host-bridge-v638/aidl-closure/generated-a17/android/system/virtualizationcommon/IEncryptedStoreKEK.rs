/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 *
 * DO NOT CHECK THIS FILE INTO A CODE TREE (e.g. git, etc..).
 * ALWAYS GENERATE THIS FILE FROM UPDATED AIDL COMPILER
 * AS A BUILD INTERMEDIATE ONLY. THIS IS NOT SOURCE CODE.
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#[allow(unused_imports)] use binder::binder_impl::IBinderInternal;
use alloc::boxed::Box;
#[cfg(any(android_vndk, not(android_ndk)))]
const FLAG_PRIVATE_LOCAL: binder::binder_impl::TransactionFlags = binder::binder_impl::FLAG_PRIVATE_LOCAL;
#[cfg(not(any(android_vndk, not(android_ndk))))]
const FLAG_PRIVATE_LOCAL: binder::binder_impl::TransactionFlags = 0;
use binder::declare_binder_interface;
declare_binder_interface! {
  IEncryptedStoreKEK["android.system.virtualizationcommon.IEncryptedStoreKEK"] {
    native: BnEncryptedStoreKEK(on_transact),
    proxy: BpEncryptedStoreKEK {
    },
    async: IEncryptedStoreKEKAsync(try_into_local_async),
  }
}
pub trait IEncryptedStoreKEK: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationcommon.IEncryptedStoreKEK" }
  fn r#getKEK<'a, >(&'a self) -> binder::Result<Option<alloc::vec::Vec<u8>>>;
  fn r#onKEKCreated<'a, 'l1, >(&'a self, _arg_kek: &'l1 [u8]) -> binder::Result<()>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IEncryptedStoreKEKAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IEncryptedStoreKEKAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationcommon.IEncryptedStoreKEK" }
  fn r#getKEK<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<alloc::vec::Vec<u8>>>>;
  fn r#onKEKCreated<'a, >(&'a self, _arg_kek: &'a [u8]) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IEncryptedStoreKEKAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationcommon.IEncryptedStoreKEK" }
  async fn r#getKEK<'a, >(&'a self) -> binder::Result<Option<alloc::vec::Vec<u8>>>;
  async fn r#onKEKCreated<'a, 'l1, >(&'a self, _arg_kek: &'l1 [u8]) -> binder::Result<()>;
}
impl BnEncryptedStoreKEK {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IEncryptedStoreKEK>
  where
    T: IEncryptedStoreKEKAsyncServer + binder::Interface + Send + Sync + 'static,
    R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
  {
    struct Wrapper<T, R> {
      _inner: T,
      _rt: R,
    }
    impl<T, R> binder::Interface for Wrapper<T, R> where T: binder::Interface, R: Send + Sync + 'static {
      fn as_binder(&self) -> binder::SpIBinder { self._inner.as_binder() }
      #[cfg(feature = "std")]
      fn dump(&self, _writer: &mut dyn std::io::Write, _args: &[&std::ffi::CStr]) -> core::result::Result<(), binder::StatusCode> { self._inner.dump(_writer, _args) }
    }
    impl<T, R> IEncryptedStoreKEK for Wrapper<T, R>
    where
      T: IEncryptedStoreKEKAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#getKEK<'a, >(&'a self) -> binder::Result<Option<alloc::vec::Vec<u8>>> {
        self._rt.block_on(self._inner.r#getKEK())
      }
      fn r#onKEKCreated<'a, 'l1, >(&'a self, _arg_kek: &'l1 [u8]) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#onKEKCreated(_arg_kek))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IEncryptedStoreKEKAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IEncryptedStoreKEKAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnEncryptedStoreKEK>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IEncryptedStoreKEKAsync<P> for Wrapper {
      fn r#getKEK<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<alloc::vec::Vec<u8>>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getKEK())
      }
      fn r#onKEKCreated<'a, >(&'a self, _arg_kek: &'a [u8]) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#onKEKCreated(_arg_kek))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IEncryptedStoreKEKAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#getKEK: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#onKEKCreated: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
}
impl BpEncryptedStoreKEK {
  fn build_parcel_getKEK(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getKEK(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Option<alloc::vec::Vec<u8>>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Option<alloc::vec::Vec<u8>> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_onKEKCreated(&self, _arg_kek: &[u8]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_kek)?;
    Ok(aidl_data)
  }
  fn read_response_onKEKCreated(&self, _arg_kek: &[u8], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IEncryptedStoreKEK for BpEncryptedStoreKEK {
  fn r#getKEK<'a, >(&'a self) -> binder::Result<Option<alloc::vec::Vec<u8>>> {
    let _aidl_data = self.build_parcel_getKEK()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getKEK, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_getKEK(_aidl_reply)
  }
  fn r#onKEKCreated<'a, 'l1, >(&'a self, _arg_kek: &'l1 [u8]) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_onKEKCreated(_arg_kek)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#onKEKCreated, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_onKEKCreated(_arg_kek, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IEncryptedStoreKEKAsync<P> for BpEncryptedStoreKEK {
  fn r#getKEK<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<alloc::vec::Vec<u8>>>> {
    let _aidl_data = match self.build_parcel_getKEK() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getKEK, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getKEK(_aidl_reply)
      }
    )
  }
  fn r#onKEKCreated<'a, >(&'a self, _arg_kek: &'a [u8]) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_onKEKCreated(_arg_kek) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#onKEKCreated, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_onKEKCreated(_arg_kek, _aidl_reply)
      }
    )
  }
}
impl IEncryptedStoreKEK for binder::binder_impl::Binder<BnEncryptedStoreKEK> {
  fn r#getKEK<'a, >(&'a self) -> binder::Result<Option<alloc::vec::Vec<u8>>> { self.0.r#getKEK() }
  fn r#onKEKCreated<'a, 'l1, >(&'a self, _arg_kek: &'l1 [u8]) -> binder::Result<()> { self.0.r#onKEKCreated(_arg_kek) }
}
fn on_transact(_aidl_service: &dyn IEncryptedStoreKEK, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
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
      let _arg_kek: alloc::vec::Vec<u8> = _aidl_data.read()?;
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
 pub use super::r#IEncryptedStoreKEK as _7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK;
}
