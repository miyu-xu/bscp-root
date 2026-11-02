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
  IGuestAgent["android.system.virtualizationcommon.IGuestAgent"] {
    native: BnGuestAgent(on_transact),
    proxy: BpGuestAgent {
    },
    async: IGuestAgentAsync(try_into_local_async),
  }
}
pub trait IGuestAgent: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationcommon.IGuestAgent" }
  fn r#shutdownAsync<'a, >(&'a self) -> binder::Result<()>;
  fn r#startDumpVsockServer<'a, 'l1, >(&'a self, _arg_args: &'l1 [alloc::string::String]) -> binder::Result<i32>;
  fn r#trimAsync<'a, >(&'a self) -> binder::Result<()>;
  fn r#userUnlocked<'a, 'l1, >(&'a self, _arg_user_id: i32, _arg_per_user_kek: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>) -> binder::Result<()>;
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_start: bool) -> binder::Result<()>;
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()>;
  fn r#userLocked<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()>;
  fn try_as_async_server<'a>(&'a self) -> Option<&'a (dyn IGuestAgentAsyncServer + Send + Sync)> {
    None
  }
}
pub trait IGuestAgentAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationcommon.IGuestAgent" }
  fn r#shutdownAsync<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#startDumpVsockServer<'a, >(&'a self, _arg_args: &'a [alloc::string::String]) -> binder::BoxFuture<'a, binder::Result<i32>>;
  fn r#trimAsync<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#userUnlocked<'a, >(&'a self, _arg_user_id: i32, _arg_per_user_kek: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_start: bool) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#userLocked<'a, >(&'a self, _arg_userId: i32) -> binder::BoxFuture<'a, binder::Result<()>>;
}
#[::async_trait::async_trait]
pub trait IGuestAgentAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.system.virtualizationcommon.IGuestAgent" }
  async fn r#shutdownAsync<'a, >(&'a self) -> binder::Result<()>;
  async fn r#startDumpVsockServer<'a, 'l1, >(&'a self, _arg_args: &'l1 [alloc::string::String]) -> binder::Result<i32>;
  async fn r#trimAsync<'a, >(&'a self) -> binder::Result<()>;
  async fn r#userUnlocked<'a, 'l1, >(&'a self, _arg_user_id: i32, _arg_per_user_kek: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>) -> binder::Result<()>;
  async fn r#startOrStopAdbd<'a, >(&'a self, _arg_start: bool) -> binder::Result<()>;
  async fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()>;
  async fn r#userLocked<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()>;
}
impl BnGuestAgent {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn IGuestAgent>
  where
    T: IGuestAgentAsyncServer + binder::Interface + Send + Sync + 'static,
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
    impl<T, R> IGuestAgent for Wrapper<T, R>
    where
      T: IGuestAgentAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#shutdownAsync<'a, >(&'a self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#shutdownAsync())
      }
      fn r#startDumpVsockServer<'a, 'l1, >(&'a self, _arg_args: &'l1 [alloc::string::String]) -> binder::Result<i32> {
        self._rt.block_on(self._inner.r#startDumpVsockServer(_arg_args))
      }
      fn r#trimAsync<'a, >(&'a self) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#trimAsync())
      }
      fn r#userUnlocked<'a, 'l1, >(&'a self, _arg_user_id: i32, _arg_per_user_kek: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#userUnlocked(_arg_user_id, _arg_per_user_kek))
      }
      fn r#startOrStopAdbd<'a, >(&'a self, _arg_start: bool) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#startOrStopAdbd(_arg_start))
      }
      fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#userRemoved(_arg_userId))
      }
      fn r#userLocked<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#userLocked(_arg_userId))
      }
      fn try_as_async_server(&self) -> Option<&(dyn IGuestAgentAsyncServer + Send + Sync)> {
        Some(&self._inner)
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
  pub fn try_into_local_async<P: binder::BinderAsyncPool + 'static>(_native: binder::binder_impl::Binder<Self>) -> Option<binder::Strong<dyn IGuestAgentAsync<P>>> {
    struct Wrapper {
      _native: binder::binder_impl::Binder<BnGuestAgent>
    }
    impl binder::Interface for Wrapper {}
    impl<P: binder::BinderAsyncPool> IGuestAgentAsync<P> for Wrapper {
      fn r#shutdownAsync<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#shutdownAsync())
      }
      fn r#startDumpVsockServer<'a, >(&'a self, _arg_args: &'a [alloc::string::String]) -> binder::BoxFuture<'a, binder::Result<i32>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#startDumpVsockServer(_arg_args))
      }
      fn r#trimAsync<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#trimAsync())
      }
      fn r#userUnlocked<'a, >(&'a self, _arg_user_id: i32, _arg_per_user_kek: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#userUnlocked(_arg_user_id, _arg_per_user_kek))
      }
      fn r#startOrStopAdbd<'a, >(&'a self, _arg_start: bool) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#startOrStopAdbd(_arg_start))
      }
      fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#userRemoved(_arg_userId))
      }
      fn r#userLocked<'a, >(&'a self, _arg_userId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
        Box::pin(self._native.try_as_async_server().unwrap().r#userLocked(_arg_userId))
      }
    }
    if _native.try_as_async_server().is_some() {
      Some(binder::Strong::new(Box::new(Wrapper { _native }) as Box<dyn IGuestAgentAsync<P>>))
    } else {
      None
    }
  }
}
pub mod transactions {
  pub const r#shutdownAsync: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#startDumpVsockServer: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 2;
  pub const r#trimAsync: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 4;
  pub const r#userUnlocked: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 5;
  pub const r#startOrStopAdbd: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 6;
  pub const r#userRemoved: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 7;
  pub const r#userLocked: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 8;
}
impl BpGuestAgent {
  fn build_parcel_shutdownAsync(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_shutdownAsync(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    Ok(())
  }
  fn build_parcel_startDumpVsockServer(&self, _arg_args: &[alloc::string::String]) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(_arg_args)?;
    Ok(aidl_data)
  }
  fn read_response_startDumpVsockServer(&self, _arg_args: &[alloc::string::String], _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<i32> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: i32 = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_trimAsync(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_trimAsync(&self, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    Ok(())
  }
  fn build_parcel_userUnlocked(&self, _arg_user_id: i32, _arg_per_user_kek: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_user_id)?;
    aidl_data.write(_arg_per_user_kek)?;
    Ok(aidl_data)
  }
  fn read_response_userUnlocked(&self, _arg_user_id: i32, _arg_per_user_kek: &binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_startOrStopAdbd(&self, _arg_start: bool) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_start)?;
    Ok(aidl_data)
  }
  fn read_response_startOrStopAdbd(&self, _arg_start: bool, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_userRemoved(&self, _arg_userId: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_userId)?;
    Ok(aidl_data)
  }
  fn read_response_userRemoved(&self, _arg_userId: i32, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_userLocked(&self, _arg_userId: i32) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_userId)?;
    Ok(aidl_data)
  }
  fn read_response_userLocked(&self, _arg_userId: i32, _aidl_reply: core::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
}
impl IGuestAgent for BpGuestAgent {
  fn r#shutdownAsync<'a, >(&'a self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_shutdownAsync()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#shutdownAsync, _aidl_data, binder::binder_impl::FLAG_ONEWAY | FLAG_PRIVATE_LOCAL);
    self.read_response_shutdownAsync(_aidl_reply)
  }
  fn r#startDumpVsockServer<'a, 'l1, >(&'a self, _arg_args: &'l1 [alloc::string::String]) -> binder::Result<i32> {
    let _aidl_data = self.build_parcel_startDumpVsockServer(_arg_args)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#startDumpVsockServer, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_startDumpVsockServer(_arg_args, _aidl_reply)
  }
  fn r#trimAsync<'a, >(&'a self) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_trimAsync()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#trimAsync, _aidl_data, binder::binder_impl::FLAG_ONEWAY | FLAG_PRIVATE_LOCAL);
    self.read_response_trimAsync(_aidl_reply)
  }
  fn r#userUnlocked<'a, 'l1, >(&'a self, _arg_user_id: i32, _arg_per_user_kek: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_userUnlocked(_arg_user_id, _arg_per_user_kek)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#userUnlocked, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_userUnlocked(_arg_user_id, _arg_per_user_kek, _aidl_reply)
  }
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_start: bool) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_startOrStopAdbd(_arg_start)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#startOrStopAdbd, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_startOrStopAdbd(_arg_start, _aidl_reply)
  }
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_userRemoved(_arg_userId)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#userRemoved, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_userRemoved(_arg_userId, _aidl_reply)
  }
  fn r#userLocked<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_userLocked(_arg_userId)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#userLocked, _aidl_data, FLAG_PRIVATE_LOCAL);
    self.read_response_userLocked(_arg_userId, _aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> IGuestAgentAsync<P> for BpGuestAgent {
  fn r#shutdownAsync<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_shutdownAsync() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#shutdownAsync, _aidl_data, binder::binder_impl::FLAG_ONEWAY | FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_shutdownAsync(_aidl_reply)
      }
    )
  }
  fn r#startDumpVsockServer<'a, >(&'a self, _arg_args: &'a [alloc::string::String]) -> binder::BoxFuture<'a, binder::Result<i32>> {
    let _aidl_data = match self.build_parcel_startDumpVsockServer(_arg_args) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#startDumpVsockServer, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_startDumpVsockServer(_arg_args, _aidl_reply)
      }
    )
  }
  fn r#trimAsync<'a, >(&'a self) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_trimAsync() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#trimAsync, _aidl_data, binder::binder_impl::FLAG_ONEWAY | FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_trimAsync(_aidl_reply)
      }
    )
  }
  fn r#userUnlocked<'a, >(&'a self, _arg_user_id: i32, _arg_per_user_kek: &'a binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_userUnlocked(_arg_user_id, _arg_per_user_kek) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#userUnlocked, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_userUnlocked(_arg_user_id, _arg_per_user_kek, _aidl_reply)
      }
    )
  }
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_start: bool) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_startOrStopAdbd(_arg_start) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#startOrStopAdbd, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_startOrStopAdbd(_arg_start, _aidl_reply)
      }
    )
  }
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_userRemoved(_arg_userId) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#userRemoved, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_userRemoved(_arg_userId, _aidl_reply)
      }
    )
  }
  fn r#userLocked<'a, >(&'a self, _arg_userId: i32) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_userLocked(_arg_userId) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(core::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#userLocked, _aidl_data, FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_userLocked(_arg_userId, _aidl_reply)
      }
    )
  }
}
impl IGuestAgent for binder::binder_impl::Binder<BnGuestAgent> {
  fn r#shutdownAsync<'a, >(&'a self) -> binder::Result<()> { self.0.r#shutdownAsync() }
  fn r#startDumpVsockServer<'a, 'l1, >(&'a self, _arg_args: &'l1 [alloc::string::String]) -> binder::Result<i32> { self.0.r#startDumpVsockServer(_arg_args) }
  fn r#trimAsync<'a, >(&'a self) -> binder::Result<()> { self.0.r#trimAsync() }
  fn r#userUnlocked<'a, 'l1, >(&'a self, _arg_user_id: i32, _arg_per_user_kek: &'l1 binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK>) -> binder::Result<()> { self.0.r#userUnlocked(_arg_user_id, _arg_per_user_kek) }
  fn r#startOrStopAdbd<'a, >(&'a self, _arg_start: bool) -> binder::Result<()> { self.0.r#startOrStopAdbd(_arg_start) }
  fn r#userRemoved<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()> { self.0.r#userRemoved(_arg_userId) }
  fn r#userLocked<'a, >(&'a self, _arg_userId: i32) -> binder::Result<()> { self.0.r#userLocked(_arg_userId) }
}
fn on_transact(_aidl_service: &dyn IGuestAgent, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> core::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#shutdownAsync => {
      let _aidl_return = _aidl_service.r#shutdownAsync();
      Ok(())
    }
    transactions::r#startDumpVsockServer => {
      let _arg_args: alloc::vec::Vec<alloc::string::String> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#startDumpVsockServer(&_arg_args);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#trimAsync => {
      let _aidl_return = _aidl_service.r#trimAsync();
      Ok(())
    }
    transactions::r#userUnlocked => {
      let _arg_user_id: i32 = _aidl_data.read()?;
      let _arg_per_user_kek: binder::Strong<dyn crate::mangled::_7_android_6_system_20_virtualizationcommon_11_ICEStoreKEK> = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#userUnlocked(_arg_user_id, &_arg_per_user_kek);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#startOrStopAdbd => {
      let _arg_start: bool = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#startOrStopAdbd(_arg_start);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#userRemoved => {
      let _arg_userId: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#userRemoved(_arg_userId);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#userLocked => {
      let _arg_userId: i32 = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#userLocked(_arg_userId);
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
 pub use super::r#IGuestAgent as _7_android_6_system_20_virtualizationcommon_11_IGuestAgent;
}
