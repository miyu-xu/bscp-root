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
  IVmnic["android.system.virtualizationservice_internal.IVmnic"] {
    native: BnVmnic(on_transact),
    proxy: BpVmnic {
    },
    async: IVmnicAsync(try_into_local_async),
  }
}
pub trait IVmnic: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IVmnic" }
  fn r#createTapInterface<'a, 'l1, >(&'a self, _arg_ifaceNameSuffix: &'l1 str) -> binder::Result<binder::ParcelFileDescriptor>;
  fn r#deleteTapInterface<'a, 'l1, >(&'a self, _arg_tapFd: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVmnicAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVmnicAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IVmnic" }
  fn r#createTapInterface<'a, >(&'a self, _arg_ifaceNameSuffix: &'a str) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>>;
  fn r#deleteTapInterface<'a, >(&'a self, _arg_tapFd: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IVmnicAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IVmnic" }
  async fn r#createTapInterface<'a, 'l1, >(&'a self, _arg_ifaceNameSuffix: &'l1 str) -> binder::Result<binder::ParcelFileDescriptor>;
  async fn r#deleteTapInterface<'a, 'l1, >(&'a self, _arg_tapFd: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()>;
}
impl BnVmnic {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVmnic>
  where
    T: IVmnicAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IVmnic for Wrapper<T, R>
    where
      T: IVmnicAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#createTapInterface<'a, 'l1, >(&'a self, _arg_ifaceNameSuffix: &'l1 str) -> binder::Result<binder::ParcelFileDescriptor> {
        self._rt.block_on(self._inner.r#createTapInterface(_arg_ifaceNameSuffix))
      }
      fn r#deleteTapInterface<'a, 'l1, >(&'a self, _arg_tapFd: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#deleteTapInterface(_arg_tapFd))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVmnicAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVmnicAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVmnic>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVmnicAsync<P> for Wrapper {
      fn r#createTapInterface<'a, >(&'a self, _arg_ifaceNameSuffix: &'a str) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#createTapInterface(_arg_ifaceNameSuffix))
      }
      fn r#deleteTapInterface<'a, >(&'a self, _arg_tapFd: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#deleteTapInterface(_arg_tapFd))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVmnicAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#createTapInterface: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#deleteTapInterface: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
}
impl BpVmnic {
  fn build_parcel_createTapInterface(&self, _arg_ifaceNameSuffix: &str) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_ifaceNameSuffix)?;
    Ok(aidl_data)
  }
  fn read_response_createTapInterface(&self, _arg_ifaceNameSuffix: &str, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::ParcelFileDescriptor> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::ParcelFileDescriptor = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_deleteTapInterface(&self, _arg_tapFd: &binder::ParcelFileDescriptor) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_tapFd)?;
    Ok(aidl_data)
  }
  fn read_response_deleteTapInterface(&self, _arg_tapFd: &binder::ParcelFileDescriptor, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IVmnic for BpVmnic {
  fn r#createTapInterface<'a, 'l1, >(&'a self, _arg_ifaceNameSuffix: &'l1 str) -> binder::Result<binder::ParcelFileDescriptor> {
    let _aidl_data = self.build_parcel_createTapInterface(_arg_ifaceNameSuffix)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#createTapInterface, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_createTapInterface(_arg_ifaceNameSuffix, _aidl_reply)
  }
  fn r#deleteTapInterface<'a, 'l1, >(&'a self, _arg_tapFd: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_deleteTapInterface(_arg_tapFd)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#deleteTapInterface, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_deleteTapInterface(_arg_tapFd, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVmnicAsync<P> for BpVmnic {
  fn r#createTapInterface<'a, >(&'a self, _arg_ifaceNameSuffix: &'a str) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>> {
    let _aidl_data = match self.build_parcel_createTapInterface(_arg_ifaceNameSuffix) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#createTapInterface, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_createTapInterface(_arg_ifaceNameSuffix, _aidl_reply)
      }
    )
  }
  fn r#deleteTapInterface<'a, >(&'a self, _arg_tapFd: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_deleteTapInterface(_arg_tapFd) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#deleteTapInterface, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_deleteTapInterface(_arg_tapFd, _aidl_reply)
      }
    )
  }
}
impl IVmnic for binder::binder_impl::Binder<BnVmnic> {
  fn r#createTapInterface<'a, 'l1, >(&'a self, _arg_ifaceNameSuffix: &'l1 str) -> binder::Result<binder::ParcelFileDescriptor> { self.0.r#createTapInterface(_arg_ifaceNameSuffix) }
  fn r#deleteTapInterface<'a, 'l1, >(&'a self, _arg_tapFd: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()> { self.0.r#deleteTapInterface(_arg_tapFd) }
}
fn on_transact(_aidl_service: &dyn IVmnic, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#createTapInterface => {
      let _arg_ifaceNameSuffix: alloc::string::String = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#createTapInterface(&_arg_ifaceNameSuffix);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#deleteTapInterface => {
      let _arg_tapFd: binder::ParcelFileDescriptor = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#deleteTapInterface(&_arg_tapFd);
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
 pub use super::r#IVmnic as _7_android_6_system_30_virtualizationservice_internal_6_IVmnic;
}
