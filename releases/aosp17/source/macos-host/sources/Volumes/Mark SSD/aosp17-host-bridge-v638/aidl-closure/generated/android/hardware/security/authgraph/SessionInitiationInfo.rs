/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct r#SessionInitiationInfo {
  pub r#key: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Key,
  pub r#identity: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_8_Identity,
  pub r#nonce: Vec<u8>,
  pub r#version: i32,
}
impl Default for r#SessionInitiationInfo {
  fn default() -> Self {
    Self {
      r#key: Default::default(),
      r#identity: Default::default(),
      r#nonce: Default::default(),
      r#version: 0,
    }
  }
}
impl binder::Parcelable for r#SessionInitiationInfo {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#key)?;
      subparcel.write(&self.r#identity)?;
      subparcel.write(&self.r#nonce)?;
      subparcel.write(&self.r#version)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#key = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#identity = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#nonce = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#version = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#SessionInitiationInfo);
binder::impl_deserialize_for_parcelable!(r#SessionInitiationInfo);
impl binder::binder_impl::ParcelableMetadata for r#SessionInitiationInfo {
  fn get_descriptor() -> &'static str { "android.hardware.security.authgraph.SessionInitiationInfo" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#SessionInitiationInfo as _7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo;
}
