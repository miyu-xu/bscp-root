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
  IAuthGraphKeyExchange["android.hardware.security.authgraph.IAuthGraphKeyExchange"] {
    native: BnAuthGraphKeyExchange(on_transact),
    proxy: BpAuthGraphKeyExchange {
    },
    async: IAuthGraphKeyExchangeAsync(try_into_local_async),
    stability: binder::binder_impl::Stability::Vintf,
  }
}
pub trait IAuthGraphKeyExchange: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.security.authgraph.IAuthGraphKeyExchange" }
  fn r#create(&self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>;
  fn r#init(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &[u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>;
  fn r#finish(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _arg_ownKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>;
  fn r#authenticationComplete(&self, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>;
  fn getDefaultImpl() -> IAuthGraphKeyExchangeDefaultRef where Self: Sized {
    DEFAULT_IMPL.lock().unwrap().clone()
  }
  fn setDefaultImpl(d: IAuthGraphKeyExchangeDefaultRef) -> IAuthGraphKeyExchangeDefaultRef where Self: Sized {
    std::mem::replace(&mut *DEFAULT_IMPL.lock().unwrap(), d)
  }
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IAuthGraphKeyExchangeAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IAuthGraphKeyExchangeAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.security.authgraph.IAuthGraphKeyExchange" }
  fn r#create<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>>;
  fn r#init<'a>(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>>;
  fn r#finish<'a>(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32, _arg_ownKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>>;
  fn r#authenticationComplete<'a>(&'a self, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'a [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::BoxFuture<'a, binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>>;
}
#[::async_trait::async_trait]
pub trait IAuthGraphKeyExchangeAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.security.authgraph.IAuthGraphKeyExchange" }
  async fn r#create(&self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>;
  async fn r#init(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &[u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>;
  async fn r#finish(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _arg_ownKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>;
  async fn r#authenticationComplete(&self, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>;
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
      fn dump(&self, _writer: &mut dyn std::io::Write, _args: &[&std::ffi::CStr]) -> std::result::Result<(), binder::StatusCode> { self._inner.dump(_writer, _args) }
    }
    impl<T, R> IAuthGraphKeyExchange for Wrapper<T, R>
    where
      T: IAuthGraphKeyExchangeAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#create(&self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo> {
        self._rt.block_on(self._inner.r#create())
      }
      fn r#init(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &[u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult> {
        self._rt.block_on(self._inner.r#init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion))
      }
      fn r#finish(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _arg_ownKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo> {
        self._rt.block_on(self._inner.r#finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey))
      }
      fn r#authenticationComplete(&self, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]> {
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
      fn r#create<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#create())
      }
      fn r#init<'a>(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion))
      }
      fn r#finish<'a>(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32, _arg_ownKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey))
      }
      fn r#authenticationComplete<'a>(&'a self, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'a [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::BoxFuture<'a, binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>> {
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
pub trait IAuthGraphKeyExchangeDefault: Send + Sync {
  fn r#create(&self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#init(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &[u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#finish(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _arg_ownKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#authenticationComplete(&self, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
}
pub mod transactions {
  pub const r#create: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#init: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#finish: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
  pub const r#authenticationComplete: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 3;
}
pub type IAuthGraphKeyExchangeDefaultRef = Option<std::sync::Arc<dyn IAuthGraphKeyExchangeDefault>>;
static DEFAULT_IMPL: std::sync::Mutex<IAuthGraphKeyExchangeDefaultRef> = std::sync::Mutex::new(None);
impl BpAuthGraphKeyExchange {
  fn build_parcel_create(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_create(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IAuthGraphKeyExchange>::getDefaultImpl() {
        return _aidl_default_impl.r#create();
      }
    }
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
  fn read_response_init(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IAuthGraphKeyExchange>::getDefaultImpl() {
        return _aidl_default_impl.r#init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion);
      }
    }
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
  fn read_response_finish(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _arg_ownKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IAuthGraphKeyExchange>::getDefaultImpl() {
        return _aidl_default_impl.r#finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey);
      }
    }
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
  fn read_response_authenticationComplete(&self, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2], _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as IAuthGraphKeyExchange>::getDefaultImpl() {
        return _aidl_default_impl.r#authenticationComplete(_arg_peerSignature, _arg_sharedKeys);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2] = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
}
impl IAuthGraphKeyExchange for BpAuthGraphKeyExchange {
  fn r#create(&self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo> {
    let _aidl_data = self.build_parcel_create()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#create, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_create(_aidl_reply)
  }
  fn r#init(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &[u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult> {
    let _aidl_data = self.build_parcel_init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#init, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion, _aidl_reply)
  }
  fn r#finish(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _arg_ownKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo> {
    let _aidl_data = self.build_parcel_finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#finish, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey, _aidl_reply)
  }
  fn r#authenticationComplete(&self, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]> {
    let _aidl_data = self.build_parcel_authenticationComplete(_arg_peerSignature, _arg_sharedKeys)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#authenticationComplete, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_authenticationComplete(_arg_peerSignature, _arg_sharedKeys, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IAuthGraphKeyExchangeAsync<P> for BpAuthGraphKeyExchange {
  fn r#create<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo>> {
    let _aidl_data = match self.build_parcel_create() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#create, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_create(_aidl_reply)
      }
    )
  }
  fn r#init<'a>(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult>> {
    let _aidl_data = match self.build_parcel_init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#init, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion, _aidl_reply)
      }
    )
  }
  fn r#finish<'a>(&'a self, _arg_peerPubKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &'a [u8], _arg_peerVersion: i32, _arg_ownKey: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::BoxFuture<'a, binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo>> {
    let _aidl_data = match self.build_parcel_finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#finish, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey, _aidl_reply)
      }
    )
  }
  fn r#authenticationComplete<'a>(&'a self, _arg_peerSignature: &'a crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &'a [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::BoxFuture<'a, binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]>> {
    let _aidl_data = match self.build_parcel_authenticationComplete(_arg_peerSignature, _arg_sharedKeys) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#authenticationComplete, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_authenticationComplete(_arg_peerSignature, _arg_sharedKeys, _aidl_reply)
      }
    )
  }
}
impl IAuthGraphKeyExchange for binder::binder_impl::Binder<BnAuthGraphKeyExchange> {
  fn r#create(&self) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo> { self.0.r#create() }
  fn r#init(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerNonce: &[u8], _arg_peerVersion: i32) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_12_KeInitResult> { self.0.r#init(_arg_peerPubKey, _arg_peerId, _arg_peerNonce, _arg_peerVersion) }
  fn r#finish(&self, _arg_peerPubKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_6_PubKey, _arg_peerId: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_peerNonce: &[u8], _arg_peerVersion: i32, _arg_ownKey: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key) -> binder::Result<crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo> { self.0.r#finish(_arg_peerPubKey, _arg_peerId, _arg_peerSignature, _arg_peerNonce, _arg_peerVersion, _arg_ownKey) }
  fn r#authenticationComplete(&self, _arg_peerSignature: &crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature, _arg_sharedKeys: &[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]) -> binder::Result<[crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2]> { self.0.r#authenticationComplete(_arg_peerSignature, _arg_sharedKeys) }
}
fn on_transact(_aidl_service: &dyn IAuthGraphKeyExchange, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> std::result::Result<(), binder::StatusCode> {
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
      let _arg_peerNonce: Vec<u8> = _aidl_data.read()?;
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
      let _arg_peerNonce: Vec<u8> = _aidl_data.read()?;
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
    _ => Err(binder::StatusCode::UNKNOWN_TRANSACTION)
  }
}
pub(crate) mod mangled {
 pub use super::r#IAuthGraphKeyExchange as _7_android_8_hardware_8_security_9_authgraph_21_IAuthGraphKeyExchange;
}
