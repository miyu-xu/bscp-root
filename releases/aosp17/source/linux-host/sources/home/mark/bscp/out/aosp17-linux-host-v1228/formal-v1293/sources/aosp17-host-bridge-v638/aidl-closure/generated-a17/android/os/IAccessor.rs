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
  IAccessor["android.os.IAccessor"] {
    native: BnAccessor(on_transact),
    proxy: BpAccessor {
    },
    async: IAccessorAsync(try_into_local_async),
  }
}
pub trait IAccessor: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.os.IAccessor" }
  fn r#addConnection<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor>;
  fn r#getInstanceName<'a, >(&'a self) -> binder::Result<alloc::string::String>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IAccessorAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IAccessorAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.os.IAccessor" }
  fn r#addConnection<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>>;
  fn r#getInstanceName<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::string::String>>;
}
#[::async_trait::async_trait]
pub trait IAccessorAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.os.IAccessor" }
  async fn r#addConnection<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor>;
  async fn r#getInstanceName<'a, >(&'a self) -> binder::Result<alloc::string::String>;
}
impl BnAccessor {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IAccessor>
  where
    T: IAccessorAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IAccessor for Wrapper<T, R>
    where
      T: IAccessorAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#addConnection<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor> {
        self._rt.block_on(self._inner.r#addConnection())
      }
      fn r#getInstanceName<'a, >(&'a self) -> binder::Result<alloc::string::String> {
        self._rt.block_on(self._inner.r#getInstanceName())
      }
      fn try_as_async_server(&self) -> Option<&(dyn IAccessorAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IAccessorAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnAccessor>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IAccessorAsync<P> for Wrapper {
      fn r#addConnection<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#addConnection())
      }
      fn r#getInstanceName<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::string::String>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getInstanceName())
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IAccessorAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#addConnection: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#getInstanceName: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
}
pub const r#ERROR_CONNECTION_INFO_NOT_FOUND: i32 = 0;
pub const r#ERROR_FAILED_TO_CREATE_SOCKET: i32 = 1;
pub const r#ERROR_FAILED_TO_CONNECT_TO_SOCKET: i32 = 2;
pub const r#ERROR_FAILED_TO_CONNECT_EACCES: i32 = 3;
pub const r#ERROR_UNSUPPORTED_SOCKET_FAMILY: i32 = 4;
impl BpAccessor {
  fn build_parcel_addConnection(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_addConnection(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::ParcelFileDescriptor> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::ParcelFileDescriptor = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getInstanceName(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getInstanceName(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::string::String> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::string::String = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
}
impl IAccessor for BpAccessor {
  fn r#addConnection<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor> {
    let _aidl_data = self.build_parcel_addConnection()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#addConnection, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_addConnection(_aidl_reply)
  }
  fn r#getInstanceName<'a, >(&'a self) -> binder::Result<alloc::string::String> {
    let _aidl_data = self.build_parcel_getInstanceName()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getInstanceName, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_getInstanceName(_aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IAccessorAsync<P> for BpAccessor {
  fn r#addConnection<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>> {
    let _aidl_data = match self.build_parcel_addConnection() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#addConnection, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_addConnection(_aidl_reply)
      }
    )
  }
  fn r#getInstanceName<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::string::String>> {
    let _aidl_data = match self.build_parcel_getInstanceName() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getInstanceName, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getInstanceName(_aidl_reply)
      }
    )
  }
}
impl IAccessor for binder::binder_impl::Binder<BnAccessor> {
  fn r#addConnection<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor> { self.0.r#addConnection() }
  fn r#getInstanceName<'a, >(&'a self) -> binder::Result<alloc::string::String> { self.0.r#getInstanceName() }
}
fn on_transact(_aidl_service: &dyn IAccessor, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#addConnection => {
      let _aidl_return = _aidl_service.r#addConnection();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getInstanceName => {
      let _aidl_return = _aidl_service.r#getInstanceName();
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
 pub use super::r#IAccessor as _7_android_2_os_9_IAccessor;
}
