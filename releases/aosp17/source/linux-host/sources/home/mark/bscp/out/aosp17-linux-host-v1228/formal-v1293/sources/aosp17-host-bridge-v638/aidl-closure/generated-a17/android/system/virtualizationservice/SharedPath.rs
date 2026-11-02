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
use alloc::boxed::Box;
#[derive(Debug)]
pub struct r#SharedPath {
  pub r#sharedPath: alloc::string::String,
  pub r#hostUid: i32,
  pub r#hostGid: i32,
  pub r#guestUid: i32,
  pub r#guestGid: i32,
  pub r#mask: i32,
  pub r#tag: alloc::string::String,
  pub r#socketPath: alloc::string::String,
}
impl Default for r#SharedPath {
  fn default() -> Self {
    Self {
      r#sharedPath: Default::default(),
      r#hostUid: 0,
      r#hostGid: 0,
      r#guestUid: 0,
      r#guestGid: 0,
      r#mask: 0,
      r#tag: Default::default(),
      r#socketPath: Default::default(),
    }
  }
}
impl binder::Parcelable for r#SharedPath {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#sharedPath)?;
      subparcel.write(&self.r#hostUid)?;
      subparcel.write(&self.r#hostGid)?;
      subparcel.write(&self.r#guestUid)?;
      subparcel.write(&self.r#guestGid)?;
      subparcel.write(&self.r#mask)?;
      subparcel.write(&self.r#tag)?;
      subparcel.write(&self.r#socketPath)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> core::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#sharedPath = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#hostUid = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#hostGid = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#guestUid = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#guestGid = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#mask = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#tag = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#socketPath = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#SharedPath);
binder::impl_deserialize_for_parcelable!(r#SharedPath);
impl binder::binder_impl::ParcelableMetadata for r#SharedPath {
  fn get_descriptor() -> &'static str { "android.system.virtualizationservice.SharedPath" }
}
pub(crate) mod mangled {
 pub use super::r#SharedPath as _7_android_6_system_21_virtualizationservice_10_SharedPath;
}
