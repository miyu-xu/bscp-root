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
  ISecretkeeper["android.hardware.security.secretkeeper.ISecretkeeper"] {
    native: BnSecretkeeper(on_transact),
    proxy: BpSecretkeeper {
      cached_version: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(-1),
      cached_hash: binder::binder_impl::Mutex<Option<alloc::string::String>> = binder::binder_impl::Mutex::new(None)
    },
    async: ISecretkeeperAsync(try_into_local_async),
    stability: binder::binder_impl::Stability::Vintf,
  }
}
pub trait ISecretkeeper: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.security.secretkeeper.ISecretkeeper" }
  fn r#getAuthGraphKe<'a, >(&'a self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange>>;
  fn r#processSecretManagementRequest<'a, 'l1, >(&'a self, _arg_request: &'l1 [u8]) -> binder::Result<alloc::vec::Vec<u8>>;
  fn r#deleteIds<'a, 'l1, >(&'a self, _arg_ids: &'l1 [crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId]) -> binder::Result<()>;
  fn r#deleteAll<'a, >(&'a self) -> binder::Result<()>;
  fn r#getSecretkeeperIdentity<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey>;
  fn r#getInterfaceVersion<'a, >(&'a self) -> binder::Result<i32> {
    Ok(VERSION)
  }
  fn r#getInterfaceHash<'a, >(&'a self) -> binder::Result<alloc::string::String> {
    Ok(HASH.into())
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn ISecretkeeperAsyncServer + Send + Sync)> {
    None
  }
}
pub trait ISecretkeeperAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.security.secretkeeper.ISecretkeeper" }
  fn r#getAuthGraphKe<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange>>>;
  fn r#processSecretManagementRequest<'a, >(&'a self, _arg_request: &'a [u8]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<u8>>>;
  fn r#deleteIds<'a, >(&'a self, _arg_ids: &'a [crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId]) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#deleteAll<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#getSecretkeeperIdentity<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey>>;
  fn r#getInterfaceVersion<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>> {
    Box::pin(async move { Ok(VERSION) })
  }
  fn r#getInterfaceHash<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::string::String>> {
    Box::pin(async move { Ok(HASH.into()) })
  }
}
#[::async_trait::async_trait]
pub trait ISecretkeeperAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.security.secretkeeper.ISecretkeeper" }
  async fn r#getAuthGraphKe<'a, >(&'a self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange>>;
  async fn r#processSecretManagementRequest<'a, 'l1, >(&'a self, _arg_request: &'l1 [u8]) -> binder::Result<alloc::vec::Vec<u8>>;
  async fn r#deleteIds<'a, 'l1, >(&'a self, _arg_ids: &'l1 [crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId]) -> binder::Result<()>;
  async fn r#deleteAll<'a, >(&'a self) -> binder::Result<()>;
  async fn r#getSecretkeeperIdentity<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey>;
}
impl BnSecretkeeper {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn ISecretkeeper>
  where
    T: ISecretkeeperAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> ISecretkeeper for Wrapper<T, R>
    where
      T: ISecretkeeperAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#getAuthGraphKe<'a, >(&'a self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange>> {
        self._rt.block_on(self._inner.r#getAuthGraphKe())
      }
      fn r#processSecretManagementRequest<'a, 'l1, >(&'a self, _arg_request: &'l1 [u8]) -> binder::Result<alloc::vec::Vec<u8>> {
        self._rt.block_on(self._inner.r#processSecretManagementRequest(_arg_request))
      }
      fn r#deleteIds<'a, 'l1, >(&'a self, _arg_ids: &'l1 [crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId]) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#deleteIds(_arg_ids))
      }
      fn r#deleteAll<'a, >(&'a self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#deleteAll())
      }
      fn r#getSecretkeeperIdentity<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey> {
        self._rt.block_on(self._inner.r#getSecretkeeperIdentity())
      }
      fn try_as_async_server(&self) -> Option<&(dyn ISecretkeeperAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn ISecretkeeperAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnSecretkeeper>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> ISecretkeeperAsync<P> for Wrapper {
      fn r#getAuthGraphKe<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getAuthGraphKe())
      }
      fn r#processSecretManagementRequest<'a, >(&'a self, _arg_request: &'a [u8]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<u8>>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#processSecretManagementRequest(_arg_request))
      }
      fn r#deleteIds<'a, >(&'a self, _arg_ids: &'a [crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId]) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#deleteIds(_arg_ids))
      }
      fn r#deleteAll<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#deleteAll())
      }
      fn r#getSecretkeeperIdentity<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#getSecretkeeperIdentity())
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn ISecretkeeperAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#getAuthGraphKe: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#processSecretManagementRequest: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#deleteIds: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
  pub const r#deleteAll: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 3;
  pub const r#getSecretkeeperIdentity: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 4;
  pub const r#getInterfaceVersion: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 16777214;
  pub const r#getInterfaceHash: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 16777213;
}
pub const r#ERROR_UNKNOWN_KEY_ID: i32 = 1;
pub const r#ERROR_INTERNAL_ERROR: i32 = 2;
pub const r#ERROR_REQUEST_MALFORMED: i32 = 3;
pub const VERSION: i32 = 2;
pub const HASH: &str = "865c6c213085ba9679d2ffc1739099679ca38a4b";
impl BpSecretkeeper {
  fn build_parcel_getAuthGraphKe(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getAuthGraphKe(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_processSecretManagementRequest(&self, _arg_request: &[u8]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_request)?;
    Ok(aidl_data)
  }
  fn read_response_processSecretManagementRequest(&self, _arg_request: &[u8], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::vec::Vec<u8>> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::vec::Vec<u8> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_deleteIds(&self, _arg_ids: &[crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_ids)?;
    Ok(aidl_data)
  }
  fn read_response_deleteIds(&self, _arg_ids: &[crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_deleteAll(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_deleteAll(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_getSecretkeeperIdentity(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getSecretkeeperIdentity(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getInterfaceVersion(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getInterfaceVersion(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<i32> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: i32 = _aidl_reply.read()?;
    self.cached_version.store(_aidl_return, core::sync::atomic::Ordering::Relaxed);
    Ok(_aidl_return)
  }
  fn build_parcel_getInterfaceHash(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getInterfaceHash(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<alloc::string::String> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: alloc::string::String = _aidl_reply.read()?;
    {
      let mut _aidl_hash_lock = binder::binder_impl::panic_if_poisoned!(self.cached_hash.lock());
      *_aidl_hash_lock = Some(_aidl_return.clone());
    }
    Ok(_aidl_return)
  }
}
impl ISecretkeeper for BpSecretkeeper {
  fn r#getAuthGraphKe<'a, >(&'a self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange>> {
    let _aidl_data = self.build_parcel_getAuthGraphKe()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getAuthGraphKe, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_getAuthGraphKe(_aidl_reply)
  }
  fn r#processSecretManagementRequest<'a, 'l1, >(&'a self, _arg_request: &'l1 [u8]) -> binder::Result<alloc::vec::Vec<u8>> {
    let _aidl_data = self.build_parcel_processSecretManagementRequest(_arg_request)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#processSecretManagementRequest, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_processSecretManagementRequest(_arg_request, _aidl_reply)
  }
  fn r#deleteIds<'a, 'l1, >(&'a self, _arg_ids: &'l1 [crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId]) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_deleteIds(_arg_ids)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#deleteIds, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_deleteIds(_arg_ids, _aidl_reply)
  }
  fn r#deleteAll<'a, >(&'a self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_deleteAll()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#deleteAll, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_deleteAll(_aidl_reply)
  }
  fn r#getSecretkeeperIdentity<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey> {
    let _aidl_data = self.build_parcel_getSecretkeeperIdentity()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getSecretkeeperIdentity, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_getSecretkeeperIdentity(_aidl_reply)
  }
  fn r#getInterfaceVersion<'a, >(&'a self) -> binder::Result<i32> {
    let _aidl_version = self.cached_version.load(core::sync::atomic::Ordering::Relaxed);
    if _aidl_version != -1 { return Ok(_aidl_version); }
    let _aidl_data = self.build_parcel_getInterfaceVersion()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getInterfaceVersion, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_getInterfaceVersion(_aidl_reply)
  }
  fn r#getInterfaceHash<'a, >(&'a self) -> binder::Result<alloc::string::String> {
    {
      let _aidl_hash_lock = binder::binder_impl::panic_if_poisoned!(self.cached_hash.lock());
      if let Some(ref _aidl_hash) = *_aidl_hash_lock {
        return Ok(_aidl_hash.clone());
      }
    }
    let _aidl_data = self.build_parcel_getInterfaceHash()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getInterfaceHash, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_getInterfaceHash(_aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> ISecretkeeperAsync<P> for BpSecretkeeper {
  fn r#getAuthGraphKe<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange>>> {
    let _aidl_data = match self.build_parcel_getAuthGraphKe() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getAuthGraphKe, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getAuthGraphKe(_aidl_reply)
      }
    )
  }
  fn r#processSecretManagementRequest<'a, >(&'a self, _arg_request: &'a [u8]) -> binder::BoxFuture<'a, binder::Result<alloc::vec::Vec<u8>>> {
    let _aidl_data = match self.build_parcel_processSecretManagementRequest(_arg_request) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#processSecretManagementRequest, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_processSecretManagementRequest(_arg_request, _aidl_reply)
      }
    )
  }
  fn r#deleteIds<'a, >(&'a self, _arg_ids: &'a [crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId]) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_deleteIds(_arg_ids) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#deleteIds, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_deleteIds(_arg_ids, _aidl_reply)
      }
    )
  }
  fn r#deleteAll<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_deleteAll() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#deleteAll, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_deleteAll(_aidl_reply)
      }
    )
  }
  fn r#getSecretkeeperIdentity<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey>> {
    let _aidl_data = match self.build_parcel_getSecretkeeperIdentity() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getSecretkeeperIdentity, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getSecretkeeperIdentity(_aidl_reply)
      }
    )
  }
  fn r#getInterfaceVersion<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>> {
    let _aidl_version = self.cached_version.load(core::sync::atomic::Ordering::Relaxed);
    if _aidl_version != -1 { return Box::pin(core::future::ready(Ok(_aidl_version))); }
    let _aidl_data = match self.build_parcel_getInterfaceVersion() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getInterfaceVersion, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getInterfaceVersion(_aidl_reply)
      }
    )
  }
  fn r#getInterfaceHash<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::string::String>> {
    {
      let _aidl_hash_lock = binder::binder_impl::panic_if_poisoned!(self.cached_hash.lock());
      if let Some(ref _aidl_hash) = *_aidl_hash_lock {
        return Box::pin(core::future::ready(Ok(_aidl_hash.clone())));
      }
    }
    let _aidl_data = match self.build_parcel_getInterfaceHash() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getInterfaceHash, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getInterfaceHash(_aidl_reply)
      }
    )
  }
}
impl ISecretkeeper for binder::binder_impl::Binder<BnSecretkeeper> {
  fn r#getAuthGraphKe<'a, >(&'a self) -> binder::Result<binder::Strong<dyn crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange>> { self.0.r#getAuthGraphKe() }
  fn r#processSecretManagementRequest<'a, 'l1, >(&'a self, _arg_request: &'l1 [u8]) -> binder::Result<alloc::vec::Vec<u8>> { self.0.r#processSecretManagementRequest(_arg_request) }
  fn r#deleteIds<'a, 'l1, >(&'a self, _arg_ids: &'l1 [crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId]) -> binder::Result<()> { self.0.r#deleteIds(_arg_ids) }
  fn r#deleteAll<'a, >(&'a self) -> binder::Result<()> { self.0.r#deleteAll() }
  fn r#getSecretkeeperIdentity<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_9_PublicKey> { self.0.r#getSecretkeeperIdentity() }
  fn r#getInterfaceVersion<'a, >(&'a self) -> binder::Result<i32> { self.0.r#getInterfaceVersion() }
  fn r#getInterfaceHash<'a, >(&'a self) -> binder::Result<alloc::string::String> { self.0.r#getInterfaceHash() }
}
fn on_transact(_aidl_service: &dyn ISecretkeeper, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#getAuthGraphKe => {
      let _aidl_return = _aidl_service.r#getAuthGraphKe();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#processSecretManagementRequest => {
      let _arg_request: alloc::vec::Vec<u8> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#processSecretManagementRequest(&_arg_request);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#deleteIds => {
      let _arg_ids: alloc::vec::Vec<crate::mangled::_7_android_8_hardware_8_security_12_secretkeeper_8_SecretId> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#deleteIds(&_arg_ids);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#deleteAll => {
      let _aidl_return = _aidl_service.r#deleteAll();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getSecretkeeperIdentity => {
      let _aidl_return = _aidl_service.r#getSecretkeeperIdentity();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getInterfaceVersion => {
      let _aidl_return = _aidl_service.r#getInterfaceVersion();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getInterfaceHash => {
      let _aidl_return = _aidl_service.r#getInterfaceHash();
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
 pub use super::r#ISecretkeeper as _7_android_8_hardware_8_security_12_secretkeeper_13_ISecretkeeper;
}
