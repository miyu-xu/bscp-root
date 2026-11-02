/*
 * This file is auto-generated.  DO NOT MODIFY.
 * Using: cmd not shown due to `--omit_invocation`
 */
#![forbid(unsafe_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct r#KeInitResult {
  pub r#sessionInitiationInfo: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_21_SessionInitiationInfo,
  pub r#sessionInfo: crate::mangled::_7_android_8_hardware_8_security_9_authgraph_11_SessionInfo,
}
impl Default for r#KeInitResult {
  fn default() -> Self {
    Self {
      r#sessionInitiationInfo: Default::default(),
      r#sessionInfo: Default::default(),
    }
  }
}
impl binder::Parcelable for r#KeInitResult {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#sessionInitiationInfo)?;
      subparcel.write(&self.r#sessionInfo)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#sessionInitiationInfo = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#sessionInfo = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#KeInitResult);
binder::impl_deserialize_for_parcelable!(r#KeInitResult);
impl binder::binder_impl::ParcelableMetadata for r#KeInitResult {
  fn get_descriptor() -> &'static str { "android.hardware.security.authgraph.KeInitResult" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#KeInitResult as _7_android_8_hardware_8_security_9_authgraph_12_KeInitResult;
}
