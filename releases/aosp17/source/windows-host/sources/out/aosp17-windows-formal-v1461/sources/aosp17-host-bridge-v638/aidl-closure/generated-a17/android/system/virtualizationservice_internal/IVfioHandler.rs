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
  IVfioHandler["android.system.virtualizationservice_internal.IVfioHandler"] {
    native: BnVfioHandler(on_transact),
    proxy: BpVfioHandler {
    },
    async: IVfioHandlerAsync(try_into_local_async),
  }
}
pub trait IVfioHandler: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IVfioHandler" }
  fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>;
  fn r#writeVmDtbo<'a, 'l1, >(&'a self, _arg_dtbo: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVfioHandlerAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVfioHandlerAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IVfioHandler" }
  fn r#bindDevicesToVfioDriver<'a, >(&'a self, _arg_devices: &'a [crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>>;
  fn r#writeVmDtbo<'a, >(&'a self, _arg_dtbo: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IVfioHandlerAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IVfioHandler" }
  async fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>;
  async fn r#writeVmDtbo<'a, 'l1, >(&'a self, _arg_dtbo: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()>;
}
impl BnVfioHandler {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVfioHandler>
  where
    T: IVfioHandlerAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IVfioHandler for Wrapper<T, R>
    where
      T: IVfioHandlerAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>> {
        self._rt.block_on(self._inner.r#bindDevicesToVfioDriver(_arg_devices))
      }
      fn r#writeVmDtbo<'a, 'l1, >(&'a self, _arg_dtbo: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#writeVmDtbo(_arg_dtbo))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVfioHandlerAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVfioHandlerAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVfioHandler>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVfioHandlerAsync<P> for Wrapper {
      fn r#bindDevicesToVfioDriver<'a, >(&'a self, _arg_devices: &'a [crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#bindDevicesToVfioDriver(_arg_devices))
      }
      fn r#writeVmDtbo<'a, >(&'a self, _arg_dtbo: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#writeVmDtbo(_arg_dtbo))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVfioHandlerAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#bindDevicesToVfioDriver: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#writeVmDtbo: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
}
impl BpVfioHandler {
  fn build_parcel_bindDevicesToVfioDriver(&self, _arg_devices: &[crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_devices)?;
    Ok(aidl_data)
  }
  fn read_response_bindDevicesToVfioDriver(&self, _arg_devices: &[crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_writeVmDtbo(&self, _arg_dtbo: &binder::ParcelFileDescriptor) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_dtbo)?;
    Ok(aidl_data)
  }
  fn read_response_writeVmDtbo(&self, _arg_dtbo: &binder::ParcelFileDescriptor, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IVfioHandler for BpVfioHandler {
  fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>> {
    let _aidl_data = self.build_parcel_bindDevicesToVfioDriver(_arg_devices)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#bindDevicesToVfioDriver, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_bindDevicesToVfioDriver(_arg_devices, _aidl_reply)
  }
  fn r#writeVmDtbo<'a, 'l1, >(&'a self, _arg_dtbo: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_writeVmDtbo(_arg_dtbo)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#writeVmDtbo, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_writeVmDtbo(_arg_dtbo, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVfioHandlerAsync<P> for BpVfioHandler {
  fn r#bindDevicesToVfioDriver<'a, >(&'a self, _arg_devices: &'a [crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>> {
    let _aidl_data = match self.build_parcel_bindDevicesToVfioDriver(_arg_devices) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#bindDevicesToVfioDriver, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_bindDevicesToVfioDriver(_arg_devices, _aidl_reply)
      }
    )
  }
  fn r#writeVmDtbo<'a, >(&'a self, _arg_dtbo: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_writeVmDtbo(_arg_dtbo) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#writeVmDtbo, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_writeVmDtbo(_arg_dtbo, _aidl_reply)
      }
    )
  }
}
impl IVfioHandler for binder::binder_impl::Binder<BnVfioHandler> {
  fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>> { self.0.r#bindDevicesToVfioDriver(_arg_devices) }
  fn r#writeVmDtbo<'a, 'l1, >(&'a self, _arg_dtbo: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()> { self.0.r#writeVmDtbo(_arg_dtbo) }
}
fn on_transact(_aidl_service: &dyn IVfioHandler, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#bindDevicesToVfioDriver => {
      let _arg_devices: alloc::vec::Vec<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#bindDevicesToVfioDriver(&_arg_devices);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#writeVmDtbo => {
      let _arg_dtbo: binder::ParcelFileDescriptor = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#writeVmDtbo(&_arg_dtbo);
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
pub mod r#VfioDev {
  use alloc::boxed::Box;
  #[derive(Debug)]
  pub struct r#VfioDev {
    pub r#sysfsPath: alloc::string::String,
    pub r#dtboLabel: alloc::string::String,
  }
  impl Default for r#VfioDev {
    fn default() -> Self {
      Self {
        r#sysfsPath: Default::default(),
        r#dtboLabel: Default::default(),
      }
    }
  }
  impl binder::Parcelable for r#VfioDev {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#sysfsPath)?;
        subparcel.write(&self.r#dtboLabel)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#sysfsPath = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#dtboLabel = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#VfioDev);
  binder::impl_deserialize_for_parcelable!(r#VfioDev);
  impl binder::binder_impl::ParcelableMetadata for r#VfioDev {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice_internal.IVfioHandler.VfioDev" }
  }
}
pub(crate) mod mangled {
 pub use super::r#IVfioHandler as _7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler;
 pub use super::r#VfioDev::r#VfioDev as _7_android_6_system_30_virtualizationservice_internal_12_IVfioHandler_7_VfioDev;
}
