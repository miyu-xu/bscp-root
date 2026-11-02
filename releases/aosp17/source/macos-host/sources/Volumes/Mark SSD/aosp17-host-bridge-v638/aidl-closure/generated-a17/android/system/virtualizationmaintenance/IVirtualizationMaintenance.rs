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
  IVirtualizationMaintenance["android.system.virtualizationmaintenance.IVirtualizationMaintenance"] {
    native: BnVirtualizationMaintenance(on_transact),
    proxy: BpVirtualizationMaintenance {
    },
    async: IVirtualizationMaintenanceAsync(try_into_local_async),
  }
}
pub trait IVirtualizationMaintenance: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationmaintenance.IVirtualizationMaintenance" }
  fn r#appRemoved<'a, >(&'a self, _arg_userId: i32, _arg_appId: i32) -> binder::Result<()>;
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()>;
  fn r#performReconciliation<'a, 'l1, >(&'a self, _arg_callback: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>) -> binder::Result<()>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVirtualizationMaintenanceAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVirtualizationMaintenanceAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationmaintenance.IVirtualizationMaintenance" }
  fn r#appRemoved<'a, >(&'a self, _arg_userId: i32, _arg_appId: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#performReconciliation<'a, >(&'a self, _arg_callback: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IVirtualizationMaintenanceAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationmaintenance.IVirtualizationMaintenance" }
  async fn r#appRemoved<'a, >(&'a self, _arg_userId: i32, _arg_appId: i32) -> binder::Result<()>;
  async fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()>;
  async fn r#performReconciliation<'a, 'l1, >(&'a self, _arg_callback: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>) -> binder::Result<()>;
}
impl BnVirtualizationMaintenance {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVirtualizationMaintenance>
  where
    T: IVirtualizationMaintenanceAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IVirtualizationMaintenance for Wrapper<T, R>
    where
      T: IVirtualizationMaintenanceAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#appRemoved<'a, >(&'a self, _arg_userId: i32, _arg_appId: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#appRemoved(_arg_userId, _arg_appId))
      }
      fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#userRemoved(_arg_userId))
      }
      fn r#performReconciliation<'a, 'l1, >(&'a self, _arg_callback: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#performReconciliation(_arg_callback))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVirtualizationMaintenanceAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVirtualizationMaintenanceAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVirtualizationMaintenance>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVirtualizationMaintenanceAsync<P> for Wrapper {
      fn r#appRemoved<'a, >(&'a self, _arg_userId: i32, _arg_appId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#appRemoved(_arg_userId, _arg_appId))
      }
      fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#userRemoved(_arg_userId))
      }
      fn r#performReconciliation<'a, >(&'a self, _arg_callback: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#performReconciliation(_arg_callback))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVirtualizationMaintenanceAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#appRemoved: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#userRemoved: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#performReconciliation: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
}
impl BpVirtualizationMaintenance {
  fn build_parcel_appRemoved(&self, _arg_userId: i32, _arg_appId: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_userId)?;
    aidl_data.write(&_arg_appId)?;
    Ok(aidl_data)
  }
  fn read_response_appRemoved(&self, _arg_userId: i32, _arg_appId: i32, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_userRemoved(&self, _arg_userId: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_userId)?;
    Ok(aidl_data)
  }
  fn read_response_userRemoved(&self, _arg_userId: i32, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_performReconciliation(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_callback)?;
    Ok(aidl_data)
  }
  fn read_response_performReconciliation(&self, _arg_callback: &binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IVirtualizationMaintenance for BpVirtualizationMaintenance {
  fn r#appRemoved<'a, >(&'a self, _arg_userId: i32, _arg_appId: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_appRemoved(_arg_userId, _arg_appId)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#appRemoved, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_appRemoved(_arg_userId, _arg_appId, _aidl_reply)
  }
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_userRemoved(_arg_userId)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#userRemoved, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_userRemoved(_arg_userId, _aidl_reply)
  }
  fn r#performReconciliation<'a, 'l1, >(&'a self, _arg_callback: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_performReconciliation(_arg_callback)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#performReconciliation, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_performReconciliation(_arg_callback, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVirtualizationMaintenanceAsync<P> for BpVirtualizationMaintenance {
  fn r#appRemoved<'a, >(&'a self, _arg_userId: i32, _arg_appId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_appRemoved(_arg_userId, _arg_appId) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#appRemoved, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_appRemoved(_arg_userId, _arg_appId, _aidl_reply)
      }
    )
  }
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_userRemoved(_arg_userId) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#userRemoved, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_userRemoved(_arg_userId, _aidl_reply)
      }
    )
  }
  fn r#performReconciliation<'a, >(&'a self, _arg_callback: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_performReconciliation(_arg_callback) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#performReconciliation, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_performReconciliation(_arg_callback, _aidl_reply)
      }
    )
  }
}
impl IVirtualizationMaintenance for binder::binder_impl::Binder<BnVirtualizationMaintenance> {
  fn r#appRemoved<'a, >(&'a self, _arg_userId: i32, _arg_appId: i32) -> binder::Result<()> { self.0.r#appRemoved(_arg_userId, _arg_appId) }
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()> { self.0.r#userRemoved(_arg_userId) }
  fn r#performReconciliation<'a, 'l1, >(&'a self, _arg_callback: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback>) -> binder::Result<()> { self.0.r#performReconciliation(_arg_callback) }
}
fn on_transact(_aidl_service: &dyn IVirtualizationMaintenance, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#appRemoved => {
      let _arg_userId: i32 = _aidl_data.read()?;
      let _arg_appId: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#appRemoved(_arg_userId, _arg_appId);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#userRemoved => {
      let _arg_userId: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#userRemoved(_arg_userId);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#performReconciliation => {
      let _arg_callback: binder::Strong<dyn crate::mangled::_7_android_6_system_25_virtualizationmaintenance_37_IVirtualizationReconciliationCallback> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#performReconciliation(&_arg_callback);
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
 pub use super::r#IVirtualizationMaintenance as _7_android_6_system_25_virtualizationmaintenance_26_IVirtualizationMaintenance;
}
