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
  IAuthGraphKeyExchange["android.hardware.security.authgraph.IAuthGraphKeyExchange"] {
    native: BnAuthGraphKeyExchange(on_transact),
    proxy: BpAuthGraphKeyExchange {
      cached_version: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(-1),
      cached_hash: binder::binder_impl::Mutex<Option<alloc::string::String>> = binder::binder_impl::Mutex::new(None)
    },
    async: IAuthGraphKeyExchangeAsync(try_into_local_async),
    stability: binder::binder_impl::Stability::Vintf,
  }
}
pub trait IAuthGraphKeyExchange: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.security.authgraph.IAuthGraphKeyExchange" }
  fn r#create<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>;
  fn r#init<'a, 'l1, 'l2, 'l3, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'l3 [u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>;
  fn r#finish<'a, 'l1, 'l2, 'l3, 'l4, 'l5, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'l3 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'l4 [u8], _arg_peerVersion: i32, _arg_ownKey: &'l5 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>;
  fn r#authenticationComplete<'a, 'l1, 'l2, >(&'a self, _arg_peerSignature: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'l2 [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>;
  fn r#getInterfaceVersion<'a, >(&'a self) -> binder::Result<i32> {
    Ok(VERSION)
  }
  fn r#getInterfaceHash<'a, >(&'a self) -> binder::Result<alloc::string::String> {
    Ok(HASH.into())
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IAuthGraphKeyExchangeAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IAuthGraphKeyExchangeAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.security.authgraph.IAuthGraphKeyExchange" }
  fn r#create<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>>;
  fn r#init<'a, >(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>>;
  fn r#finish<'a, >(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32, _arg_ownKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>>;
  fn r#authenticationComplete<'a, >(&'a self, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'a [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::BoxFuture<'a, binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>>;
  fn r#getInterfaceVersion<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>> {
    Box::pin(async move { Ok(VERSION) })
  }
  fn r#getInterfaceHash<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<alloc::string::String>> {
    Box::pin(async move { Ok(HASH.into()) })
  }
}
#[::async_trait::async_trait]
pub trait IAuthGraphKeyExchangeAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.security.authgraph.IAuthGraphKeyExchange" }
  async fn r#create<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>;
  async fn r#init<'a, 'l1, 'l2, 'l3, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'l3 [u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>;
  async fn r#finish<'a, 'l1, 'l2, 'l3, 'l4, 'l5, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'l3 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'l4 [u8], _arg_peerVersion: i32, _arg_ownKey: &'l5 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>;
  async fn r#authenticationComplete<'a, 'l1, 'l2, >(&'a self, _arg_peerSignature: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'l2 [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>;
}
impl BnAuthGraphKeyExchange {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IAuthGraphKeyExchange>
  where
    T: IAuthGraphKeyExchangeAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IAuthGraphKeyExchange for Wrapper<T, R>
    where
      T: IAuthGraphKeyExchangeAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#create<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo> {
        self._rt.block_on(self._inner.r#create())
      }
      fn r#init<'a, 'l1, 'l2, 'l3, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'l3 [u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult> {
        self._rt.block_on(self._inner.r#init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion))
      }
      fn r#finish<'a, 'l1, 'l2, 'l3, 'l4, 'l5, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'l3 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'l4 [u8], _arg_peerVersion: i32, _arg_ownKey: &'l5 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo> {
        self._rt.block_on(self._inner.r#finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey))
      }
      fn r#authenticationComplete<'a, 'l1, 'l2, >(&'a self, _arg_peerSignature: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'l2 [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]> {
        self._rt.block_on(self._inner.r#authenticationComplete(_arg_peerSignature, _arg_sharedKeys))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IAuthGraphKeyExchangeAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IAuthGraphKeyExchangeAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnAuthGraphKeyExchange>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IAuthGraphKeyExchangeAsync<P> for Wrapper {
      fn r#create<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#create())
      }
      fn r#init<'a, >(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion))
      }
      fn r#finish<'a, >(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32, _arg_ownKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey))
      }
      fn r#authenticationComplete<'a, >(&'a self, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'a [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::BoxFuture<'a, binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#authenticationComplete(_arg_peerSignature, _arg_sharedKeys))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IAuthGraphKeyExchangeAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#create: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#init: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#finish: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
  pub const r#authenticationComplete: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 3;
  pub const r#getInterfaceVersion: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 16777214;
  pub const r#getInterfaceHash: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 16777213;
}
pub const VERSION: i32 = 1;
pub const HASH: &str = "3758824e7b75acdb1ca66620fb8a8aec0ec6dfcc";
impl BpAuthGraphKeyExchange {
  fn build_parcel_create(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_create(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_init(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &[u8], _arg_peerVersion: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_peerPubKey)?;
    aidl_data.write(_arg_peerId)?;
    aidl_data.write(_arg_peerNonce)?;
    aidl_data.write(&_arg_peerVersion)?;
    Ok(aidl_data)
  }
  fn read_response_init(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_finish(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _arg_ownKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_peerPubKey)?;
    aidl_data.write(_arg_peerId)?;
    aidl_data.write(_arg_peerSignature)?;
    aidl_data.write(_arg_peerNonce)?;
    aidl_data.write(&_arg_peerVersion)?;
    aidl_data.write(_arg_ownKey)?;
    Ok(aidl_data)
  }
  fn read_response_finish(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _arg_ownKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_authenticationComplete(&self, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_peerSignature)?;
    aidl_data.write(_arg_sharedKeys)?;
    Ok(aidl_data)
  }
  fn read_response_authenticationComplete(&self, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2] = _aidl_reply.read()?;
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
impl IAuthGraphKeyExchange for BpAuthGraphKeyExchange {
  fn r#create<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo> {
    let _aidl_data = self.build_parcel_create()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#create, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_create(_aidl_reply)
  }
  fn r#init<'a, 'l1, 'l2, 'l3, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'l3 [u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult> {
    let _aidl_data = self.build_parcel_init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#init, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion, _aidl_reply)
  }
  fn r#finish<'a, 'l1, 'l2, 'l3, 'l4, 'l5, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'l3 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'l4 [u8], _arg_peerVersion: i32, _arg_ownKey: &'l5 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo> {
    let _aidl_data = self.build_parcel_finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#finish, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey, _aidl_reply)
  }
  fn r#authenticationComplete<'a, 'l1, 'l2, >(&'a self, _arg_peerSignature: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'l2 [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]> {
    let _aidl_data = self.build_parcel_authenticationComplete(_arg_peerSignature, _arg_sharedKeys)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#authenticationComplete, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_authenticationComplete(_arg_peerSignature, _arg_sharedKeys, _aidl_reply)
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
impl<P: binder::BinderAsyncPool> IAuthGraphKeyExchangeAsync<P> for BpAuthGraphKeyExchange {
  fn r#create<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>> {
    let _aidl_data = match self.build_parcel_create() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#create, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_create(_aidl_reply)
      }
    )
  }
  fn r#init<'a, >(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>> {
    let _aidl_data = match self.build_parcel_init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#init, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion, _aidl_reply)
      }
    )
  }
  fn r#finish<'a, >(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32, _arg_ownKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>> {
    let _aidl_data = match self.build_parcel_finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#finish, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey, _aidl_reply)
      }
    )
  }
  fn r#authenticationComplete<'a, >(&'a self, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'a [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::BoxFuture<'a, binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>> {
    let _aidl_data = match self.build_parcel_authenticationComplete(_arg_peerSignature, _arg_sharedKeys) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#authenticationComplete, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_authenticationComplete(_arg_peerSignature, _arg_sharedKeys, _aidl_reply)
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
impl IAuthGraphKeyExchange for binder::binder_impl::Binder<BnAuthGraphKeyExchange> {
  fn r#create<'a, >(&'a self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo> { self.0.r#create() }
  fn r#init<'a, 'l1, 'l2, 'l3, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'l3 [u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult> { self.0.r#init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion) }
  fn r#finish<'a, 'l1, 'l2, 'l3, 'l4, 'l5, >(&'a self, _arg_peerPubKey: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'l2 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'l3 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'l4 [u8], _arg_peerVersion: i32, _arg_ownKey: &'l5 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo> { self.0.r#finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey) }
  fn r#authenticationComplete<'a, 'l1, 'l2, >(&'a self, _arg_peerSignature: &'l1 crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'l2 [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]> { self.0.r#authenticationComplete(_arg_peerSignature, _arg_sharedKeys) }
  fn r#getInterfaceVersion<'a, >(&'a self) -> binder::Result<i32> { self.0.r#getInterfaceVersion() }
  fn r#getInterfaceHash<'a, >(&'a self) -> binder::Result<alloc::string::String> { self.0.r#getInterfaceHash() }
}
fn on_transact(_aidl_service: &dyn IAuthGraphKeyExchange, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#create => {
      let _aidl_return = _aidl_service.r#create();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#init => {
      let _arg_peerPubKey: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey = _aidl_data.read()?;
      let _arg_peerId: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity = _aidl_data.read()?;
      let _arg_peerNonce: alloc::vec::Vec<u8> = _aidl_data.read()?;
      let _arg_peerVersion: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#init(&_arg_peerPubKey, &_arg_peerId, &_arg_peerNonce, _arg_peerVersion);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#finish => {
      let _arg_peerPubKey: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey = _aidl_data.read()?;
      let _arg_peerId: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity = _aidl_data.read()?;
      let _arg_peerSignature: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature = _aidl_data.read()?;
      let _arg_peerNonce: alloc::vec::Vec<u8> = _aidl_data.read()?;
      let _arg_peerVersion: i32 = _aidl_data.read()?;
      let _arg_ownKey: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#finish(&_arg_peerPubKey, &_arg_peerId, &_arg_peerSignature, &_arg_peerNonce, _arg_peerVersion, &_arg_ownKey);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#authenticationComplete => {
      let _arg_peerSignature: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature = _aidl_data.read()?;
      let _arg_sharedKeys: [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2] = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#authenticationComplete(&_arg_peerSignature, &_arg_sharedKeys);
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
 pub use super::r#IAuthGraphKeyExchange as _7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange;
}
