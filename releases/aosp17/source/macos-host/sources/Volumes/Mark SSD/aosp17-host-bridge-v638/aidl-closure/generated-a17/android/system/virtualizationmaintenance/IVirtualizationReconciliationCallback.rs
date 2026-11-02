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
  IVirtualizationReconciliationCallback["android.system.virtualizationmaintenance.IVirtualizationReconciliationCallback"] {
    native: BnVirtualizationReconciliationCallback(on_transact),
    proxy: BpVirtualizationReconciliationCallback {
    },
    async: IVirtualizationReconciliationCallbackAsync(try_into_local_async),
  }
}
pub trait IVirtualizationReconciliationCallback: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationmaintenance.IVirtualizationReconciliationCallback" }
  fn r#doUsersExist<'a, 'l1, >(&'a self, _arg_userIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>>;
  fn r#doAppsExist<'a, 'l1, >(&'a self, _arg_userId: i32, _arg_appIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVirtualizationReconciliationCallbackAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVirtualizationReconciliationCallbackAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationmaintenance.IVirtualizationReconciliationCallback" }
  fn r#doUsersExist<'a, >(&'a self, _arg_userIds: &'a [i32]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<bool>>>;
  fn r#doAppsExist<'a, >(&'a self, _arg_userId: i32, _arg_appIds: &'a [i32]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<bool>>>;
}
#[::async_trait::async_trait]
pub trait IVirtualizationReconciliationCallbackAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationmaintenance.IVirtualizationReconciliationCallback" }
  async fn r#doUsersExist<'a, 'l1, >(&'a self, _arg_userIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>>;
  async fn r#doAppsExist<'a, 'l1, >(&'a self, _arg_userId: i32, _arg_appIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>>;
}
impl BnVirtualizationReconciliationCallback {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVirtualizationReconciliationCallback>
  where
    T: IVirtualizationReconciliationCallbackAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IVirtualizationReconciliationCallback for Wrapper<T, R>
    where
      T: IVirtualizationReconciliationCallbackAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#doUsersExist<'a, 'l1, >(&'a self, _arg_userIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>> {
        self._rt.block_on(self._inner.r#doUsersExist(_arg_userIds))
      }
      fn r#doAppsExist<'a, 'l1, >(&'a self, _arg_userId: i32, _arg_appIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>> {
        self._rt.block_on(self._inner.r#doAppsExist(_arg_userId, _arg_appIds))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVirtualizationReconciliationCallbackAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVirtualizationReconciliationCallbackAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVirtualizationReconciliationCallback>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVirtualizationReconciliationCallbackAsync<P> for Wrapper {
      fn r#doUsersExist<'a, >(&'a self, _arg_userIds: &'a [i32]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<bool>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#doUsersExist(_arg_userIds))
      }
      fn r#doAppsExist<'a, >(&'a self, _arg_userId: i32, _arg_appIds: &'a [i32]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<bool>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#doAppsExist(_arg_userId, _arg_appIds))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVirtualizationReconciliationCallbackAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#doUsersExist: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#doAppsExist: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
}
pub const r#ERROR_STOP_REQUESTED: i32 = 1;
impl BpVirtualizationReconciliationCallback {
  fn build_parcel_doUsersExist(&self, _arg_userIds: &[i32]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_userIds)?;
    Ok(aidl_data)
  }
  fn read_response_doUsersExist(&self, _arg_userIds: &[i32], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::vec::Vec<bool>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::vec::Vec<bool> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_doAppsExist(&self, _arg_userId: i32, _arg_appIds: &[i32]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_userId)?;
    aidl_data.write(_arg_appIds)?;
    Ok(aidl_data)
  }
  fn read_response_doAppsExist(&self, _arg_userId: i32, _arg_appIds: &[i32], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::vec::Vec<bool>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::vec::Vec<bool> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
}
impl IVirtualizationReconciliationCallback for BpVirtualizationReconciliationCallback {
  fn r#doUsersExist<'a, 'l1, >(&'a self, _arg_userIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>> {
    let _aidl_data = self.build_parcel_doUsersExist(_arg_userIds)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#doUsersExist, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_doUsersExist(_arg_userIds, _aidl_reply)
  }
  fn r#doAppsExist<'a, 'l1, >(&'a self, _arg_userId: i32, _arg_appIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>> {
    let _aidl_data = self.build_parcel_doAppsExist(_arg_userId, _arg_appIds)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#doAppsExist, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_doAppsExist(_arg_userId, _arg_appIds, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVirtualizationReconciliationCallbackAsync<P> for BpVirtualizationReconciliationCallback {
  fn r#doUsersExist<'a, >(&'a self, _arg_userIds: &'a [i32]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<bool>>> {
    let _aidl_data = match self.build_parcel_doUsersExist(_arg_userIds) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#doUsersExist, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_doUsersExist(_arg_userIds, _aidl_reply)
      }
    )
  }
  fn r#doAppsExist<'a, >(&'a self, _arg_userId: i32, _arg_appIds: &'a [i32]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<bool>>> {
    let _aidl_data = match self.build_parcel_doAppsExist(_arg_userId, _arg_appIds) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#doAppsExist, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_doAppsExist(_arg_userId, _arg_appIds, _aidl_reply)
      }
    )
  }
}
impl IVirtualizationReconciliationCallback for binder::binder_impl::Binder<BnVirtualizationReconciliationCallback> {
  fn r#doUsersExist<'a, 'l1, >(&'a self, _arg_userIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>> { self.0.r#doUsersExist(_arg_userIds) }
  fn r#doAppsExist<'a, 'l1, >(&'a self, _arg_userId: i32, _arg_appIds: &'l1 [i32]) -> binder::Result<alloc::vec::Vec<bool>> { self.0.r#doAppsExist(_arg_userId, _arg_appIds) }
}
fn on_transact(_aidl_service: &dyn IVirtualizationReconciliationCallback, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#doUsersExist => {
      let _arg_userIds: alloc::vec::Vec<i32> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#doUsersExist(&_arg_userIds);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#doAppsExist => {
      let _arg_userId: i32 = _aidl_data.read()?;
      let _arg_appIds: alloc::vec::Vec<i32> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#doAppsExist(_arg_userId, &_arg_appIds);
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
 pub use super::r#IVirtualizationReconciliationCallback as _7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback;
}
