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
  IRpcProvider["android.os.IRpcProvider"] {
    native: BnRpcProvider(on_transact),
    proxy: BpRpcProvider {
    },
    async: IRpcProviderAsync(try_into_local_async),
  }
}
pub trait IRpcProvider: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.os.IRpcProvider" }
  fn r#getServiceConnectionInfo<'a, 'l1, >(&'a self, _arg_name: &'l1 str) -> binder::Result<crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IRpcProviderAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IRpcProviderAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.os.IRpcProvider" }
  fn r#getServiceConnectionInfo<'a, >(&'a self, _arg_name: &'a str) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo>>;
}
#[::async_trait::async_trait]
pub trait IRpcProviderAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.os.IRpcProvider" }
  async fn r#getServiceConnectionInfo<'a, 'l1, >(&'a self, _arg_name: &'l1 str) -> binder::Result<crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo>;
}
impl BnRpcProvider {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IRpcProvider>
  where
    T: IRpcProviderAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IRpcProvider for Wrapper<T, R>
    where
      T: IRpcProviderAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#getServiceConnectionInfo<'a, 'l1, >(&'a self, _arg_name: &'l1 str) -> binder::Result<crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo> {
        self._rt.block_on(self._inner.r#getServiceConnectionInfo(_arg_name))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IRpcProviderAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IRpcProviderAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnRpcProvider>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IRpcProviderAsync<P> for Wrapper {
      fn r#getServiceConnectionInfo<'a, >(&'a self, _arg_name: &'a str) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getServiceConnectionInfo(_arg_name))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IRpcProviderAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#getServiceConnectionInfo: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
}
impl BpRpcProvider {
  fn build_parcel_getServiceConnectionInfo(&self, _arg_name: &str) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_name)?;
    Ok(aidl_data)
  }
  fn read_response_getServiceConnectionInfo(&self, _arg_name: &str, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
}
impl IRpcProvider for BpRpcProvider {
  fn r#getServiceConnectionInfo<'a, 'l1, >(&'a self, _arg_name: &'l1 str) -> binder::Result<crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo> {
    let _aidl_data = self.build_parcel_getServiceConnectionInfo(_arg_name)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getServiceConnectionInfo, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_getServiceConnectionInfo(_arg_name, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IRpcProviderAsync<P> for BpRpcProvider {
  fn r#getServiceConnectionInfo<'a, >(&'a self, _arg_name: &'a str) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo>> {
    let _aidl_data = match self.build_parcel_getServiceConnectionInfo(_arg_name) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getServiceConnectionInfo, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getServiceConnectionInfo(_arg_name, _aidl_reply)
      }
    )
  }
}
impl IRpcProvider for binder::binder_impl::Binder<BnRpcProvider> {
  fn r#getServiceConnectionInfo<'a, 'l1, >(&'a self, _arg_name: &'l1 str) -> binder::Result<crate::mangled::_7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo> { self.0.r#getServiceConnectionInfo(_arg_name) }
}
fn on_transact(_aidl_service: &dyn IRpcProvider, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#getServiceConnectionInfo => {
      let _arg_name: alloc::string::String = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#getServiceConnectionInfo(&_arg_name);
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
pub mod r#Vsock {
  use alloc::boxed::Box;
  #[derive(Debug, Clone, Eq, PartialEq)]
  pub struct r#Vsock {
    pub r#cid: i32,
    pub r#port: i32,
  }
  impl Default for r#Vsock {
    fn default() -> Self {
      Self {
        r#cid: 0,
        r#port: 0,
      }
    }
  }
  impl binder::Parcelable for r#Vsock {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#cid)?;
        subparcel.write(&self.r#port)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#cid = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#port = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#Vsock);
  binder::impl_deserialize_for_parcelable!(r#Vsock);
  impl binder::binder_impl::ParcelableMetadata for r#Vsock {
    fn get_descriptor() -> &'static str { "android.os.IRpcProvider.Vsock" }
  }
}
pub mod r#ServiceConnectionInfo {
  use alloc::boxed::Box;
  #[derive(Debug, Clone, Eq, PartialEq)]
  pub enum r#ServiceConnectionInfo {
    Vsock(crate::mangled::_7_android_2_os_12_IRpcProvider_5_Vsock),
  }
  impl Default for r#ServiceConnectionInfo {
    fn default() -> Self {
      Self::Vsock(Default::default())
    }
  }
  impl binder::Parcelable for r#ServiceConnectionInfo {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      match self {
        Self::Vsock(v) => {
          parcel.write(&0i32)?;
          parcel.write(v)
        }
      }
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      let tag: i32 = parcel.read()?;
      match tag {
        0 => {
          let value: crate::mangled::_7_android_2_os_12_IRpcProvider_5_Vsock = parcel.read()?;
          *self = Self::Vsock(value);
          Ok(())
        }
        _ => {
          Err(binder::StatusCode::BAD_VALUE)
        }
      }
    }
  }
  binder::impl_serialize_for_parcelable!(r#ServiceConnectionInfo);
  binder::impl_deserialize_for_parcelable!(r#ServiceConnectionInfo);
  impl binder::binder_impl::ParcelableMetadata for r#ServiceConnectionInfo {
    fn get_descriptor() -> &'static str { "android.os.IRpcProvider.ServiceConnectionInfo" }
  }
  pub mod r#Tag {
    #![allow(non_upper_case_globals)]
    use binder::declare_binder_enum;
    declare_binder_enum! {
      #[repr(align(4))]
      r#Tag : [i32; 1] {
        r#vsock = 0,
      }
    }
  }
}
pub(crate) mod mangled {
 pub use super::r#IRpcProvider as _7_android_2_os_12_IRpcProvider;
 pub use super::r#Vsock::r#Vsock as _7_android_2_os_12_IRpcProvider_5_Vsock;
 pub use super::r#ServiceConnectionInfo::r#ServiceConnectionInfo as _7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo;
 pub use super::r#ServiceConnectionInfo::r#Tag::r#Tag as _7_android_2_os_12_IRpcProvider_21_ServiceConnectionInfo_3_Tag;
}
