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
  IVirtualizationServiceInternal["android.system.virtualizationservice_internal.IVirtualizationServiceInternal"] {
    native: BnVirtualizationServiceInternal(on_transact),
    proxy: BpVirtualizationServiceInternal {
    },
    async: IVirtualizationServiceInternalAsync(try_into_local_async),
  }
}
pub trait IVirtualizationServiceInternal: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IVirtualizationServiceInternal" }
  fn r#removeMemlockRlimit<'a, >(&'a self) -> binder::Result<()>;
  fn r#allocateVmContext<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext>;
  fn r#registerVirtualMachine<'a, 'l1, >(&'a self, _arg_cid: i32, _arg_vm: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>) -> binder::Result<()>;
  fn r#unregisterVirtualMachine<'a, >(&'a self, _arg_cid: i32) -> binder::Result<()>;
  fn r#forwardAtom<'a, 'l1, 'l2, >(&'a self, _arg_atom: &'l1 crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &'l2 str) -> binder::Result<()>;
  fn r#debugListVms<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>;
  fn r#requestAttestation<'a, 'l1, >(&'a self, _arg_csr: &'l1 [u8], _arg_requesterUid: i32, _arg_testMode: bool) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>;
  fn r#enableTestAttestation<'a, >(&'a self) -> binder::Result<()>;
  fn r#isRemoteAttestationSupported<'a, >(&'a self) -> binder::Result<bool>;
  fn r#getAssignableDevices<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>;
  fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [alloc::string::String]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>;
  fn r#getDtboFile<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor>;
  fn r#allocateInstanceId<'a, >(&'a self) -> binder::Result<[u8; 64]>;
  fn r#removeVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()>;
  fn r#claimVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()>;
  fn r#setDisplayService<'a, 'l1, >(&'a self, _arg_ibinder: &'l1 binder::SpIBinder) -> binder::Result<()>;
  fn r#clearDisplayService<'a, >(&'a self) -> binder::Result<()>;
  fn r#waitDisplayService<'a, >(&'a self) -> binder::Result<binder::SpIBinder>;
  fn r#createTapInterface<'a, 'l1, >(&'a self, _arg_ifaceNameSuffix: &'l1 str) -> binder::Result<binder::ParcelFileDescriptor>;
  fn r#deleteTapInterface<'a, 'l1, >(&'a self, _arg_tapFd: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()>;
  fn r#claimSecretkeeperEntry<'a, 'l1, >(&'a self, _arg_id: &'l1 [u8; 64]) -> binder::Result<()>;
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVirtualizationServiceInternalAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVirtualizationServiceInternalAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IVirtualizationServiceInternal" }
  fn r#removeMemlockRlimit<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#allocateVmContext<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext>>;
  fn r#registerVirtualMachine<'a, >(&'a self, _arg_cid: i32, _arg_vm: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#unregisterVirtualMachine<'a, >(&'a self, _arg_cid: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#forwardAtom<'a, >(&'a self, _arg_atom: &'a crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &'a str) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#debugListVms<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>>;
  fn r#requestAttestation<'a, >(&'a self, _arg_csr: &'a [u8], _arg_requesterUid: i32, _arg_testMode: bool) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>>;
  fn r#enableTestAttestation<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#isRemoteAttestationSupported<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>>;
  fn r#getAssignableDevices<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>>;
  fn r#bindDevicesToVfioDriver<'a, >(&'a self, _arg_devices: &'a [alloc::string::String]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>>;
  fn r#getDtboFile<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>>;
  fn r#allocateInstanceId<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<[u8; 64]>>;
  fn r#removeVmInstance<'a, >(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#claimVmInstance<'a, >(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#setDisplayService<'a, >(&'a self, _arg_ibinder: &'a binder::SpIBinder) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#clearDisplayService<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#waitDisplayService<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::SpIBinder>>;
  fn r#createTapInterface<'a, >(&'a self, _arg_ifaceNameSuffix: &'a str) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>>;
  fn r#deleteTapInterface<'a, >(&'a self, _arg_tapFd: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#claimSecretkeeperEntry<'a, >(&'a self, _arg_id: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IVirtualizationServiceInternalAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationservice_internal.IVirtualizationServiceInternal" }
  async fn r#removeMemlockRlimit<'a, >(&'a self) -> binder::Result<()>;
  async fn r#allocateVmContext<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext>;
  async fn r#registerVirtualMachine<'a, 'l1, >(&'a self, _arg_cid: i32, _arg_vm: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>) -> binder::Result<()>;
  async fn r#unregisterVirtualMachine<'a, >(&'a self, _arg_cid: i32) -> binder::Result<()>;
  async fn r#forwardAtom<'a, 'l1, 'l2, >(&'a self, _arg_atom: &'l1 crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &'l2 str) -> binder::Result<()>;
  async fn r#debugListVms<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>;
  async fn r#requestAttestation<'a, 'l1, >(&'a self, _arg_csr: &'l1 [u8], _arg_requesterUid: i32, _arg_testMode: bool) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>;
  async fn r#enableTestAttestation<'a, >(&'a self) -> binder::Result<()>;
  async fn r#isRemoteAttestationSupported<'a, >(&'a self) -> binder::Result<bool>;
  async fn r#getAssignableDevices<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>;
  async fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [alloc::string::String]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>;
  async fn r#getDtboFile<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor>;
  async fn r#allocateInstanceId<'a, >(&'a self) -> binder::Result<[u8; 64]>;
  async fn r#removeVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()>;
  async fn r#claimVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()>;
  async fn r#setDisplayService<'a, 'l1, >(&'a self, _arg_ibinder: &'l1 binder::SpIBinder) -> binder::Result<()>;
  async fn r#clearDisplayService<'a, >(&'a self) -> binder::Result<()>;
  async fn r#waitDisplayService<'a, >(&'a self) -> binder::Result<binder::SpIBinder>;
  async fn r#createTapInterface<'a, 'l1, >(&'a self, _arg_ifaceNameSuffix: &'l1 str) -> binder::Result<binder::ParcelFileDescriptor>;
  async fn r#deleteTapInterface<'a, 'l1, >(&'a self, _arg_tapFd: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()>;
  async fn r#claimSecretkeeperEntry<'a, 'l1, >(&'a self, _arg_id: &'l1 [u8; 64]) -> binder::Result<()>;
  async fn r#startOrStopAdbd<'a, >(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()>;
}
impl BnVirtualizationServiceInternal {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVirtualizationServiceInternal>
  where
    T: IVirtualizationServiceInternalAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IVirtualizationServiceInternal for Wrapper<T, R>
    where
      T: IVirtualizationServiceInternalAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#removeMemlockRlimit<'a, >(&'a self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#removeMemlockRlimit())
      }
      fn r#allocateVmContext<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext> {
        self._rt.block_on(self._inner.r#allocateVmContext())
      }
      fn r#registerVirtualMachine<'a, 'l1, >(&'a self, _arg_cid: i32, _arg_vm: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#registerVirtualMachine(_arg_cid, _arg_vm))
      }
      fn r#unregisterVirtualMachine<'a, >(&'a self, _arg_cid: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#unregisterVirtualMachine(_arg_cid))
      }
      fn r#forwardAtom<'a, 'l1, 'l2, >(&'a self, _arg_atom: &'l1 crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &'l2 str) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#forwardAtom(_arg_atom, _arg_requesterUid, _arg_vmIdentifier))
      }
      fn r#debugListVms<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> {
        self._rt.block_on(self._inner.r#debugListVms())
      }
      fn r#requestAttestation<'a, 'l1, >(&'a self, _arg_csr: &'l1 [u8], _arg_requesterUid: i32, _arg_testMode: bool) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>> {
        self._rt.block_on(self._inner.r#requestAttestation(_arg_csr, _arg_requesterUid, _arg_testMode))
      }
      fn r#enableTestAttestation<'a, >(&'a self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#enableTestAttestation())
      }
      fn r#isRemoteAttestationSupported<'a, >(&'a self) -> binder::Result<bool> {
        self._rt.block_on(self._inner.r#isRemoteAttestationSupported())
      }
      fn r#getAssignableDevices<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>> {
        self._rt.block_on(self._inner.r#getAssignableDevices())
      }
      fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [alloc::string::String]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>> {
        self._rt.block_on(self._inner.r#bindDevicesToVfioDriver(_arg_devices))
      }
      fn r#getDtboFile<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor> {
        self._rt.block_on(self._inner.r#getDtboFile())
      }
      fn r#allocateInstanceId<'a, >(&'a self) -> binder::Result<[u8; 64]> {
        self._rt.block_on(self._inner.r#allocateInstanceId())
      }
      fn r#removeVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#removeVmInstance(_arg_instanceId))
      }
      fn r#claimVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#claimVmInstance(_arg_instanceId))
      }
      fn r#setDisplayService<'a, 'l1, >(&'a self, _arg_ibinder: &'l1 binder::SpIBinder) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#setDisplayService(_arg_ibinder))
      }
      fn r#clearDisplayService<'a, >(&'a self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#clearDisplayService())
      }
      fn r#waitDisplayService<'a, >(&'a self) -> binder::Result<binder::SpIBinder> {
        self._rt.block_on(self._inner.r#waitDisplayService())
      }
      fn r#createTapInterface<'a, 'l1, >(&'a self, _arg_ifaceNameSuffix: &'l1 str) -> binder::Result<binder::ParcelFileDescriptor> {
        self._rt.block_on(self._inner.r#createTapInterface(_arg_ifaceNameSuffix))
      }
      fn r#deleteTapInterface<'a, 'l1, >(&'a self, _arg_tapFd: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#deleteTapInterface(_arg_tapFd))
      }
      fn r#claimSecretkeeperEntry<'a, 'l1, >(&'a self, _arg_id: &'l1 [u8; 64]) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#claimSecretkeeperEntry(_arg_id))
      }
      fn r#startOrStopAdbd<'a, >(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#startOrStopAdbd(_arg_cid, _arg_start))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVirtualizationServiceInternalAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVirtualizationServiceInternalAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVirtualizationServiceInternal>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVirtualizationServiceInternalAsync<P> for Wrapper {
      fn r#removeMemlockRlimit<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#removeMemlockRlimit())
      }
      fn r#allocateVmContext<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#allocateVmContext())
      }
      fn r#registerVirtualMachine<'a, >(&'a self, _arg_cid: i32, _arg_vm: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#registerVirtualMachine(_arg_cid, _arg_vm))
      }
      fn r#unregisterVirtualMachine<'a, >(&'a self, _arg_cid: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#unregisterVirtualMachine(_arg_cid))
      }
      fn r#forwardAtom<'a, >(&'a self, _arg_atom: &'a crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &'a str) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#forwardAtom(_arg_atom, _arg_requesterUid, _arg_vmIdentifier))
      }
      fn r#debugListVms<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#debugListVms())
      }
      fn r#requestAttestation<'a, >(&'a self, _arg_csr: &'a [u8], _arg_requesterUid: i32, _arg_testMode: bool) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#requestAttestation(_arg_csr, _arg_requesterUid, _arg_testMode))
      }
      fn r#enableTestAttestation<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#enableTestAttestation())
      }
      fn r#isRemoteAttestationSupported<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#isRemoteAttestationSupported())
      }
      fn r#getAssignableDevices<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getAssignableDevices())
      }
      fn r#bindDevicesToVfioDriver<'a, >(&'a self, _arg_devices: &'a [alloc::string::String]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#bindDevicesToVfioDriver(_arg_devices))
      }
      fn r#getDtboFile<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getDtboFile())
      }
      fn r#allocateInstanceId<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<[u8; 64]>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#allocateInstanceId())
      }
      fn r#removeVmInstance<'a, >(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#removeVmInstance(_arg_instanceId))
      }
      fn r#claimVmInstance<'a, >(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#claimVmInstance(_arg_instanceId))
      }
      fn r#setDisplayService<'a, >(&'a self, _arg_ibinder: &'a binder::SpIBinder) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#setDisplayService(_arg_ibinder))
      }
      fn r#clearDisplayService<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#clearDisplayService())
      }
      fn r#waitDisplayService<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::SpIBinder>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#waitDisplayService())
      }
      fn r#createTapInterface<'a, >(&'a self, _arg_ifaceNameSuffix: &'a str) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#createTapInterface(_arg_ifaceNameSuffix))
      }
      fn r#deleteTapInterface<'a, >(&'a self, _arg_tapFd: &'a binder::ParcelFileDescriptor) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#deleteTapInterface(_arg_tapFd))
      }
      fn r#claimSecretkeeperEntry<'a, >(&'a self, _arg_id: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#claimSecretkeeperEntry(_arg_id))
      }
      fn r#startOrStopAdbd<'a, >(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#startOrStopAdbd(_arg_cid, _arg_start))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVirtualizationServiceInternalAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#removeMemlockRlimit: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#allocateVmContext: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#registerVirtualMachine: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
  pub const r#unregisterVirtualMachine: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 3;
  pub const r#forwardAtom: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 4;
  pub const r#debugListVms: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 5;
  pub const r#requestAttestation: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 6;
  pub const r#enableTestAttestation: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 7;
  pub const r#isRemoteAttestationSupported: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 8;
  pub const r#getAssignableDevices: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 9;
  pub const r#bindDevicesToVfioDriver: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 10;
  pub const r#getDtboFile: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 11;
  pub const r#allocateInstanceId: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 12;
  pub const r#removeVmInstance: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 13;
  pub const r#claimVmInstance: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 14;
  pub const r#setDisplayService: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 15;
  pub const r#clearDisplayService: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 16;
  pub const r#waitDisplayService: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 17;
  pub const r#createTapInterface: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 18;
  pub const r#deleteTapInterface: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 19;
  pub const r#claimSecretkeeperEntry: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 20;
  pub const r#startOrStopAdbd: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 21;
}
impl BpVirtualizationServiceInternal {
  fn build_parcel_removeMemlockRlimit(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_removeMemlockRlimit(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_allocateVmContext(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_allocateVmContext(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_registerVirtualMachine(&self, _arg_cid: i32, _arg_vm: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_cid)?;
    aidl_data.write(_arg_vm)?;
    Ok(aidl_data)
  }
  fn read_response_registerVirtualMachine(&self, _arg_cid: i32, _arg_vm: &binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_unregisterVirtualMachine(&self, _arg_cid: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_cid)?;
    Ok(aidl_data)
  }
  fn read_response_unregisterVirtualMachine(&self, _arg_cid: i32, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &str) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_atom)?;
    aidl_data.write(&_arg_requesterUid)?;
    aidl_data.write(_arg_vmIdentifier)?;
    Ok(aidl_data)
  }
  fn read_response_forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &str, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_debugListVms(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_debugListVms(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_requestAttestation(&self, _arg_csr: &[u8], _arg_requesterUid: i32, _arg_testMode: bool) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_csr)?;
    aidl_data.write(&_arg_requesterUid)?;
    aidl_data.write(&_arg_testMode)?;
    Ok(aidl_data)
  }
  fn read_response_requestAttestation(&self, _arg_csr: &[u8], _arg_requesterUid: i32, _arg_testMode: bool, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_enableTestAttestation(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_enableTestAttestation(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_isRemoteAttestationSupported(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_isRemoteAttestationSupported(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<bool> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: bool = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getAssignableDevices(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getAssignableDevices(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_bindDevicesToVfioDriver(&self, _arg_devices: &[alloc::string::String]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_devices)?;
    Ok(aidl_data)
  }
  fn read_response_bindDevicesToVfioDriver(&self, _arg_devices: &[alloc::string::String], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getDtboFile(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getDtboFile(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::ParcelFileDescriptor> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::ParcelFileDescriptor = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_allocateInstanceId(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_allocateInstanceId(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<[u8; 64]> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: [u8; 64] = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_removeVmInstance(&self, _arg_instanceId: &[u8; 64]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_instanceId)?;
    Ok(aidl_data)
  }
  fn read_response_removeVmInstance(&self, _arg_instanceId: &[u8; 64], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
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
  fn read_response_claimVmInstance(&self, _arg_instanceId: &[u8; 64], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_setDisplayService(&self, _arg_ibinder: &binder::SpIBinder) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_ibinder)?;
    Ok(aidl_data)
  }
  fn read_response_setDisplayService(&self, _arg_ibinder: &binder::SpIBinder, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_clearDisplayService(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_clearDisplayService(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_waitDisplayService(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_waitDisplayService(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::SpIBinder> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::SpIBinder = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
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
  fn build_parcel_claimSecretkeeperEntry(&self, _arg_id: &[u8; 64]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_id)?;
    Ok(aidl_data)
  }
  fn read_response_claimSecretkeeperEntry(&self, _arg_id: &[u8; 64], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
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
  fn read_response_startOrStopAdbd(&self, _arg_cid: i32, _arg_start: bool, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IVirtualizationServiceInternal for BpVirtualizationServiceInternal {
  fn r#removeMemlockRlimit<'a, >(&'a self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_removeMemlockRlimit()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#removeMemlockRlimit, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_removeMemlockRlimit(_aidl_reply)
  }
  fn r#allocateVmContext<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext> {
    let _aidl_data = self.build_parcel_allocateVmContext()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#allocateVmContext, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_allocateVmContext(_aidl_reply)
  }
  fn r#registerVirtualMachine<'a, 'l1, >(&'a self, _arg_cid: i32, _arg_vm: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_registerVirtualMachine(_arg_cid, _arg_vm)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#registerVirtualMachine, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_registerVirtualMachine(_arg_cid, _arg_vm, _aidl_reply)
  }
  fn r#unregisterVirtualMachine<'a, >(&'a self, _arg_cid: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_unregisterVirtualMachine(_arg_cid)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#unregisterVirtualMachine, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_unregisterVirtualMachine(_arg_cid, _aidl_reply)
  }
  fn r#forwardAtom<'a, 'l1, 'l2, >(&'a self, _arg_atom: &'l1 crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &'l2 str) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_forwardAtom(_arg_atom, _arg_requesterUid, _arg_vmIdentifier)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#forwardAtom, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_forwardAtom(_arg_atom, _arg_requesterUid, _arg_vmIdentifier, _aidl_reply)
  }
  fn r#debugListVms<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> {
    let _aidl_data = self.build_parcel_debugListVms()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#debugListVms, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_debugListVms(_aidl_reply)
  }
  fn r#requestAttestation<'a, 'l1, >(&'a self, _arg_csr: &'l1 [u8], _arg_requesterUid: i32, _arg_testMode: bool) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>> {
    let _aidl_data = self.build_parcel_requestAttestation(_arg_csr, _arg_requesterUid, _arg_testMode)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#requestAttestation, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_requestAttestation(_arg_csr, _arg_requesterUid, _arg_testMode, _aidl_reply)
  }
  fn r#enableTestAttestation<'a, >(&'a self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_enableTestAttestation()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#enableTestAttestation, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_enableTestAttestation(_aidl_reply)
  }
  fn r#isRemoteAttestationSupported<'a, >(&'a self) -> binder::Result<bool> {
    let _aidl_data = self.build_parcel_isRemoteAttestationSupported()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#isRemoteAttestationSupported, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_isRemoteAttestationSupported(_aidl_reply)
  }
  fn r#getAssignableDevices<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>> {
    let _aidl_data = self.build_parcel_getAssignableDevices()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getAssignableDevices, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_getAssignableDevices(_aidl_reply)
  }
  fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [alloc::string::String]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>> {
    let _aidl_data = self.build_parcel_bindDevicesToVfioDriver(_arg_devices)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#bindDevicesToVfioDriver, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_bindDevicesToVfioDriver(_arg_devices, _aidl_reply)
  }
  fn r#getDtboFile<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor> {
    let _aidl_data = self.build_parcel_getDtboFile()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getDtboFile, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_getDtboFile(_aidl_reply)
  }
  fn r#allocateInstanceId<'a, >(&'a self) -> binder::Result<[u8; 64]> {
    let _aidl_data = self.build_parcel_allocateInstanceId()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#allocateInstanceId, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_allocateInstanceId(_aidl_reply)
  }
  fn r#removeVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_removeVmInstance(_arg_instanceId)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#removeVmInstance, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_removeVmInstance(_arg_instanceId, _aidl_reply)
  }
  fn r#claimVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_claimVmInstance(_arg_instanceId)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#claimVmInstance, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_claimVmInstance(_arg_instanceId, _aidl_reply)
  }
  fn r#setDisplayService<'a, 'l1, >(&'a self, _arg_ibinder: &'l1 binder::SpIBinder) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_setDisplayService(_arg_ibinder)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#setDisplayService, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_setDisplayService(_arg_ibinder, _aidl_reply)
  }
  fn r#clearDisplayService<'a, >(&'a self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_clearDisplayService()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#clearDisplayService, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_clearDisplayService(_aidl_reply)
  }
  fn r#waitDisplayService<'a, >(&'a self) -> binder::Result<binder::SpIBinder> {
    let _aidl_data = self.build_parcel_waitDisplayService()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#waitDisplayService, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_waitDisplayService(_aidl_reply)
  }
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
  fn r#claimSecretkeeperEntry<'a, 'l1, >(&'a self, _arg_id: &'l1 [u8; 64]) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_claimSecretkeeperEntry(_arg_id)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#claimSecretkeeperEntry, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_claimSecretkeeperEntry(_arg_id, _aidl_reply)
  }
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_startOrStopAdbd(_arg_cid, _arg_start)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#startOrStopAdbd, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_startOrStopAdbd(_arg_cid, _arg_start, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVirtualizationServiceInternalAsync<P> for BpVirtualizationServiceInternal {
  fn r#removeMemlockRlimit<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_removeMemlockRlimit() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#removeMemlockRlimit, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_removeMemlockRlimit(_aidl_reply)
      }
    )
  }
  fn r#allocateVmContext<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext>> {
    let _aidl_data = match self.build_parcel_allocateVmContext() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#allocateVmContext, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_allocateVmContext(_aidl_reply)
      }
    )
  }
  fn r#registerVirtualMachine<'a, >(&'a self, _arg_cid: i32, _arg_vm: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_registerVirtualMachine(_arg_cid, _arg_vm) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#registerVirtualMachine, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_registerVirtualMachine(_arg_cid, _arg_vm, _aidl_reply)
      }
    )
  }
  fn r#unregisterVirtualMachine<'a, >(&'a self, _arg_cid: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_unregisterVirtualMachine(_arg_cid) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#unregisterVirtualMachine, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_unregisterVirtualMachine(_arg_cid, _aidl_reply)
      }
    )
  }
  fn r#forwardAtom<'a, >(&'a self, _arg_atom: &'a crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &'a str) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_forwardAtom(_arg_atom, _arg_requesterUid, _arg_vmIdentifier) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#forwardAtom, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_forwardAtom(_arg_atom, _arg_requesterUid, _arg_vmIdentifier, _aidl_reply)
      }
    )
  }
  fn r#debugListVms<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>>> {
    let _aidl_data = match self.build_parcel_debugListVms() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#debugListVms, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_debugListVms(_aidl_reply)
      }
    )
  }
  fn r#requestAttestation<'a, >(&'a self, _arg_csr: &'a [u8], _arg_requesterUid: i32, _arg_testMode: bool) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>> {
    let _aidl_data = match self.build_parcel_requestAttestation(_arg_csr, _arg_requesterUid, _arg_testMode) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#requestAttestation, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_requestAttestation(_arg_csr, _arg_requesterUid, _arg_testMode, _aidl_reply)
      }
    )
  }
  fn r#enableTestAttestation<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_enableTestAttestation() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#enableTestAttestation, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_enableTestAttestation(_aidl_reply)
      }
    )
  }
  fn r#isRemoteAttestationSupported<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<bool>> {
    let _aidl_data = match self.build_parcel_isRemoteAttestationSupported() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#isRemoteAttestationSupported, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_isRemoteAttestationSupported(_aidl_reply)
      }
    )
  }
  fn r#getAssignableDevices<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>>> {
    let _aidl_data = match self.build_parcel_getAssignableDevices() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getAssignableDevices, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getAssignableDevices(_aidl_reply)
      }
    )
  }
  fn r#bindDevicesToVfioDriver<'a, >(&'a self, _arg_devices: &'a [alloc::string::String]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>>> {
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
  fn r#getDtboFile<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::ParcelFileDescriptor>> {
    let _aidl_data = match self.build_parcel_getDtboFile() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getDtboFile, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getDtboFile(_aidl_reply)
      }
    )
  }
  fn r#allocateInstanceId<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<[u8; 64]>> {
    let _aidl_data = match self.build_parcel_allocateInstanceId() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#allocateInstanceId, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_allocateInstanceId(_aidl_reply)
      }
    )
  }
  fn r#removeVmInstance<'a, >(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_removeVmInstance(_arg_instanceId) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#removeVmInstance, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_removeVmInstance(_arg_instanceId, _aidl_reply)
      }
    )
  }
  fn r#claimVmInstance<'a, >(&'a self, _arg_instanceId: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_claimVmInstance(_arg_instanceId) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#claimVmInstance, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_claimVmInstance(_arg_instanceId, _aidl_reply)
      }
    )
  }
  fn r#setDisplayService<'a, >(&'a self, _arg_ibinder: &'a binder::SpIBinder) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_setDisplayService(_arg_ibinder) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#setDisplayService, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_setDisplayService(_arg_ibinder, _aidl_reply)
      }
    )
  }
  fn r#clearDisplayService<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_clearDisplayService() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#clearDisplayService, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_clearDisplayService(_aidl_reply)
      }
    )
  }
  fn r#waitDisplayService<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::SpIBinder>> {
    let _aidl_data = match self.build_parcel_waitDisplayService() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#waitDisplayService, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_waitDisplayService(_aidl_reply)
      }
    )
  }
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
  fn r#claimSecretkeeperEntry<'a, >(&'a self, _arg_id: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_claimSecretkeeperEntry(_arg_id) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#claimSecretkeeperEntry, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_claimSecretkeeperEntry(_arg_id, _aidl_reply)
      }
    )
  }
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_startOrStopAdbd(_arg_cid, _arg_start) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#startOrStopAdbd, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_startOrStopAdbd(_arg_cid, _arg_start, _aidl_reply)
      }
    )
  }
}
impl IVirtualizationServiceInternal for binder::binder_impl::Binder<BnVirtualizationServiceInternal> {
  fn r#removeMemlockRlimit<'a, >(&'a self) -> binder::Result<()> { self.0.r#removeMemlockRlimit() }
  fn r#allocateVmContext<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext> { self.0.r#allocateVmContext() }
  fn r#registerVirtualMachine<'a, 'l1, >(&'a self, _arg_cid: i32, _arg_vm: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine>) -> binder::Result<()> { self.0.r#registerVirtualMachine(_arg_cid, _arg_vm) }
  fn r#unregisterVirtualMachine<'a, >(&'a self, _arg_cid: i32) -> binder::Result<()> { self.0.r#unregisterVirtualMachine(_arg_cid) }
  fn r#forwardAtom<'a, 'l1, 'l2, >(&'a self, _arg_atom: &'l1 crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _arg_requesterUid: i32, _arg_vmIdentifier: &'l2 str) -> binder::Result<()> { self.0.r#forwardAtom(_arg_atom, _arg_requesterUid, _arg_vmIdentifier) }
  fn r#debugListVms<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_23_VirtualMachineDebugInfo>> { self.0.r#debugListVms() }
  fn r#requestAttestation<'a, 'l1, >(&'a self, _arg_csr: &'l1 [u8], _arg_requesterUid: i32, _arg_testMode: bool) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>> { self.0.r#requestAttestation(_arg_csr, _arg_requesterUid, _arg_testMode) }
  fn r#enableTestAttestation<'a, >(&'a self) -> binder::Result<()> { self.0.r#enableTestAttestation() }
  fn r#isRemoteAttestationSupported<'a, >(&'a self) -> binder::Result<bool> { self.0.r#isRemoteAttestationSupported() }
  fn r#getAssignableDevices<'a, >(&'a self) -> binder::Result<alloc::vec::Vec<crate::mangled::_7_android_6_system_21_virtualizationservice_16_AssignableDevice>> { self.0.r#getAssignableDevices() }
  fn r#bindDevicesToVfioDriver<'a, 'l1, >(&'a self, _arg_devices: &'l1 [alloc::string::String]) -> binder::Result<alloc::vec::Vec<binder::Strong<dyn crate::mangled::_7_android_6_system_30_virtualizationservice_internal_12_IBoundDevice>>> { self.0.r#bindDevicesToVfioDriver(_arg_devices) }
  fn r#getDtboFile<'a, >(&'a self) -> binder::Result<binder::ParcelFileDescriptor> { self.0.r#getDtboFile() }
  fn r#allocateInstanceId<'a, >(&'a self) -> binder::Result<[u8; 64]> { self.0.r#allocateInstanceId() }
  fn r#removeVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()> { self.0.r#removeVmInstance(_arg_instanceId) }
  fn r#claimVmInstance<'a, 'l1, >(&'a self, _arg_instanceId: &'l1 [u8; 64]) -> binder::Result<()> { self.0.r#claimVmInstance(_arg_instanceId) }
  fn r#setDisplayService<'a, 'l1, >(&'a self, _arg_ibinder: &'l1 binder::SpIBinder) -> binder::Result<()> { self.0.r#setDisplayService(_arg_ibinder) }
  fn r#clearDisplayService<'a, >(&'a self) -> binder::Result<()> { self.0.r#clearDisplayService() }
  fn r#waitDisplayService<'a, >(&'a self) -> binder::Result<binder::SpIBinder> { self.0.r#waitDisplayService() }
  fn r#createTapInterface<'a, 'l1, >(&'a self, _arg_ifaceNameSuffix: &'l1 str) -> binder::Result<binder::ParcelFileDescriptor> { self.0.r#createTapInterface(_arg_ifaceNameSuffix) }
  fn r#deleteTapInterface<'a, 'l1, >(&'a self, _arg_tapFd: &'l1 binder::ParcelFileDescriptor) -> binder::Result<()> { self.0.r#deleteTapInterface(_arg_tapFd) }
  fn r#claimSecretkeeperEntry<'a, 'l1, >(&'a self, _arg_id: &'l1 [u8; 64]) -> binder::Result<()> { self.0.r#claimSecretkeeperEntry(_arg_id) }
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_cid: i32, _arg_start: bool) -> binder::Result<()> { self.0.r#startOrStopAdbd(_arg_cid, _arg_start) }
}
fn on_transact(_aidl_service: &dyn IVirtualizationServiceInternal, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#removeMemlockRlimit => {
      let _aidl_return = _aidl_service.r#removeMemlockRlimit();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#allocateVmContext => {
      let _aidl_return = _aidl_service.r#allocateVmContext();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#registerVirtualMachine => {
      let _arg_cid: i32 = _aidl_data.read()?;
      let _arg_vm: binder::Strong<dyn crate::mangled::_7_android_6_system_21_virtualizationservice_15_IVirtualMachine> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#registerVirtualMachine(_arg_cid, &_arg_vm);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#unregisterVirtualMachine => {
      let _arg_cid: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#unregisterVirtualMachine(_arg_cid);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#forwardAtom => {
      let _arg_atom: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom = _aidl_data.read()?;
      let _arg_requesterUid: i32 = _aidl_data.read()?;
      let _arg_vmIdentifier: alloc::string::String = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#forwardAtom(&_arg_atom, _arg_requesterUid, &_arg_vmIdentifier);
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
    transactions::r#requestAttestation => {
      let _arg_csr: alloc::vec::Vec<u8> = _aidl_data.read()?;
      let _arg_requesterUid: i32 = _aidl_data.read()?;
      let _arg_testMode: bool = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#requestAttestation(&_arg_csr, _arg_requesterUid, _arg_testMode);
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
    transactions::r#bindDevicesToVfioDriver => {
      let _arg_devices: alloc::vec::Vec<alloc::string::String> = _aidl_data.read()?;
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
    transactions::r#getDtboFile => {
      let _aidl_return = _aidl_service.r#getDtboFile();
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
    transactions::r#setDisplayService => {
      let _arg_ibinder: binder::SpIBinder = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#setDisplayService(&_arg_ibinder);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#clearDisplayService => {
      let _aidl_return = _aidl_service.r#clearDisplayService();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#waitDisplayService => {
      let _aidl_return = _aidl_service.r#waitDisplayService();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
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
    transactions::r#claimSecretkeeperEntry => {
      let _arg_id: [u8; 64] = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#claimSecretkeeperEntry(&_arg_id);
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
    _ => Err(binder::StatusCode::UNKNOWN_TRANSACTION)
  }
}
pub mod r#VmContext {
  use alloc::boxed::Box;
  #[derive(Debug)]
  pub struct r#VmContext {
    pub r#cid: i32,
    pub r#tempDir: alloc::string::String,
  }
  impl Default for r#VmContext {
    fn default() -> Self {
      Self {
        r#cid: 0,
        r#tempDir: Default::default(),
      }
    }
  }
  impl binder::Parcelable for r#VmContext {
    fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_write(|subparcel| {
        subparcel.write(&self.r#cid)?;
        subparcel.write(&self.r#tempDir)?;
        Ok(())
      })
    }
    fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
      parcel.sized_read(|subparcel| {
        if subparcel.has_more_data() {
          self.r#cid = subparcel.read()?;
        }
        if subparcel.has_more_data() {
          self.r#tempDir = subparcel.read()?;
        }
        Ok(())
      })
    }
  }
  binder::impl_serialize_for_parcelable!(r#VmContext);
  binder::impl_deserialize_for_parcelable!(r#VmContext);
  impl binder::binder_impl::ParcelableMetadata for r#VmContext {
    fn get_descriptor() -> &'static str { "android.system.virtualizationservice_internal.IVirtualizationServiceInternal.VmContext" }
  }
}
pub(crate) mod mangled {
 pub use super::r#IVirtualizationServiceInternal as _7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal;
 pub use super::r#VmContext::r#VmContext as _7_android_6_system_30_virtualizationservice_internal_30_IVirtualizationServiceInternal_9_VmContext;
}
