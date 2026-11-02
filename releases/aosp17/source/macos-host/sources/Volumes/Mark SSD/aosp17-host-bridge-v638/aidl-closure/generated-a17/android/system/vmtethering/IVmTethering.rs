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
  IVmTethering["android.system.vmtethering.IVmTethering"] {
    native: BnVmTethering(on_transact),
    proxy: BpVmTethering {
    },
    async: IVmTetheringAsync(try_into_local_async),
  }
}
pub trait IVmTethering: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.vmtethering.IVmTethering" }
  fn r#enableVmTethering<'a, >(&'a self) -> binder::Result<()>;
  fn r#disableVmTethering<'a, >(&'a self) -> binder::Result<()>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVmTetheringAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVmTetheringAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.vmtethering.IVmTethering" }
  fn r#enableVmTethering<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#disableVmTethering<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IVmTetheringAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.vmtethering.IVmTethering" }
  async fn r#enableVmTethering<'a, >(&'a self) -> binder::Result<()>;
  async fn r#disableVmTethering<'a, >(&'a self) -> binder::Result<()>;
}
impl BnVmTethering {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVmTethering>
  where
    T: IVmTetheringAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IVmTethering for Wrapper<T, R>
    where
      T: IVmTetheringAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#enableVmTethering<'a, >(&'a self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#enableVmTethering())
      }
      fn r#disableVmTethering<'a, >(&'a self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#disableVmTethering())
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVmTetheringAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVmTetheringAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVmTethering>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVmTetheringAsync<P> for Wrapper {
      fn r#enableVmTethering<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#enableVmTethering())
      }
      fn r#disableVmTethering<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#disableVmTethering())
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVmTetheringAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#enableVmTethering: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#disableVmTethering: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
}
impl BpVmTethering {
  fn build_parcel_enableVmTethering(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_enableVmTethering(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_disableVmTethering(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_disableVmTethering(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IVmTethering for BpVmTethering {
  fn r#enableVmTethering<'a, >(&'a self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_enableVmTethering()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#enableVmTethering, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_enableVmTethering(_aidl_reply)
  }
  fn r#disableVmTethering<'a, >(&'a self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_disableVmTethering()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#disableVmTethering, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_disableVmTethering(_aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVmTetheringAsync<P> for BpVmTethering {
  fn r#enableVmTethering<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_enableVmTethering() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#enableVmTethering, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_enableVmTethering(_aidl_reply)
      }
    )
  }
  fn r#disableVmTethering<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_disableVmTethering() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#disableVmTethering, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_disableVmTethering(_aidl_reply)
      }
    )
  }
}
impl IVmTethering for binder::binder_impl::Binder<BnVmTethering> {
  fn r#enableVmTethering<'a, >(&'a self) -> binder::Result<()> { self.0.r#enableVmTethering() }
  fn r#disableVmTethering<'a, >(&'a self) -> binder::Result<()> { self.0.r#disableVmTethering() }
}
fn on_transact(_aidl_service: &dyn IVmTethering, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#enableVmTethering => {
      let _aidl_return = _aidl_service.r#enableVmTethering();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#disableVmTethering => {
      let _aidl_return = _aidl_service.r#disableVmTethering();
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
 pub use super::r#IVmTethering as _7_android_6_system_11_vmtethering_12_IVmTethering;
}
