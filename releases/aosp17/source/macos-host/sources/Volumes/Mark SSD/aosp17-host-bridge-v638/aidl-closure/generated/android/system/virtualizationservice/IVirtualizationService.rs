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
  IVirtualizationService["android.system.virtualizationservice.IVirtualizationService"] {
    native: BnVirtualizationService(on_transact),
    proxy: BpVirtualizationService {
    },
    async: IVirtualizationServiceAsync(try_into_local_async),
  }
}
pub trait IVirtualizationService: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice.IVirtualizationService" }
  fn r#createVm(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&binder::ParcelFileDescriptor>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>>;
  fn r#allocateInstanceId(&self) -> binder::Result<[u8; 64]>;
  fn r#initializeWritablePartition(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::Result<()>;
  fn r#setEncryptedStorageSize(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_size: i64) -> binder::Result<()>;
  fn r#createOrUpdateIdsigFile(&self, _arg_inputFd: &binder::ParcelFileDescriptor, _arg_idsigFd: &binder::ParcelFileDescriptor) -> binder::Result<()>;
  fn r#debugListVms(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>;
  fn r#getAssignableDevices(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>;
  fn r#getSupportedOSList(&self) -> binder::Result<Vec<String>>;
  fn r#getDebugPolicy(&self) -> binder::Result<String>;
  fn r#isFeatureEnabled(&self, _arg_feature: &str) -> binder::Result<bool>;
  fn r#enableTestAttestation(&self) -> binder::Result<()>;
  fn r#isRemoteAttestationSupported(&self) -> binder::Result<bool>;
  fn r#isUpdatableVmSupported(&self) -> binder::Result<bool>;
  fn r#removeVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()>;
  fn r#claimVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()>;
  fn r#startOrStopAdbd(&self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()>;
  fn r#requestShutdown(&self) -> binder::Result<()>;
  fn getDefaultImpl() -> IVirtualizationServiceDefaultRef where Self: Sized {
    DEFAULT_IMPL.lock().unwrap().clone()
  }
  fn setDefaultImpl(d: IVirtualizationServiceDefaultRef) -> IVirtualizationServiceDefaultRef where Self: Sized {
    std::mem::replace(&mut *DEFAULT_IMPL.lock().unwrap(), d)
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVirtualizationServiceAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVirtualizationServiceAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice.IVirtualizationService" }
  fn r#createVm<'a>(&'a self, _arg_config: &'a crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&'a binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&'a binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&'a binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&'a binder::ParcelFileDescriptor>) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>>>;
  fn r#allocateInstanceId<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<[u8; 64]>>;
  fn r#initializeWritablePartition<'a>(&'a self, _arg_imageFd: &'a binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#setEncryptedStorageSize<'a>(&'a self, _arg_imageFd: &'a binder::ParcelFileDescriptor, _arg_size: i64) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#createOrUpdateIdsigFile<'a>(&'a self, _arg_inputFd: &'a binder::ParcelFileDescriptor, _arg_idsigFd: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#debugListVms<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>>;
  fn r#getAssignableDevices<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>>;
  fn r#getSupportedOSList<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<String>>>;
  fn r#getDebugPolicy<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>>;
  fn r#isFeatureEnabled<'a>(&'a self, _arg_feature: &'a str) -> binder::BoxFuture<'a, binder::Result<bool>>;
  fn r#enableTestAttestation<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#isRemoteAttestationSupported<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>>;
  fn r#isUpdatableVmSupported<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>>;
  fn r#removeVmInstance<'a>(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#claimVmInstance<'a>(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#startOrStopAdbd<'a>(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#requestShutdown<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IVirtualizationServiceAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice.IVirtualizationService" }
  async fn r#createVm(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&binder::ParcelFileDescriptor>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>>;
  async fn r#allocateInstanceId(&self) -> binder::Result<[u8; 64]>;
  async fn r#initializeWritablePartition(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::Result<()>;
  async fn r#setEncryptedStorageSize(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_size: i64) -> binder::Result<()>;
  async fn r#createOrUpdateIdsigFile(&self, _arg_inputFd: &binder::ParcelFileDescriptor, _arg_idsigFd: &binder::ParcelFileDescriptor) -> binder::Result<()>;
  async fn r#debugListVms(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>;
  async fn r#getAssignableDevices(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>;
  async fn r#getSupportedOSList(&self) -> binder::Result<Vec<String>>;
  async fn r#getDebugPolicy(&self) -> binder::Result<String>;
  async fn r#isFeatureEnabled(&self, _arg_feature: &str) -> binder::Result<bool>;
  async fn r#enableTestAttestation(&self) -> binder::Result<()>;
  async fn r#isRemoteAttestationSupported(&self) -> binder::Result<bool>;
  async fn r#isUpdatableVmSupported(&self) -> binder::Result<bool>;
  async fn r#removeVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()>;
  async fn r#claimVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()>;
  async fn r#startOrStopAdbd(&self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()>;
  async fn r#requestShutdown(&self) -> binder::Result<()>;
}
impl BnVirtualizationService {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVirtualizationService>
  where
    T: IVirtualizationServiceAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IVirtualizationService for Wrapper<T, R>
    where
      T: IVirtualizationServiceAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#createVm(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&binder::ParcelFileDescriptor>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>> {
        self._rt.block_on(self._inner.r#createVm(_arg_config, _arg_consoleOutFd, _arg_consoleInFd, _arg_osLogFd, _arg_dumpDtFd))
      }
      fn r#allocateInstanceId(&self) -> binder::Result<[u8; 64]> {
        self._rt.block_on(self._inner.r#allocateInstanceId())
      }
      fn r#initializeWritablePartition(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#initializeWritablePartition(_arg_imageFd, _arg_sizeBytes, _arg_type))
      }
      fn r#setEncryptedStorageSize(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_size: i64) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#setEncryptedStorageSize(_arg_imageFd, _arg_size))
      }
      fn r#createOrUpdateIdsigFile(&self, _arg_inputFd: &binder::ParcelFileDescriptor, _arg_idsigFd: &binder::ParcelFileDescriptor) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#createOrUpdateIdsigFile(_arg_inputFd, _arg_idsigFd))
      }
      fn r#debugListVms(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> {
        self._rt.block_on(self._inner.r#debugListVms())
      }
      fn r#getAssignableDevices(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>> {
        self._rt.block_on(self._inner.r#getAssignableDevices())
      }
      fn r#getSupportedOSList(&self) -> binder::Result<Vec<String>> {
        self._rt.block_on(self._inner.r#getSupportedOSList())
      }
      fn r#getDebugPolicy(&self) -> binder::Result<String> {
        self._rt.block_on(self._inner.r#getDebugPolicy())
      }
      fn r#isFeatureEnabled(&self, _arg_feature: &str) -> binder::Result<bool> {
        self._rt.block_on(self._inner.r#isFeatureEnabled(_arg_feature))
      }
      fn r#enableTestAttestation(&self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#enableTestAttestation())
      }
      fn r#isRemoteAttestationSupported(&self) -> binder::Result<bool> {
        self._rt.block_on(self._inner.r#isRemoteAttestationSupported())
      }
      fn r#isUpdatableVmSupported(&self) -> binder::Result<bool> {
        self._rt.block_on(self._inner.r#isUpdatableVmSupported())
      }
      fn r#removeVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#removeVmInstance(_arg_instanceId))
      }
      fn r#claimVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#claimVmInstance(_arg_instanceId))
      }
      fn r#startOrStopAdbd(&self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#startOrStopAdbd(_arg_cid, _arg_start))
      }
      fn r#requestShutdown(&self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#requestShutdown())
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVirtualizationServiceAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVirtualizationServiceAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVirtualizationService>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVirtualizationServiceAsync<P> for Wrapper {
      fn r#createVm<'a>(&'a self, _arg_config: &'a crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&'a binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&'a binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&'a binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&'a binder::ParcelFileDescriptor>) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#createVm(_arg_config, _arg_consoleOutFd, _arg_consoleInFd, _arg_osLogFd, _arg_dumpDtFd))
      }
      fn r#allocateInstanceId<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<[u8; 64]>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#allocateInstanceId())
      }
      fn r#initializeWritablePartition<'a>(&'a self, _arg_imageFd: &'a binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#initializeWritablePartition(_arg_imageFd, _arg_sizeBytes, _arg_type))
      }
      fn r#setEncryptedStorageSize<'a>(&'a self, _arg_imageFd: &'a binder::ParcelFileDescriptor, _arg_size: i64) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#setEncryptedStorageSize(_arg_imageFd, _arg_size))
      }
      fn r#createOrUpdateIdsigFile<'a>(&'a self, _arg_inputFd: &'a binder::ParcelFileDescriptor, _arg_idsigFd: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#createOrUpdateIdsigFile(_arg_inputFd, _arg_idsigFd))
      }
      fn r#debugListVms<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#debugListVms())
      }
      fn r#getAssignableDevices<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getAssignableDevices())
      }
      fn r#getSupportedOSList<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<String>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getSupportedOSList())
      }
      fn r#getDebugPolicy<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getDebugPolicy())
      }
      fn r#isFeatureEnabled<'a>(&'a self, _arg_feature: &'a str) -> binder::BoxFuture<'a, binder::Result<bool>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#isFeatureEnabled(_arg_feature))
      }
      fn r#enableTestAttestation<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#enableTestAttestation())
      }
      fn r#isRemoteAttestationSupported<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#isRemoteAttestationSupported())
      }
      fn r#isUpdatableVmSupported<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#isUpdatableVmSupported())
      }
      fn r#removeVmInstance<'a>(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#removeVmInstance(_arg_instanceId))
      }
      fn r#claimVmInstance<'a>(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#claimVmInstance(_arg_instanceId))
      }
      fn r#startOrStopAdbd<'a>(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#startOrStopAdbd(_arg_cid, _arg_start))
      }
      fn r#requestShutdown<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#requestShutdown())
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVirtualizationServiceAsync<P>>))
    } else {
      None
    }
  }
}
pub trait IVirtualizationServiceDefault: Send + Sync {
  fn r#createVm(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&binder::ParcelFileDescriptor>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#allocateInstanceId(&self) -> binder::Result<[u8; 64]> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#initializeWritablePartition(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#setEncryptedStorageSize(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_size: i64) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#createOrUpdateIdsigFile(&self, _arg_inputFd: &binder::ParcelFileDescriptor, _arg_idsigFd: &binder::ParcelFileDescriptor) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#debugListVms(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getAssignableDevices(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getSupportedOSList(&self) -> binder::Result<Vec<String>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getDebugPolicy(&self) -> binder::Result<String> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#isFeatureEnabled(&self, _arg_feature: &str) -> binder::Result<bool> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#enableTestAttestation(&self) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#isRemoteAttestationSupported(&self) -> binder::Result<bool> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#isUpdatableVmSupported(&self) -> binder::Result<bool> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#removeVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#claimVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#startOrStopAdbd(&self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#requestShutdown(&self) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
}
pub mod transactions {
  pub const r#createVm: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#allocateInstanceId: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#initializeWritablePartition: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
  pub const r#setEncryptedStorageSize: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 3;
  pub const r#createOrUpdateIdsigFile: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 4;
  pub const r#debugListVms: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 5;
  pub const r#getAssignableDevices: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 6;
  pub const r#getSupportedOSList: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 7;
  pub const r#getDebugPolicy: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 8;
  pub const r#isFeatureEnabled: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 9;
  pub const r#enableTestAttestation: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 10;
  pub const r#isRemoteAttestationSupported: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 11;
  pub const r#isUpdatableVmSupported: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 12;
  pub const r#removeVmInstance: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 13;
  pub const r#claimVmInstance: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 14;
  pub const r#startOrStopAdbd: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 15;
  pub const r#requestShutdown: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 16;
}
pub type IVirtualizationServiceDefaultRef = Option<std::sync::Arc<dyn IVirtualizationServiceDefault>>;
static DEFAULT_IMPL: std::sync::Mutex<IVirtualizationServiceDefaultRef> = std::sync::Mutex::new(None);
pub const r#FEATURE_DICE_CHANGES: &str = "com.android.kvm.DICE_CHANGES";
pub const r#FEATURE_LLPVM_CHANGES: &str = "com.android.kvm.LLPVM_CHANGES";
pub const r#FEATURE_NETWORK: &str = "com.android.kvm.NETWORK";
pub const r#FEATURE_VENDOR_MODULES: &str = "com.android.kvm.VENDOR_MODULES";
pub const r#FEATURE_ADVANCE_MULTITENANCY: &str = "com.android.kvm.ADVANCE_MULTITENANCY";
impl BpVirtualizationService {
  fn build_parcel_createVm(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&binder::ParcelFileDescriptor>) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_config)?;
    aidl_data.write(&_arg_consoleOutFd)?;
    aidl_data.write(&_arg_consoleInFd)?;
    aidl_data.write(&_arg_osLogFd)?;
    aidl_data.write(&_arg_dumpDtFd)?;
    Ok(aidl_data)
  }
  fn read_response_createVm(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&binder::ParcelFileDescriptor>, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#createVm(_arg_config, _arg_consoleOutFd, _arg_consoleInFd, _arg_osLogFd, _arg_dumpDtFd);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_allocateInstanceId(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_allocateInstanceId(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<[u8; 64]> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#allocateInstanceId();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: [u8; 64] = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_initializeWritablePartition(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_imageFd)?;
    aidl_data.write(&_arg_sizeBytes)?;
    aidl_data.write(&_arg_type)?;
    Ok(aidl_data)
  }
  fn read_response_initializeWritablePartition(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#initializeWritablePartition(_arg_imageFd, _arg_sizeBytes, _arg_type);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_setEncryptedStorageSize(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_size: i64) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_imageFd)?;
    aidl_data.write(&_arg_size)?;
    Ok(aidl_data)
  }
  fn read_response_setEncryptedStorageSize(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_size: i64, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#setEncryptedStorageSize(_arg_imageFd, _arg_size);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_createOrUpdateIdsigFile(&self, _arg_inputFd: &binder::ParcelFileDescriptor, _arg_idsigFd: &binder::ParcelFileDescriptor) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_inputFd)?;
    aidl_data.write(_arg_idsigFd)?;
    Ok(aidl_data)
  }
  fn read_response_createOrUpdateIdsigFile(&self, _arg_inputFd: &binder::ParcelFileDescriptor, _arg_idsigFd: &binder::ParcelFileDescriptor, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#createOrUpdateIdsigFile(_arg_inputFd, _arg_idsigFd);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_debugListVms(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_debugListVms(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#debugListVms();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getAssignableDevices(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getAssignableDevices(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#getAssignableDevices();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getSupportedOSList(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getSupportedOSList(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Vec<String>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#getSupportedOSList();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Vec<String> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getDebugPolicy(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getDebugPolicy(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<String> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#getDebugPolicy();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: String = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_isFeatureEnabled(&self, _arg_feature: &str) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_feature)?;
    Ok(aidl_data)
  }
  fn read_response_isFeatureEnabled(&self, _arg_feature: &str, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<bool> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#isFeatureEnabled(_arg_feature);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: bool = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_enableTestAttestation(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_enableTestAttestation(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#enableTestAttestation();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_isRemoteAttestationSupported(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_isRemoteAttestationSupported(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<bool> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#isRemoteAttestationSupported();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: bool = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_isUpdatableVmSupported(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_isUpdatableVmSupported(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<bool> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#isUpdatableVmSupported();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: bool = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_removeVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_instanceId)?;
    Ok(aidl_data)
  }
  fn read_response_removeVmInstance(&self, _arg_instanceId: &[u8; 64], _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#removeVmInstance(_arg_instanceId);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_claimVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_instanceId)?;
    Ok(aidl_data)
  }
  fn read_response_claimVmInstance(&self, _arg_instanceId: &[u8; 64], _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#claimVmInstance(_arg_instanceId);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_startOrStopAdbd(&self, _arg_cid: i32, _arg_start: bool) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_cid)?;
    aidl_data.write(&_arg_start)?;
    Ok(aidl_data)
  }
  fn read_response_startOrStopAdbd(&self, _arg_cid: i32, _arg_start: bool, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#startOrStopAdbd(_arg_cid, _arg_start);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_requestShutdown(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_requestShutdown(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualizationService>::getDefaultImpl() {
        return _aidl_default_impl.r#requestShutdown();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IVirtualizationService for BpVirtualizationService {
  fn r#createVm(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&binder::ParcelFileDescriptor>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>> {
    let _aidl_data = self.build_parcel_createVm(_arg_config, _arg_consoleOutFd, _arg_consoleInFd, _arg_osLogFd, _arg_dumpDtFd)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#createVm, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_createVm(_arg_config, _arg_consoleOutFd, _arg_consoleInFd, _arg_osLogFd, _arg_dumpDtFd, _aidl_reply)
  }
  fn r#allocateInstanceId(&self) -> binder::Result<[u8; 64]> {
    let _aidl_data = self.build_parcel_allocateInstanceId()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#allocateInstanceId, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_allocateInstanceId(_aidl_reply)
  }
  fn r#initializeWritablePartition(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_initializeWritablePartition(_arg_imageFd, _arg_sizeBytes, _arg_type)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#initializeWritablePartition, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_initializeWritablePartition(_arg_imageFd, _arg_sizeBytes, _arg_type, _aidl_reply)
  }
  fn r#setEncryptedStorageSize(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_size: i64) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_setEncryptedStorageSize(_arg_imageFd, _arg_size)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#setEncryptedStorageSize, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_setEncryptedStorageSize(_arg_imageFd, _arg_size, _aidl_reply)
  }
  fn r#createOrUpdateIdsigFile(&self, _arg_inputFd: &binder::ParcelFileDescriptor, _arg_idsigFd: &binder::ParcelFileDescriptor) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_createOrUpdateIdsigFile(_arg_inputFd, _arg_idsigFd)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#createOrUpdateIdsigFile, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_createOrUpdateIdsigFile(_arg_inputFd, _arg_idsigFd, _aidl_reply)
  }
  fn r#debugListVms(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> {
    let _aidl_data = self.build_parcel_debugListVms()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#debugListVms, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_debugListVms(_aidl_reply)
  }
  fn r#getAssignableDevices(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>> {
    let _aidl_data = self.build_parcel_getAssignableDevices()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getAssignableDevices, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getAssignableDevices(_aidl_reply)
  }
  fn r#getSupportedOSList(&self) -> binder::Result<Vec<String>> {
    let _aidl_data = self.build_parcel_getSupportedOSList()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getSupportedOSList, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getSupportedOSList(_aidl_reply)
  }
  fn r#getDebugPolicy(&self) -> binder::Result<String> {
    let _aidl_data = self.build_parcel_getDebugPolicy()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getDebugPolicy, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getDebugPolicy(_aidl_reply)
  }
  fn r#isFeatureEnabled(&self, _arg_feature: &str) -> binder::Result<bool> {
    let _aidl_data = self.build_parcel_isFeatureEnabled(_arg_feature)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#isFeatureEnabled, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_isFeatureEnabled(_arg_feature, _aidl_reply)
  }
  fn r#enableTestAttestation(&self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_enableTestAttestation()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#enableTestAttestation, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_enableTestAttestation(_aidl_reply)
  }
  fn r#isRemoteAttestationSupported(&self) -> binder::Result<bool> {
    let _aidl_data = self.build_parcel_isRemoteAttestationSupported()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#isRemoteAttestationSupported, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_isRemoteAttestationSupported(_aidl_reply)
  }
  fn r#isUpdatableVmSupported(&self) -> binder::Result<bool> {
    let _aidl_data = self.build_parcel_isUpdatableVmSupported()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#isUpdatableVmSupported, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_isUpdatableVmSupported(_aidl_reply)
  }
  fn r#removeVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_removeVmInstance(_arg_instanceId)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#removeVmInstance, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_removeVmInstance(_arg_instanceId, _aidl_reply)
  }
  fn r#claimVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_claimVmInstance(_arg_instanceId)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#claimVmInstance, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_claimVmInstance(_arg_instanceId, _aidl_reply)
  }
  fn r#startOrStopAdbd(&self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_startOrStopAdbd(_arg_cid, _arg_start)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#startOrStopAdbd, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_startOrStopAdbd(_arg_cid, _arg_start, _aidl_reply)
  }
  fn r#requestShutdown(&self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_requestShutdown()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#requestShutdown, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_requestShutdown(_aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVirtualizationServiceAsync<P> for BpVirtualizationService {
  fn r#createVm<'a>(&'a self, _arg_config: &'a crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&'a binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&'a binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&'a binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&'a binder::ParcelFileDescriptor>) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>>> {
    let _aidl_data = match self.build_parcel_createVm(_arg_config, _arg_consoleOutFd, _arg_consoleInFd, _arg_osLogFd, _arg_dumpDtFd) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#createVm, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_createVm(_arg_config, _arg_consoleOutFd, _arg_consoleInFd, _arg_osLogFd, _arg_dumpDtFd, _aidl_reply)
      }
    )
  }
  fn r#allocateInstanceId<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<[u8; 64]>> {
    let _aidl_data = match self.build_parcel_allocateInstanceId() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#allocateInstanceId, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_allocateInstanceId(_aidl_reply)
      }
    )
  }
  fn r#initializeWritablePartition<'a>(&'a self, _arg_imageFd: &'a binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_initializeWritablePartition(_arg_imageFd, _arg_sizeBytes, _arg_type) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#initializeWritablePartition, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_initializeWritablePartition(_arg_imageFd, _arg_sizeBytes, _arg_type, _aidl_reply)
      }
    )
  }
  fn r#setEncryptedStorageSize<'a>(&'a self, _arg_imageFd: &'a binder::ParcelFileDescriptor, _arg_size: i64) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_setEncryptedStorageSize(_arg_imageFd, _arg_size) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#setEncryptedStorageSize, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_setEncryptedStorageSize(_arg_imageFd, _arg_size, _aidl_reply)
      }
    )
  }
  fn r#createOrUpdateIdsigFile<'a>(&'a self, _arg_inputFd: &'a binder::ParcelFileDescriptor, _arg_idsigFd: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_createOrUpdateIdsigFile(_arg_inputFd, _arg_idsigFd) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#createOrUpdateIdsigFile, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_createOrUpdateIdsigFile(_arg_inputFd, _arg_idsigFd, _aidl_reply)
      }
    )
  }
  fn r#debugListVms<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>> {
    let _aidl_data = match self.build_parcel_debugListVms() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#debugListVms, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_debugListVms(_aidl_reply)
      }
    )
  }
  fn r#getAssignableDevices<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>> {
    let _aidl_data = match self.build_parcel_getAssignableDevices() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getAssignableDevices, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getAssignableDevices(_aidl_reply)
      }
    )
  }
  fn r#getSupportedOSList<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<String>>> {
    let _aidl_data = match self.build_parcel_getSupportedOSList() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getSupportedOSList, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getSupportedOSList(_aidl_reply)
      }
    )
  }
  fn r#getDebugPolicy<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
    let _aidl_data = match self.build_parcel_getDebugPolicy() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getDebugPolicy, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getDebugPolicy(_aidl_reply)
      }
    )
  }
  fn r#isFeatureEnabled<'a>(&'a self, _arg_feature: &'a str) -> binder::BoxFuture<'a, binder::Result<bool>> {
    let _aidl_data = match self.build_parcel_isFeatureEnabled(_arg_feature) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#isFeatureEnabled, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_isFeatureEnabled(_arg_feature, _aidl_reply)
      }
    )
  }
  fn r#enableTestAttestation<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_enableTestAttestation() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#enableTestAttestation, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_enableTestAttestation(_aidl_reply)
      }
    )
  }
  fn r#isRemoteAttestationSupported<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>> {
    let _aidl_data = match self.build_parcel_isRemoteAttestationSupported() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#isRemoteAttestationSupported, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_isRemoteAttestationSupported(_aidl_reply)
      }
    )
  }
  fn r#isUpdatableVmSupported<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>> {
    let _aidl_data = match self.build_parcel_isUpdatableVmSupported() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#isUpdatableVmSupported, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_isUpdatableVmSupported(_aidl_reply)
      }
    )
  }
  fn r#removeVmInstance<'a>(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_removeVmInstance(_arg_instanceId) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#removeVmInstance, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_removeVmInstance(_arg_instanceId, _aidl_reply)
      }
    )
  }
  fn r#claimVmInstance<'a>(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_claimVmInstance(_arg_instanceId) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#claimVmInstance, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_claimVmInstance(_arg_instanceId, _aidl_reply)
      }
    )
  }
  fn r#startOrStopAdbd<'a>(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_startOrStopAdbd(_arg_cid, _arg_start) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#startOrStopAdbd, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_startOrStopAdbd(_arg_cid, _arg_start, _aidl_reply)
      }
    )
  }
  fn r#requestShutdown<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_requestShutdown() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#requestShutdown, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_requestShutdown(_aidl_reply)
      }
    )
  }
}
impl IVirtualizationService for binder::binder_impl::Binder<BnVirtualizationService> {
  fn r#createVm(&self, _arg_config: &crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig, _arg_consoleOutFd: Option<&binder::ParcelFileDescriptor>, _arg_consoleInFd: Option<&binder::ParcelFileDescriptor>, _arg_osLogFd: Option<&binder::ParcelFileDescriptor>, _arg_dumpDtFd: Option<&binder::ParcelFileDescriptor>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>> { self.0.r#createVm(_arg_config, _arg_consoleOutFd, _arg_consoleInFd, _arg_osLogFd, _arg_dumpDtFd) }
  fn r#allocateInstanceId(&self) -> binder::Result<[u8; 64]> { self.0.r#allocateInstanceId() }
  fn r#initializeWritablePartition(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_sizeBytes: i64, _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType) -> binder::Result<()> { self.0.r#initializeWritablePartition(_arg_imageFd, _arg_sizeBytes, _arg_type) }
  fn r#setEncryptedStorageSize(&self, _arg_imageFd: &binder::ParcelFileDescriptor, _arg_size: i64) -> binder::Result<()> { self.0.r#setEncryptedStorageSize(_arg_imageFd, _arg_size) }
  fn r#createOrUpdateIdsigFile(&self, _arg_inputFd: &binder::ParcelFileDescriptor, _arg_idsigFd: &binder::ParcelFileDescriptor) -> binder::Result<()> { self.0.r#createOrUpdateIdsigFile(_arg_inputFd, _arg_idsigFd) }
  fn r#debugListVms(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> { self.0.r#debugListVms() }
  fn r#getAssignableDevices(&self) -> binder::Result<Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>> { self.0.r#getAssignableDevices() }
  fn r#getSupportedOSList(&self) -> binder::Result<Vec<String>> { self.0.r#getSupportedOSList() }
  fn r#getDebugPolicy(&self) -> binder::Result<String> { self.0.r#getDebugPolicy() }
  fn r#isFeatureEnabled(&self, _arg_feature: &str) -> binder::Result<bool> { self.0.r#isFeatureEnabled(_arg_feature) }
  fn r#enableTestAttestation(&self) -> binder::Result<()> { self.0.r#enableTestAttestation() }
  fn r#isRemoteAttestationSupported(&self) -> binder::Result<bool> { self.0.r#isRemoteAttestationSupported() }
  fn r#isUpdatableVmSupported(&self) -> binder::Result<bool> { self.0.r#isUpdatableVmSupported() }
  fn r#removeVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()> { self.0.r#removeVmInstance(_arg_instanceId) }
  fn r#claimVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<()> { self.0.r#claimVmInstance(_arg_instanceId) }
  fn r#startOrStopAdbd(&self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()> { self.0.r#startOrStopAdbd(_arg_cid, _arg_start) }
  fn r#requestShutdown(&self) -> binder::Result<()> { self.0.r#requestShutdown() }
}
fn on_transact(_aidl_service: &dyn IVirtualizationService, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> std::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#createVm => {
      let _arg_config: crate::mangled::_7_android_6_system_21_virtualizationservice_20_VirtualMachineConfig = _aidl_data.read()?;
      let _arg_consoleOutFd: Option<binder::ParcelFileDescriptor> = _aidl_data.read()?;
      let _arg_consoleInFd: Option<binder::ParcelFileDescriptor> = _aidl_data.read()?;
      let _arg_osLogFd: Option<binder::ParcelFileDescriptor> = _aidl_data.read()?;
      let _arg_dumpDtFd: Option<binder::ParcelFileDescriptor> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#createVm(&_arg_config, _arg_consoleOutFd.as_ref(), _arg_consoleInFd.as_ref(), _arg_osLogFd.as_ref(), _arg_dumpDtFd.as_ref());
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#allocateInstanceId => {
      let _aidl_return = _aidl_service.r#allocateInstanceId();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#initializeWritablePartition => {
      let _arg_imageFd: binder::ParcelFileDescriptor = _aidl_data.read()?;
      let _arg_sizeBytes: i64 = _aidl_data.read()?;
      let _arg_type: crate::mangled::_7_android_6_system_21_virtualizationservice_13_PartitionType = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#initializeWritablePartition(&_arg_imageFd, _arg_sizeBytes, _arg_type);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#setEncryptedStorageSize => {
      let _arg_imageFd: binder::ParcelFileDescriptor = _aidl_data.read()?;
      let _arg_size: i64 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#setEncryptedStorageSize(&_arg_imageFd, _arg_size);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#createOrUpdateIdsigFile => {
      let _arg_inputFd: binder::ParcelFileDescriptor = _aidl_data.read()?;
      let _arg_idsigFd: binder::ParcelFileDescriptor = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#createOrUpdateIdsigFile(&_arg_inputFd, &_arg_idsigFd);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#debugListVms => {
      let _aidl_return = _aidl_service.r#debugListVms();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getAssignableDevices => {
      let _aidl_return = _aidl_service.r#getAssignableDevices();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getSupportedOSList => {
      let _aidl_return = _aidl_service.r#getSupportedOSList();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getDebugPolicy => {
      let _aidl_return = _aidl_service.r#getDebugPolicy();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#isFeatureEnabled => {
      let _arg_feature: String = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#isFeatureEnabled(&_arg_feature);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#enableTestAttestation => {
      let _aidl_return = _aidl_service.r#enableTestAttestation();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#isRemoteAttestationSupported => {
      let _aidl_return = _aidl_service.r#isRemoteAttestationSupported();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#isUpdatableVmSupported => {
      let _aidl_return = _aidl_service.r#isUpdatableVmSupported();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#removeVmInstance => {
      let _arg_instanceId: [u8; 64] = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#removeVmInstance(&_arg_instanceId);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#claimVmInstance => {
      let _arg_instanceId: [u8; 64] = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#claimVmInstance(&_arg_instanceId);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#startOrStopAdbd => {
      let _arg_cid: i32 = _aidl_data.read()?;
      let _arg_start: bool = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#startOrStopAdbd(_arg_cid, _arg_start);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#requestShutdown => {
      let _aidl_return = _aidl_service.r#requestShutdown();
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
 pub use super::r#IVirtualizationService as _7_android_6_system_21_virtualizationservice_22_IVirtualizationService;
}
