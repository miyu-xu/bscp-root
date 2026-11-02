/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct r#SessionInfo {
  pub r#sharedKeys: [crate::mangled::_7_android_8_hardware_8_security_9_authgraph_3_Arc; 2],
  pub r#sessionId: Vec<u8>,
  pub r#signature: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_18_SessionIdSignature,
}
impl Default for r#SessionInfo {
  fn default() -> Self {
    Self {
      r#sharedKeys: [Default::default(), Default::default()],
      r#sessionId: Default::default(),
      r#signature: Default::default(),
    }
  }
}
impl binder::Parcelable for r#SessionInfo {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#sharedKeys)?;
      subparcel.write(&self.r#sessionId)?;
      subparcel.write(&self.r#signature)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#sharedKeys = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#sessionId = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#signature = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#SessionInfo);
binder::impl_deserialize_for_parcelable!(r#SessionInfo);
impl binder::binder_impl::ParcelableMetadata for r#SessionInfo {
  fn get_descriptor() -> &'static str { "android.hardware.security.authgraph.SessionInfo" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#SessionInfo as _7_android_8_hardware_8_security_9_authgraph_11_SessionInfo;
}
