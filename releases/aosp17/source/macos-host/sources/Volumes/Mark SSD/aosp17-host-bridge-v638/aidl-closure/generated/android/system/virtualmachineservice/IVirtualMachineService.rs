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
  IVirtualMachineService["android.system.virtualmachineservice.IVirtualMachineService"] {
    native: BnVirtualMachineService(on_transact),
    proxy: BpVirtualMachineService {
    },
    async: IVirtualMachineServiceAsync(try_into_local_async),
  }
}
pub trait IVirtualMachineService: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualmachineservice.IVirtualMachineService" }
  fn r#registerGuestAgent(&self, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()>;
  fn r#notifyPayloadStarted(&self) -> binder::Result<()>;
  fn r#notifyPayloadReady(&self) -> binder::Result<()>;
  fn r#notifyPayloadFinished(&self, _arg_exitCode: i32) -> binder::Result<()>;
  fn r#notifyError(&self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()>;
  fn r#requestAttestation(&self, _arg_csr: &[u8], _arg_testMode: bool) -> binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>;
  fn r#getSecretkeeper(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>>;
  fn r#claimSecretkeeperEntry(&self, _arg_id: &[u8; 64]) -> binder::Result<()>;
  fn r#getHostRpcProvider(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>>;
  fn r#getEncryptedStoreKEK(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>>;
  fn r#forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::Result<()>;
  fn getDefaultImpl() -> IVirtualMachineServiceDefaultRef where Self: Sized {
    DEFAULT_IMPL.lock().unwrap().clone()
  }
  fn setDefaultImpl(d: IVirtualMachineServiceDefaultRef) -> IVirtualMachineServiceDefaultRef where Self: Sized {
    std::mem::replace(&mut *DEFAULT_IMPL.lock().unwrap(), d)
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IVirtualMachineServiceAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IVirtualMachineServiceAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualmachineservice.IVirtualMachineService" }
  fn r#registerGuestAgent<'a>(&'a self, _arg_guestAgent: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#notifyPayloadStarted<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#notifyPayloadReady<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#notifyPayloadFinished<'a>(&'a self, _arg_exitCode: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#notifyError<'a>(&'a self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &'a str) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#requestAttestation<'a>(&'a self, _arg_csr: &'a [u8], _arg_testMode: bool) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>>;
  fn r#getSecretkeeper<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>>>;
  fn r#claimSecretkeeperEntry<'a>(&'a self, _arg_id: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#getHostRpcProvider<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>>>;
  fn r#getEncryptedStoreKEK<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>>>;
  fn r#forwardAtom<'a>(&'a self, _arg_atom: &'a crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IVirtualMachineServiceAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualmachineservice.IVirtualMachineService" }
  async fn r#registerGuestAgent(&self, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()>;
  async fn r#notifyPayloadStarted(&self) -> binder::Result<()>;
  async fn r#notifyPayloadReady(&self) -> binder::Result<()>;
  async fn r#notifyPayloadFinished(&self, _arg_exitCode: i32) -> binder::Result<()>;
  async fn r#notifyError(&self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()>;
  async fn r#requestAttestation(&self, _arg_csr: &[u8], _arg_testMode: bool) -> binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>;
  async fn r#getSecretkeeper(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>>;
  async fn r#claimSecretkeeperEntry(&self, _arg_id: &[u8; 64]) -> binder::Result<()>;
  async fn r#getHostRpcProvider(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>>;
  async fn r#getEncryptedStoreKEK(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>>;
  async fn r#forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::Result<()>;
}
impl BnVirtualMachineService {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IVirtualMachineService>
  where
    T: IVirtualMachineServiceAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IVirtualMachineService for Wrapper<T, R>
    where
      T: IVirtualMachineServiceAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#registerGuestAgent(&self, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#registerGuestAgent(_arg_guestAgent))
      }
      fn r#notifyPayloadStarted(&self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#notifyPayloadStarted())
      }
      fn r#notifyPayloadReady(&self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#notifyPayloadReady())
      }
      fn r#notifyPayloadFinished(&self, _arg_exitCode: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#notifyPayloadFinished(_arg_exitCode))
      }
      fn r#notifyError(&self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#notifyError(_arg_errorCode, _arg_message))
      }
      fn r#requestAttestation(&self, _arg_csr: &[u8], _arg_testMode: bool) -> binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>> {
        self._rt.block_on(self._inner.r#requestAttestation(_arg_csr, _arg_testMode))
      }
      fn r#getSecretkeeper(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>> {
        self._rt.block_on(self._inner.r#getSecretkeeper())
      }
      fn r#claimSecretkeeperEntry(&self, _arg_id: &[u8; 64]) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#claimSecretkeeperEntry(_arg_id))
      }
      fn r#getHostRpcProvider(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>> {
        self._rt.block_on(self._inner.r#getHostRpcProvider())
      }
      fn r#getEncryptedStoreKEK(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>> {
        self._rt.block_on(self._inner.r#getEncryptedStoreKEK())
      }
      fn r#forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#forwardAtom(_arg_atom))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IVirtualMachineServiceAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IVirtualMachineServiceAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnVirtualMachineService>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IVirtualMachineServiceAsync<P> for Wrapper {
      fn r#registerGuestAgent<'a>(&'a self, _arg_guestAgent: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#registerGuestAgent(_arg_guestAgent))
      }
      fn r#notifyPayloadStarted<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#notifyPayloadStarted())
      }
      fn r#notifyPayloadReady<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#notifyPayloadReady())
      }
      fn r#notifyPayloadFinished<'a>(&'a self, _arg_exitCode: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#notifyPayloadFinished(_arg_exitCode))
      }
      fn r#notifyError<'a>(&'a self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &'a str) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#notifyError(_arg_errorCode, _arg_message))
      }
      fn r#requestAttestation<'a>(&'a self, _arg_csr: &'a [u8], _arg_testMode: bool) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#requestAttestation(_arg_csr, _arg_testMode))
      }
      fn r#getSecretkeeper<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getSecretkeeper())
      }
      fn r#claimSecretkeeperEntry<'a>(&'a self, _arg_id: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#claimSecretkeeperEntry(_arg_id))
      }
      fn r#getHostRpcProvider<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getHostRpcProvider())
      }
      fn r#getEncryptedStoreKEK<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getEncryptedStoreKEK())
      }
      fn r#forwardAtom<'a>(&'a self, _arg_atom: &'a crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#forwardAtom(_arg_atom))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IVirtualMachineServiceAsync<P>>))
    } else {
      None
    }
  }
}
pub trait IVirtualMachineServiceDefault: Send + Sync {
  fn r#registerGuestAgent(&self, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#notifyPayloadStarted(&self) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#notifyPayloadReady(&self) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#notifyPayloadFinished(&self, _arg_exitCode: i32) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#notifyError(&self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#requestAttestation(&self, _arg_csr: &[u8], _arg_testMode: bool) -> binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getSecretkeeper(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#claimSecretkeeperEntry(&self, _arg_id: &[u8; 64]) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getHostRpcProvider(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getEncryptedStoreKEK(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
}
pub mod transactions {
  pub const r#registerGuestAgent: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#notifyPayloadStarted: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
  pub const r#notifyPayloadReady: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 3;
  pub const r#notifyPayloadFinished: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 4;
  pub const r#notifyError: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 5;
  pub const r#requestAttestation: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 6;
  pub const r#getSecretkeeper: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 7;
  pub const r#claimSecretkeeperEntry: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 8;
  pub const r#getHostRpcProvider: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 9;
  pub const r#getEncryptedStoreKEK: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 10;
  pub const r#forwardAtom: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 11;
}
pub type IVirtualMachineServiceDefaultRef = Option<std::sync::Arc<dyn IVirtualMachineServiceDefault>>;
static DEFAULT_IMPL: std::sync::Mutex<IVirtualMachineServiceDefaultRef> = std::sync::Mutex::new(None);
pub const r#VM_TOMBSTONES_SERVICE_PORT: i32 = 2000;
impl BpVirtualMachineService {
  fn build_parcel_registerGuestAgent(&self, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_guestAgent)?;
    Ok(aidl_data)
  }
  fn read_response_registerGuestAgent(&self, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#registerGuestAgent(_arg_guestAgent);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_notifyPayloadStarted(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_notifyPayloadStarted(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#notifyPayloadStarted();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_notifyPayloadReady(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_notifyPayloadReady(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#notifyPayloadReady();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_notifyPayloadFinished(&self, _arg_exitCode: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_exitCode)?;
    Ok(aidl_data)
  }
  fn read_response_notifyPayloadFinished(&self, _arg_exitCode: i32, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#notifyPayloadFinished(_arg_exitCode);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_notifyError(&self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_errorCode)?;
    aidl_data.write(_arg_message)?;
    Ok(aidl_data)
  }
  fn read_response_notifyError(&self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#notifyError(_arg_errorCode, _arg_message);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_requestAttestation(&self, _arg_csr: &[u8], _arg_testMode: bool) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_csr)?;
    aidl_data.write(&_arg_testMode)?;
    Ok(aidl_data)
  }
  fn read_response_requestAttestation(&self, _arg_csr: &[u8], _arg_testMode: bool, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#requestAttestation(_arg_csr, _arg_testMode);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getSecretkeeper(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getSecretkeeper(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#getSecretkeeper();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_claimSecretkeeperEntry(&self, _arg_id: &[u8; 64]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_id)?;
    Ok(aidl_data)
  }
  fn read_response_claimSecretkeeperEntry(&self, _arg_id: &[u8; 64], _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#claimSecretkeeperEntry(_arg_id);
      }
    }
    let _aidl_reply = _aidl_reply?;
    Ok(())
  }
  fn build_parcel_getHostRpcProvider(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getHostRpcProvider(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#getHostRpcProvider();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getEncryptedStoreKEK(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getEncryptedStoreKEK(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#getEncryptedStoreKEK();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_atom)?;
    Ok(aidl_data)
  }
  fn read_response_forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IVirtualMachineService>::getDefaultImpl() {
        return _aidl_default_impl.r#forwardAtom(_arg_atom);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IVirtualMachineService for BpVirtualMachineService {
  fn r#registerGuestAgent(&self, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_registerGuestAgent(_arg_guestAgent)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#registerGuestAgent, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_registerGuestAgent(_arg_guestAgent, _aidl_reply)
  }
  fn r#notifyPayloadStarted(&self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_notifyPayloadStarted()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#notifyPayloadStarted, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_notifyPayloadStarted(_aidl_reply)
  }
  fn r#notifyPayloadReady(&self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_notifyPayloadReady()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#notifyPayloadReady, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_notifyPayloadReady(_aidl_reply)
  }
  fn r#notifyPayloadFinished(&self, _arg_exitCode: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_notifyPayloadFinished(_arg_exitCode)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#notifyPayloadFinished, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_notifyPayloadFinished(_arg_exitCode, _aidl_reply)
  }
  fn r#notifyError(&self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_notifyError(_arg_errorCode, _arg_message)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#notifyError, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_notifyError(_arg_errorCode, _arg_message, _aidl_reply)
  }
  fn r#requestAttestation(&self, _arg_csr: &[u8], _arg_testMode: bool) -> binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>> {
    let _aidl_data = self.build_parcel_requestAttestation(_arg_csr, _arg_testMode)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#requestAttestation, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_requestAttestation(_arg_csr, _arg_testMode, _aidl_reply)
  }
  fn r#getSecretkeeper(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>> {
    let _aidl_data = self.build_parcel_getSecretkeeper()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getSecretkeeper, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getSecretkeeper(_aidl_reply)
  }
  fn r#claimSecretkeeperEntry(&self, _arg_id: &[u8; 64]) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_claimSecretkeeperEntry(_arg_id)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#claimSecretkeeperEntry, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_claimSecretkeeperEntry(_arg_id, _aidl_reply)
  }
  fn r#getHostRpcProvider(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>> {
    let _aidl_data = self.build_parcel_getHostRpcProvider()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getHostRpcProvider, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getHostRpcProvider(_aidl_reply)
  }
  fn r#getEncryptedStoreKEK(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>> {
    let _aidl_data = self.build_parcel_getEncryptedStoreKEK()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getEncryptedStoreKEK, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getEncryptedStoreKEK(_aidl_reply)
  }
  fn r#forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_forwardAtom(_arg_atom)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#forwardAtom, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_forwardAtom(_arg_atom, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IVirtualMachineServiceAsync<P> for BpVirtualMachineService {
  fn r#registerGuestAgent<'a>(&'a self, _arg_guestAgent: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_registerGuestAgent(_arg_guestAgent) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#registerGuestAgent, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_registerGuestAgent(_arg_guestAgent, _aidl_reply)
      }
    )
  }
  fn r#notifyPayloadStarted<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_notifyPayloadStarted() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#notifyPayloadStarted, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_notifyPayloadStarted(_aidl_reply)
      }
    )
  }
  fn r#notifyPayloadReady<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_notifyPayloadReady() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#notifyPayloadReady, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_notifyPayloadReady(_aidl_reply)
      }
    )
  }
  fn r#notifyPayloadFinished<'a>(&'a self, _arg_exitCode: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_notifyPayloadFinished(_arg_exitCode) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#notifyPayloadFinished, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_notifyPayloadFinished(_arg_exitCode, _aidl_reply)
      }
    )
  }
  fn r#notifyError<'a>(&'a self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &'a str) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_notifyError(_arg_errorCode, _arg_message) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#notifyError, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_notifyError(_arg_errorCode, _arg_message, _aidl_reply)
      }
    )
  }
  fn r#requestAttestation<'a>(&'a self, _arg_csr: &'a [u8], _arg_testMode: bool) -> binder::BoxFuture<'a, binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>>> {
    let _aidl_data = match self.build_parcel_requestAttestation(_arg_csr, _arg_testMode) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#requestAttestation, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_requestAttestation(_arg_csr, _arg_testMode, _aidl_reply)
      }
    )
  }
  fn r#getSecretkeeper<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>>> {
    let _aidl_data = match self.build_parcel_getSecretkeeper() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getSecretkeeper, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getSecretkeeper(_aidl_reply)
      }
    )
  }
  fn r#claimSecretkeeperEntry<'a>(&'a self, _arg_id: &'a [u8; 64]) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_claimSecretkeeperEntry(_arg_id) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#claimSecretkeeperEntry, _aidl_data, binder::binder_impl::FLAG_ONEWAY | binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_claimSecretkeeperEntry(_arg_id, _aidl_reply)
      }
    )
  }
  fn r#getHostRpcProvider<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>>> {
    let _aidl_data = match self.build_parcel_getHostRpcProvider() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getHostRpcProvider, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getHostRpcProvider(_aidl_reply)
      }
    )
  }
  fn r#getEncryptedStoreKEK<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>>> {
    let _aidl_data = match self.build_parcel_getEncryptedStoreKEK() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getEncryptedStoreKEK, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getEncryptedStoreKEK(_aidl_reply)
      }
    )
  }
  fn r#forwardAtom<'a>(&'a self, _arg_atom: &'a crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_forwardAtom(_arg_atom) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#forwardAtom, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_forwardAtom(_arg_atom, _aidl_reply)
      }
    )
  }
}
impl IVirtualMachineService for binder::binder_impl::Binder<BnVirtualMachineService> {
  fn r#registerGuestAgent(&self, _arg_guestAgent: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent>) -> binder::Result<()> { self.0.r#registerGuestAgent(_arg_guestAgent) }
  fn r#notifyPayloadStarted(&self) -> binder::Result<()> { self.0.r#notifyPayloadStarted() }
  fn r#notifyPayloadReady(&self) -> binder::Result<()> { self.0.r#notifyPayloadReady() }
  fn r#notifyPayloadFinished(&self, _arg_exitCode: i32) -> binder::Result<()> { self.0.r#notifyPayloadFinished(_arg_exitCode) }
  fn r#notifyError(&self, _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode, _arg_message: &str) -> binder::Result<()> { self.0.r#notifyError(_arg_errorCode, _arg_message) }
  fn r#requestAttestation(&self, _arg_csr: &[u8], _arg_testMode: bool) -> binder::Result<Vec<crate::mangled::_7_android_6_system_20_virtualizationcommon_11_Certificate>> { self.0.r#requestAttestation(_arg_csr, _arg_testMode) }
  fn r#getSecretkeeper(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper>> { self.0.r#getSecretkeeper() }
  fn r#claimSecretkeeperEntry(&self, _arg_id: &[u8; 64]) -> binder::Result<()> { self.0.r#claimSecretkeeperEntry(_arg_id) }
  fn r#getHostRpcProvider(&self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_2_os_12_IRpcProvider>> { self.0.r#getHostRpcProvider() }
  fn r#getEncryptedStoreKEK(&self) -> binder::Result<Option<binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_18_IEncryptedStoreKEK>>> { self.0.r#getEncryptedStoreKEK() }
  fn r#forwardAtom(&self, _arg_atom: &crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom) -> binder::Result<()> { self.0.r#forwardAtom(_arg_atom) }
}
fn on_transact(_aidl_service: &dyn IVirtualMachineService, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> std::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#registerGuestAgent => {
      let _arg_guestAgent: binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_IGuestAgent> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#registerGuestAgent(&_arg_guestAgent);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#notifyPayloadStarted => {
      let _aidl_return = _aidl_service.r#notifyPayloadStarted();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#notifyPayloadReady => {
      let _aidl_return = _aidl_service.r#notifyPayloadReady();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#notifyPayloadFinished => {
      let _arg_exitCode: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#notifyPayloadFinished(_arg_exitCode);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#notifyError => {
      let _arg_errorCode: crate::mangled::_7_android_6_system_20_virtualizationcommon_9_ErrorCode = _aidl_data.read()?;
      let _arg_message: String = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#notifyError(_arg_errorCode, &_arg_message);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#requestAttestation => {
      let _arg_csr: Vec<u8> = _aidl_data.read()?;
      let _arg_testMode: bool = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#requestAttestation(&_arg_csr, _arg_testMode);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getSecretkeeper => {
      let _aidl_return = _aidl_service.r#getSecretkeeper();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#claimSecretkeeperEntry => {
      let _arg_id: [u8; 64] = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#claimSecretkeeperEntry(&_arg_id);
      Ok(())
    }
    transactions::r#getHostRpcProvider => {
      let _aidl_return = _aidl_service.r#getHostRpcProvider();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getEncryptedStoreKEK => {
      let _aidl_return = _aidl_service.r#getEncryptedStoreKEK();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#forwardAtom => {
      let _arg_atom: crate::mangled::_7_android_6_system_20_virtualizationcommon_4_Atom = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#forwardAtom(&_arg_atom);
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
 pub use super::r#IVirtualMachineService as _7_android_6_system_21_virtualmachineservice_22_IVirtualMachineService;
}
